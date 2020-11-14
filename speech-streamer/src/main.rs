use std::env;

mod cpal_audio;

fn main() {
    println!("Hello, world!");
    let mode = env::args().nth(1).expect("Requires mode");
    match mode.as_str() {
        "record" => cpal_audio::do_input(),
        "play" => cpal_audio::do_output(),
        "info" => cpal_audio::print_info(),
        _ => panic!("Unknown command"),
    }
}
