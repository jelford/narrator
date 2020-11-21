use anyhow;
use anyhow::Result;
#[macro_use]
extern crate clap;
use clap::Arg;
mod cpal_audio;
use rawdio::{self, AudioInputStream, AudioStream};
use std::fs::File;
use std::thread;
use std::time::Duration;

use std::io::{BufReader, Read, Write};

arg_enum! {
    #[derive(PartialEq, Debug)]
    enum Mode {
        Record,
        Play,
        Info
    }
}

fn main() -> Result<()> {
    let matches = clap::App::new("rawdio")
        .arg(
            Arg::with_name("mode")
                .required(true)
                .possible_values(&Mode::variants())
                .case_insensitive(true),
        )
        .get_matches();

    let mode = value_t!(matches, "mode", Mode).unwrap();
    match mode {
        Mode::Record => do_input(),
        Mode::Play => do_output(),
        Mode::Info => cpal_audio::print_info(),
    };
    Ok(())
}

fn do_output() {
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

    cpal_audio::play_raw(data);
}

pub(crate) fn do_input() {
    let mut audio_input_stream = rawdio::create_input_stream();
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

    audio_input_stream.start();
    thread::sleep(Duration::from_millis(5000));
    audio_input_stream.stop();

    writer.join().expect("Joining writer thread");
}
