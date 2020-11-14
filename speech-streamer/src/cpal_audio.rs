use cpal::traits::HostTrait;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{SupportedStreamConfigRange};

use std::fs::File;
use std::io::{BufReader, Read, Write};

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const DEEPSPEECH_INPUT_FORMAT: cpal::SampleFormat = cpal::SampleFormat::I16;
const DEEPSPEECH_SAMPLE_RATE: cpal::SampleRate = cpal::SampleRate(16000);
const DEEPSPEECH_NUM_CHANNELS: cpal::ChannelCount = 1;

fn play_stream_for_duration<S: AudioStream>(stream: &mut S) {
    stream.start();
    sleep(Duration::from_millis(5_000));
    stream.pause();
}

fn fits_format_requirements(config: &SupportedStreamConfigRange) -> bool {
    config.channels() == DEEPSPEECH_NUM_CHANNELS
        && config.sample_format() == DEEPSPEECH_INPUT_FORMAT
        && config.max_sample_rate() >= DEEPSPEECH_SAMPLE_RATE
        && config.min_sample_rate() <= DEEPSPEECH_SAMPLE_RATE
}

pub(crate) trait AudioStream {
    fn start(&mut self);
    fn pause(&mut self);
    fn stop(self);
}

pub(crate) trait AudioInputStream: AudioStream {
    fn receive(&mut self) -> Option<mpsc::Receiver<Vec<i16>>>;
}

struct AudioInputStreamContainer {
    stream: cpal::Stream,
    rx: Option<mpsc::Receiver<Vec<i16>>>,
}

impl AudioStream for AudioInputStreamContainer {
    fn start(&mut self) {
        self.stream.play().expect("Unable to play stream");
    }

    fn pause(&mut self) {
        self.stream.pause().expect("unable to pause stream");
    }

    fn stop(self) {
        self.stream.pause().expect("unable to stop stream");
    }
}

impl AudioInputStream for AudioInputStreamContainer {
    fn receive(&mut self) -> Option<mpsc::Receiver<Vec<i16>>> {
        self.rx.take()
    }
}

pub(crate) fn create_input_stream() -> impl AudioInputStream {
    let (tx, rx) = mpsc::channel();

    let stream = stream_audio_to_channel(tx);

    AudioInputStreamContainer {
        stream,
        rx: Some(rx),
    }
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

pub(crate) fn do_input() -> () {
    let mut audio_input_stream = create_input_stream();
    let rx = audio_input_stream
        .receive()
        .expect("Unable to get audio stream receiver");

    let writer = thread::spawn(move || {
        let mut output_file = File::create("audio.raw").expect("Unable to create output file");
        for d in rx {
            let byte_chunks: Vec<[u8; 2]> = d.iter().map(|p| p.to_ne_bytes()).collect();
            for b in byte_chunks {
                output_file.write_all(&b).expect("Unable to write out data");
            }
        }
    });

    play_stream_for_duration(&mut audio_input_stream);
    audio_input_stream.stop();

    writer.join().expect("Joining writer thread");
}

pub(crate) fn play_raw(data: Vec<i16>) -> () {
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
    println!("going to wait for barrier");
    // barrier.wait();
    let _guard = cond
        .wait_while(lock.lock().expect("aquiring finished lock"), |f| !*f)
        .expect("waiting for finished");
    drop(_guard);
    println!("crossed barrier");
    stream.pause().expect("stopping stream");
    drop(stream);
    println!("finished playing in lib");
}

pub(crate) fn do_output() -> () {
    let mut input_file =
        BufReader::new(File::open("audio.raw").expect("Unable to open output file"));

    let mut data = Vec::new();
    input_file
        .read_to_end(&mut data)
        .expect("reading raw input file");

    let mut int_buff = [0u8; 2];
    let data: Vec<i16> = data
        .chunks_exact(2)
        .map(|b| {
            int_buff.copy_from_slice(b);
            i16::from_ne_bytes(int_buff)
        })
        .collect();

    play_raw(data);
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
