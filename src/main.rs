use clap::Parser;
use clap_derive::*;

#[derive(Parser, Debug)]
struct Arguments {
    mode: String,
    input_file: std::path::PathBuf,
}

fn main() {
    let args = Arguments::parse();
    println!("{args:?}")
}
