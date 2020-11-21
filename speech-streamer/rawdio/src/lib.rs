mod cpal_audio;
use std::sync::mpsc;

pub trait AudioStream {
    fn start(&mut self);
    fn pause(&mut self);
    fn stop(self);
}

pub trait AudioInputStream: AudioStream {
    fn receive(&mut self) -> Option<mpsc::Receiver<Vec<i16>>>;
}

pub use cpal_audio::create_input_stream;

pub fn play_raw(data: Vec<i16>) {
    cpal_audio::play_raw(data)
}

pub fn print_debug_info() {
    cpal_audio::print_info();
}
