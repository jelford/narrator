use super::cpal_audio::{self, AudioInputStream, AudioStream};
use super::deepspeech_stt::{self, Recognizer};

pub(crate) fn recognize_stream() {
    let mut audio_input = cpal_audio::create_input_stream();
    let mut recognizer = deepspeech_stt::recognizer();

    let rx_chan = audio_input.receive().expect("Getting audio receive channel");

    audio_input.start();
    
    let mut sample_length = 0usize;

    for sample in rx_chan {
        sample_length += sample.len();
        recognizer.feed(&sample);
        if sample_length >= 16_000 * 2 {
            let out = recognizer.recognize();
            println!(": {}", out);
            sample_length = 0;
        }
    }
}