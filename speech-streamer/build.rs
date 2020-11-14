


fn main() {
    println!("cargo:rerun-if-changed=deepspeech-lib/libdeepspeech.so");
    println!("cargo:rustc-link-search=native=deepspeech-lib/");
}