mod parser;

use clap::{arg, Command, Arg};

fn main() {
    let matches = Command::new("GoldASM Assembler")
        .version("0.1.0-dev")
        .about("Assembler for the Gold assembly language, and tools for using it")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("assemble")
                .about("Assemble the given file")
                .arg(Arg::new("sourceFile").required(true))
                .arg(arg!(-o --output ... "Output file"))
        )
        .subcommand(
            Command::new("simulate")
                .about("Simulate the given file")
                .propagate_version(true)
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("asm")
                        .about("Simulate the given assembly file")
                        .arg(Arg::new("sourceFile").required(true))
                )
                .subcommand(
                    Command::new("bin")
                        .about("Simulate the given binary file")
                        .arg(Arg::new("sourceFile").required(true))
                )
        )
        .get_matches();
    match matches.subcommand() {
        Some(("assemble", sub_matches)) => {
            let target_file = sub_matches.get_one::<String>("sourceFile").unwrap();
            println!("Assembling file {target_file}");
            parser::parse(target_file);
        },
        Some(("simulate", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("asm", sub_matches)) => {
                    println!("Simulating assembly file {}", sub_matches.get_one::<String>("sourceFile").unwrap());
                },
                Some(("bin", sub_matches)) => {
                    println!("Simulating binary file {}", sub_matches.get_one::<String>("sourceFile").unwrap());
                },
                _ => unreachable!("Subcommand is required")
            }
        }
        _ => unreachable!("Subcommand is required"),
    }
}
