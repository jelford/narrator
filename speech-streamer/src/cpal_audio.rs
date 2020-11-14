use cpal::traits::HostTrait;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Stream, SupportedStreamConfigRange};

use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::sync::mpsc::{self};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const DEEPSPEECH_INPUT_FORMAT: cpal::SampleFormat = cpal::SampleFormat::I16;
const DEEPSPEECH_SAMPLE_RATE: cpal::SampleRate = cpal::SampleRate(16000);
const DEEPSPEECH_NUM_CHANNELS: cpal::ChannelCount = 1;

fn play_for_duration(stream: Stream) {
    stream.play().unwrap();
    sleep(Duration::from_millis(5000));
    drop(stream);
}

fn fits_format_requirements(config: &SupportedStreamConfigRange) -> bool {
    config.channels() == DEEPSPEECH_NUM_CHANNELS
        && config.sample_format() == DEEPSPEECH_INPUT_FORMAT
        && config.max_sample_rate() >= DEEPSPEECH_SAMPLE_RATE
        && config.min_sample_rate() <= DEEPSPEECH_SAMPLE_RATE
}

pub(crate) fn do_input() -> () {
    let host = cpal::default_host();
    let (tx, rx) = mpsc::channel::<Vec<f32>>();

    let writer = thread::spawn(move || {
        let mut output_file = File::create("audio.raw").expect("Unable to create output file");
        for d in rx {
            let byte_chunks: Vec<[u8; 4]> = d.iter().map(|p| (p / 3.0).to_ne_bytes()).collect();
            for b in byte_chunks {
                output_file.write_all(&b).expect("Unable to write out data");
            }
        }
    });

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

    let stream = input_device
        .build_input_stream(
            &first_supported_config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut buff = Vec::with_capacity(data.len());
                buff.extend(data);
                tx.send(buff)
                    .expect("Unable to pass data back to main thread");
            },
            |_err| panic!("Received error in audio stream callback thread"),
        )
        .expect("Unable to setup audio stream");

    play_for_duration(stream);
    writer.join().expect("Joining writer thread");
}

pub(crate) fn do_output() -> () {
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

    let supported_format = first_supported_config.sample_format();
    println!("Playback in format: {:?}", supported_format);

    let mut input_file =
        BufReader::new(File::open("audio.raw").expect("Unable to open output file"));

    let mut buff = Box::new(Vec::new());
    input_file.read_to_end(&mut buff).expect("Reading audio in");
    let mut raw = buff.into_iter();

    let stream = output_device
        .build_output_stream(
            &first_supported_config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // let _localbuff = buff;
                let mut point_buff = [0u8; 4];

                for d in data.iter_mut() {
                    let byte = raw.next();
                    match byte {
                        None => {
                            *d = 0.0;
                        }
                        Some(byte) => {
                            point_buff[0] = byte;
                            point_buff[1] = raw.next().unwrap();
                            point_buff[2] = raw.next().unwrap();
                            point_buff[3] = raw.next().unwrap();
                            *d = f32::from_ne_bytes(point_buff);
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

    play_for_duration(stream);
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
