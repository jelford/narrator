use std::env;

mod cpal_audio;
mod deepspeech_stt;
mod streamer;

fn main() {
    let mode = env::args().nth(1).expect("Requires mode");
    match mode.as_str() {
        "record" => cpal_audio::do_input(),
        "play" => cpal_audio::do_output(),
        "info" => cpal_audio::print_info(),
        "recognize" => deepspeech_stt::recognize_raw_sample(),
        "recognize_stream" => deepspeech_stt::recognize_raw_sample_as_stream(),
        "stream" => streamer::recognize_stream(),
        _ => panic!("Unknown command"),
    }
}
