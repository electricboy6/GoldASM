mod asm_parser;
mod assembler;
mod simulator;
mod disassembler;

use clap::{arg, Command, Arg, value_parser};

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
                .arg(arg!(-o --output [output]).default_value("out"))
                .arg(arg!(--size [size]).value_parser(value_parser!(u16)).default_value("65535"))
        )
        .subcommand(
            Command::new("simulate")
                .about("Simulate the given binary file")
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
            
            let directory = target_file.rsplitn(2, '/').nth(1).unwrap().to_string() + "/";
            let filename = target_file.rsplitn(2, '/').nth(0).unwrap();
            println!("INFO: Assembling file \"{filename}\" in directory \"{}\"", directory.strip_suffix('/').unwrap_or(&directory));
            
            let parsed_values = asm_parser::parse(&directory, filename);
            
            let instructions = asm_parser::postprocess(parsed_values.0, parsed_values.1);
            
            let (binary_instructions, symbol_table) = assembler::assemble(instructions, *output_size);
            
            assembler::write(&binary_instructions, &directory, &(output_file.to_string() + ".bin"));
            assembler::write(&symbol_table.to_bytes(), &directory, &(output_file.to_string() + ".symbols"));
            
        },
        Some(("simulate", sub_matches)) => {
            let target_file = sub_matches.get_one::<String>("sourceFile").unwrap();
            let symbol_table_file = sub_matches.get_one::<String>("symbolTable");
            if let Some(symbol_table_file) = symbol_table_file {
                println!("Simulating binary file {} with symbol table {}", target_file, symbol_table_file);
                simulator::run_with_symbol_table(target_file.clone(), symbol_table_file.clone()).unwrap();
            } else {
                println!("Simulating binary file {}", target_file);
                simulator::run(target_file.clone()).unwrap();
            }
        }
        _ => unreachable!("Subcommand is required"),
    }
}
