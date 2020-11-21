use super::deepspeech_stt::{self, Recognizer};
use super::vad::VoiceEvent;

pub struct RecognizedStrings<I: Iterator<Item = VoiceEvent>, R: Recognizer> {
    recognizer: R,
    voice_events: I,
}

impl<I: Iterator<Item = VoiceEvent>, R: Recognizer> Iterator for RecognizedStrings<I, R> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            let sample = self.voice_events.next()?;
            match sample {
                VoiceEvent::Start(data) | VoiceEvent::Data(data) => {
                    self.recognizer.feed(&data);
                }
                VoiceEvent::End => {
                    let output = self.recognizer.recognize();
                    if !output.is_empty() {
                        return Some(output);
                    }
                }
            }
        }
    }
}

pub(crate) fn recognize_stream<I: Iterator<Item = VoiceEvent>>(
    voice_events: I,
) -> RecognizedStrings<I, impl Recognizer> {
    let recognizer = deepspeech_stt::recognizer();

    RecognizedStrings {
        recognizer,
        voice_events,
    }
}
