#![allow(clippy::unusual_byte_groupings, clippy::manual_split_once, clippy::needless_splitn, dead_code)]
// annoying lints that I don't care about
mod asm_parser;
mod assembler;
mod simulator;
mod disassembler;
mod loader;

use clap::{arg, Command, Arg, value_parser};
use crate::disassembler::symbols::SymbolTable;

fn main() {
    let matches = Command::new("GoldASM Assembler")
        .version("0.1.0")
        .about("Assembler and simulator for the Gold assembly language")
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("assemble")
                .about("Assemble the given file")
                .arg(Arg::new("sourceFile").required(true))
                .arg(arg!(-o --output [output]).default_value("out"))
                .arg(arg!(--size [size]).value_parser(value_parser!(u16)).default_value("65535"))
        )
        .subcommand(
            Command::new("simulate")
                .about("Simulate the given binary file (with an optional symbol table)")
                .propagate_version(true)
                .arg_required_else_help(true)
                .arg(Arg::new("sourceFile").required(true))
                .arg(Arg::new("symbolTable").required(false))
        )
        .get_matches();
    match matches.subcommand() {
        Some(("assemble", sub_matches)) => {
            let target_file = sub_matches.get_one::<String>("sourceFile").unwrap();
            let output_file = sub_matches.get_one::<String>("output").unwrap();
            let output_size = sub_matches.get_one::<u16>("size").unwrap();

            let directory;
            let filename;

            if cfg!(target_os = "linux") || !target_file.contains('\\') {
                // linux or windows with forward slashes
                directory = target_file.rsplitn(2, '/').nth(1).unwrap_or(".").to_string() + "/";
                filename = target_file.rsplitn(2, '/').next().unwrap();
                println!("INFO: Assembling file \"{filename}\" in directory \"{}\"", directory.strip_suffix('/').unwrap_or(&directory));
            } else if cfg!(target_os = "windows") {
                // windows with backslashes
                directory = target_file.rsplitn(2, '\\').nth(1).unwrap_or(".").to_string() + "\\";
                filename = target_file.rsplitn(2, '\\').next().unwrap();
                println!("INFO: Assembling file \"{filename}\" in directory \"{}\"", directory.strip_suffix('\\').unwrap_or(&directory));
            } else {
                panic!("Only linux and windows are supported!");
            }
            
            let parsed_values = asm_parser::parse(&directory, filename, SymbolTable::new());
            
            let instructions = asm_parser::postprocess(parsed_values.0, parsed_values.2, parsed_values.1);
            
            let (binary_instructions, symbol_table) = assembler::assemble(instructions.0, *output_size, instructions.1);
            
            assembler::write(&binary_instructions, &directory, &(output_file.to_string() + ".bin"));
            assembler::write(&symbol_table.to_bytes(), &directory, &(output_file.to_string() + ".symbols"));
            
        },
        Some(("simulate", sub_matches)) => {
            let target_file = sub_matches.get_one::<String>("sourceFile").unwrap();
            let symbol_table_file = sub_matches.get_one::<String>("symbolTable");

            if let Some(symbol_table_file) = symbol_table_file {
                println!("Simulating binary file {target_file} with symbol table {symbol_table_file}");
                simulator::run_with_symbol_table(target_file.clone(), symbol_table_file.clone()).unwrap();
            } else {
                println!("Simulating binary file {target_file}");
                simulator::run(target_file.clone()).unwrap();
            }
        }
        _ => unreachable!("Subcommand is required, clap should've already panicked."),
    }
}
