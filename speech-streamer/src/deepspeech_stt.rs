use deepspeech as ds;
use std::path::PathBuf;
use std::io::{Read};
use std::fs::File;
use std::mem;

fn read_raw_data_from_file() -> Vec<i16> {

    let mut raw_file = File::open("audio.raw").expect("opening audio sample");
    let mut buff: Vec<u8> = Vec::new();
    raw_file.read_to_end(&mut buff).expect("reading audio");
    let mut point_read_buff = [0u8; 2];
    
        buff.chunks_exact(2)
            .map(|c| {
                point_read_buff[0] = c[0]; 
                point_read_buff[1] = c[1];
                i16::from_ne_bytes(point_read_buff)
            }).collect()
}

pub trait Recognizer {
    fn recognize(&mut self) -> String;
    fn feed(&mut self,  data: &[i16]);
}

struct DsRecognizer {
    model: ds::Model,
    stream: ds::Stream,
}

impl Recognizer for DsRecognizer {
    fn recognize(&mut self) -> String {
        let new_stream = self.model.create_stream().expect("initializing new stream context");
        let old_stream = mem::replace(&mut self.stream, new_stream);

        let result = old_stream.finish().expect("Streaming recognition failed");
        result
    }

    fn feed(&mut self, data: &[i16]) {
        self.stream.feed_audio(&data);

    }
}

pub(crate) fn recognizer() -> impl Recognizer {
    let mut model = load_ds_model();
    let stream = model.create_stream().expect("creating stream from model");
    DsRecognizer {
        model, stream
    }
}

fn load_ds_model() -> ds::Model {
    let model_path = PathBuf::from("models/deepspeech-0.9.1-models.pbmm");
    let scorer_path = PathBuf::from("models/deepspeech-0.9.1-models.scorer");
    let mut model = ds::Model::load_from_files(&model_path).expect("loading model");
    model.enable_external_scorer(&scorer_path).expect("enabling scrorer");

    model
}

pub(crate) fn recognize_raw_sample() {
    println!("deepspeech version: {}", ds::deepspeech_version().expect("resolving deepspeech version"));
    let mut model = load_ds_model();

    let sample_rate = model.get_sample_rate();
    println!("model sample rate: {}", sample_rate);

    let data = read_raw_data_from_file();
    
    let result = model.speech_to_text_with_metadata(&data, 5).expect("running model");
    for t in result.transcripts() {
        println!("Candiate (confidence: {}): ", t.confidence());
        let tokens: Vec<&str> = t.tokens().iter().map(|t| t.text().expect("text")).collect();
        println!("Tokens: {}", tokens.join(", "));
        
        println!();
    }
}

pub(crate) fn recognize_raw_sample_as_stream() {
    println!("deepspeech version: {}", ds::deepspeech_version().expect("resolving deepspeech version"));
    let mut model = load_ds_model();

    let sample_rate = model.get_sample_rate();
    println!("model sample rate: {}", sample_rate);

    let data = read_raw_data_from_file();

    let mut streamer = model.create_stream().expect("creating streamer");

    for chunk in data.chunks_exact(16_000 * 2) {
        streamer.feed_audio(&chunk);
        let s = streamer.intermediate_decode().expect("Intermediate decode");
        println!("s: {}", s);
    }
    
}