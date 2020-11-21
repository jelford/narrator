use clap::{self, arg_enum, value_t, Arg};
use rawdio::{self, AudioInputStream, AudioStream};
mod deepspeech_stt;
mod streamer;
mod vad;
use anyhow::Result;

arg_enum! {
    #[derive(Debug, PartialEq)]
    enum Mode {
        Stream,
        RecognizeRaw,
    }
}

fn main() -> Result<()> {
    let m = clap::App::new("audioriser")
        .version("0.1")
        .arg(
            Arg::with_name("mode")
                .required(true)
                .possible_values(&Mode::variants())
                .case_insensitive(true),
        )
        .get_matches();

    let mode = value_t!(m, "mode", Mode)?;
    match mode {
        Mode::Stream => do_streaming_recognize(),
        Mode::RecognizeRaw => deepspeech_stt::recognize_raw_sample_as_stream(),
    };

    Ok(())
}

fn do_streaming_recognize() {
    let mut stream = rawdio::create_input_stream();
    let rx_chan = stream.receive().expect("Getting audio receive channel");

    stream.start();

    for utterance in streamer::recognize_stream(vad::VoiceEvent::iter_from_audio(rx_chan.into_iter())) {
        println!(": {}", utterance);
    }
}
