use std::collections::VecDeque;

pub enum VoiceEvent {
    Start(Vec<i16>),
    Data(Vec<i16>),
    End,
}

impl VoiceEvent {
    pub fn iter_from_audio<I: Iterator<Item = Vec<i16>>>(
        audio_input: I,
    ) -> impl Iterator<Item = VoiceEvent> {
        VadFilter {
            input: audio_input,
            was_speaking: false,
            transitioned_to_stopped: false,
            state: RingBuff::new(),
            last_few_samples: VecDeque::with_capacity(16_000),
        }
    }
}

struct RingBuff {
    data: [i16; 16_000],
    idx: usize,
}

impl RingBuff {
    fn new() -> RingBuff {
        RingBuff {
            data: [0; 16_000],
            idx: 0,
        }
    }

    fn append(&mut self, d: &[i16]) {
        let mut d_idx = 0;
        while d_idx < d.len() {
            let to_add = usize::min(self.data.len() - self.idx, d.len() - d_idx);
            self.data[self.idx..(self.idx + to_add)].copy_from_slice(&d[d_idx..d_idx + to_add]);
            d_idx += to_add;
            self.idx = (self.idx + to_add) % self.data.len();
        }
    }

    fn sum(&self) -> u32 {
        self.data
            .iter()
            .map(|i| i.saturating_abs() as u32)
            .fold(0u32, |a, b| a.saturating_add(b))
    }
}

struct VadFilter<I: Iterator<Item = Vec<i16>>> {
    input: I,
    was_speaking: bool,
    transitioned_to_stopped: bool,
    state: RingBuff,
    last_few_samples: VecDeque<i16>,
}

impl<I: Iterator<Item = Vec<i16>>> Iterator for VadFilter<I> {
    type Item = VoiceEvent;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.transitioned_to_stopped {
                self.transitioned_to_stopped = false;
                return Some(VoiceEvent::End);
            } else {
                let mut v = self.input.next()?;

                self.state.append(&v);
                let activation = self.state.sum();
                let speaking_now = if !self.was_speaking {
                    activation > 70_000
                } else {
                    activation > 40_000
                };

                if !speaking_now {
                    print!("\r                    \ractivation: {}", activation);
                }

                let (new_was_speaking, new_transitioned_to_not_speaking) =
                    match (self.was_speaking, speaking_now) {
                        (true, false) => (false, true),
                        (true, true) => (true, false),
                        (false, true) => (true, false),
                        (false, false) => (false, false),
                    };

                let transitioned_to_speaking = new_was_speaking && !self.was_speaking;
                self.was_speaking = new_was_speaking;
                self.transitioned_to_stopped = new_transitioned_to_not_speaking;

                let this_sample_length = v.len();
                if speaking_now {
                    if !self.last_few_samples.is_empty() {
                        v.extend(self.last_few_samples.drain(0..));
                        v.rotate_left(this_sample_length);
                        self.last_few_samples.clear();
                    }
                    print!("\r                                                                         \rspeaking now... activation: {}", activation);

                    if transitioned_to_speaking {
                        return Some(VoiceEvent::Start(v));
                    } else {
                        return Some(VoiceEvent::Data(v));
                    }
                } else {
                    let to_drop = (v.len() + self.last_few_samples.len())
                        .saturating_sub(self.last_few_samples.capacity());

                    for _ in 0..to_drop {
                        self.last_few_samples.pop_front();
                    }

                    for i in v.iter() {
                        self.last_few_samples.push_back(*i);
                    }
                    v.clear();
                }
            }
        }
    }
}
