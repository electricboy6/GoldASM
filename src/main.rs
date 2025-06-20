mod asm_parser;
mod assembler;

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
                .arg(arg!(-o --output [output]).default_value("a.out"))
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
            // Safeness: sourceFile is required so it will never be none
            let target_file = sub_matches.get_one::<String>("sourceFile").unwrap();
            // Safeness: output has a default value and will never be none
            let output_file = sub_matches.get_one::<String>("output").unwrap();
            
            let directory = target_file.rsplitn(2, '/').nth(1).unwrap().to_string() + "/";
            let filename = target_file.rsplitn(2, '/').nth(0).unwrap();
            println!("Assembling file {filename} in directory {directory}");
            
            let parsed_values = asm_parser::parse(&directory, filename);
            
            let instructions = asm_parser::postprocess(parsed_values.0, parsed_values.1);
            
            let binary_instructions = assembler::assemble(instructions);
            
            assembler::write(binary_instructions.clone(), directory, output_file);
            println!("Assembled Binary: {:02x?}", binary_instructions);
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
