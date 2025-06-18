use crate::asm_parser;

/*
Assembling overview:
pass 1: convert all instructions and parameters to binary, except labeled jumps (inc. subroutines)
pass 2: insert jumps around subroutines that use placeholder addresses
pass 3: calculate addresses for labels and subroutines and replace the placeholders
 */

struct IntermediateAddress {
    address: Option<asm_parser::Address>,
    label: Option<asm_parser::Label>,
    subroutine: Option<asm_parser::Subroutine>,
}

pub fn assemble(instructions: Vec<asm_parser::Instruction>) -> Vec<u8> {
    vec!()
}