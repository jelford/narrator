use cpal::traits::HostTrait;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::SupportedStreamConfigRange;

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Condvar, Mutex};

const DEEPSPEECH_INPUT_FORMAT: cpal::SampleFormat = cpal::SampleFormat::I16;
const DEEPSPEECH_SAMPLE_RATE: cpal::SampleRate = cpal::SampleRate(16000);
const DEEPSPEECH_NUM_CHANNELS: cpal::ChannelCount = 1;

fn fits_format_requirements(config: &SupportedStreamConfigRange) -> bool {
    config.channels() == DEEPSPEECH_NUM_CHANNELS
        && config.sample_format() == DEEPSPEECH_INPUT_FORMAT
        && config.max_sample_rate() >= DEEPSPEECH_SAMPLE_RATE
        && config.min_sample_rate() <= DEEPSPEECH_SAMPLE_RATE
}

fn stream_audio_to_channel(tx: Sender<Vec<i16>>) -> cpal::Stream {
    let host = cpal::default_host();
    let input_device = host.default_input_device().expect("No output device");

    let inputs_configs = input_device
        .supported_input_configs()
        .expect("unable to get supported input configurations for default audio device");
    let first_supported_config = inputs_configs
        .filter(fits_format_requirements)
        .last()
        .expect("Didn't find matching input format")
        .with_sample_rate(DEEPSPEECH_SAMPLE_RATE);
    let supported_format = first_supported_config.sample_format();
    println!("Recording in format: {:?}", supported_format);

    input_device
        .build_input_stream(
            &first_supported_config.into(),
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                let mut buff = Vec::with_capacity(data.len());

                buff.extend(data);
                tx.send(buff)
                    .expect("Unable to pass data back to main thread");
            },
            |_err| panic!("Received error in audio stream callback thread"),
        )
        .expect("Unable to setup audio stream")
}

#[allow(dead_code)]
pub fn create_input_stream() -> AudioInputStreamContainer {
    let (tx, rx) = mpsc::channel();

    let stream = stream_audio_to_channel(tx);

    AudioInputStreamContainer {
        stream,
        rx: Some(rx),
    }
}

pub(crate) fn play_raw(data: Vec<i16>) {
    let host = cpal::default_host();
    let output_device = host.default_output_device().expect("No output device");

    let outputs_config = output_device
        .supported_output_configs()
        .expect("unable to get supported output configurations for default audio device");
    let first_supported_config = outputs_config
        .filter(fits_format_requirements)
        .last()
        .expect("No supported configs found")
        .with_sample_rate(DEEPSPEECH_SAMPLE_RATE);

    let mut raw = data.into_iter();

    let lock_pair = Arc::new((Mutex::new(false), Condvar::new()));
    let lock_pair2 = lock_pair.clone();

    let stream = output_device
        .build_output_stream(
            &first_supported_config.into(),
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                let (lock, cond) = &*lock_pair2;

                for d in data.iter_mut() {
                    if *lock.lock().expect("lock") {
                        *d = 0;
                        continue;
                    }

                    let byte = raw.next();
                    match byte {
                        None => {
                            *d = 0;
                            *lock.lock().expect("lock") = true;
                            cond.notify_all();
                        }
                        Some(point) => {
                            *d = point;
                        }
                    }
                }
            },
            |_err| {
                panic!("Received error in audio stream callback thread")
                // error handling
            },
        )
        .expect("Unable to setup audio stream");

    let (lock, cond) = &*lock_pair;

    stream.play().expect("Playing stream");
    // barrier.wait();
    let _guard = cond
        .wait_while(lock.lock().expect("aquiring finished lock"), |f| !*f)
        .expect("waiting for finished");
    drop(_guard);
    stream.pause().expect("stopping stream");
    drop(stream);
}

pub(crate) fn print_info() {
    let host = cpal::default_host();

    for d in host.devices().expect("Unable to enumerate devices") {
        println!(
            "Found device: {}",
            d.name().expect("can't fetch name of device")
        );
    }

    let default_input_device = host
        .default_input_device()
        .expect("Unable to find default input device");
    println!(
        "Default input device: {}",
        default_input_device
            .name()
            .expect("resolving input device name")
    );

    let input_configs = default_input_device
        .supported_input_configs()
        .expect("Unable to get configs of default input");
    for ic in input_configs {
        println!("config: {:?}", ic);
    }

    let default_output_device = host
        .default_output_device()
        .expect("Unable to find default output device");
    println!(
        "Default output device: {}",
        default_output_device
            .name()
            .expect("resolving output device name")
    );
}

pub struct AudioInputStreamContainer {
    stream: cpal::Stream,
    rx: Option<mpsc::Receiver<Vec<i16>>>,
}

impl AudioInputStreamContainer {
    pub(crate) fn play(&mut self) {
        self.stream.play().expect("playing audio stream");
    }

    pub(crate) fn pause(&mut self) {
        self.stream.pause().expect("pausing audio stream");
    }
}


impl super::AudioStream for AudioInputStreamContainer {
    fn start(&mut self) {
        self.play();
    }

    fn pause(&mut self) {
        self.pause();
    }

    fn stop(mut self) {
        self.pause();
    }
}

impl super::AudioInputStream for AudioInputStreamContainer {
    fn receive(&mut self) -> Option<mpsc::Receiver<Vec<i16>>> {
        self.rx.take()
    }
}
