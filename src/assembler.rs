use crate::asm_parser;
use crate::asm_parser::Instruction;
/*
Assembling overview:
pass 1: convert all instructions and parameters to binary, except labeled jumps (inc. subroutines)
pass 2: insert jumps around subroutines that use placeholder addresses
pass 3: calculate addresses for labels and subroutines and replace the placeholders
 */

struct IntermediateAddress {
    address: Option<asm_parser::Address>,
    label: Option<asm_parser::Label>,
}

/**
Replaces subroutines and rts's with their corresponding jumps and stack pushes/pops
*/
pub fn preprocess(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let original_instructions = instructions.clone();
    let mut resulting_instructions = Vec::with_capacity((original_instructions.len() as f32 * 1.1f32) as usize);
    for instruction in instructions {
        match instruction {
            Instruction::Subroutine(label) => {
                resulting_instructions.push(Instruction::Jump(None, Some(
                    asm_parser::Label { name: label.clone() + "_EndSubroutine" }
                )));
                resulting_instructions.push(Instruction::Label(label));
                resulting_instructions.push(Instruction::PushProgramCounter);
            }
            Instruction::ReturnFromSubroutine(label) => {
                resulting_instructions.push(Instruction::PopProgramCounter);
                resulting_instructions.push(Instruction::IncrementProgramCounter);
                // label is used to skip over subroutine, so we need it to be after the return code
                resulting_instructions.push(Instruction::Label(label.name)); // postfix is automatically added for us
            }
            Instruction::JumpSubroutine(address, label) => {
                if let Some(label_value) = label {
                    resulting_instructions.push(Instruction::Jump(
                        address, Some(asm_parser::Label { name: label_value.name })
                    ));
                } else {
                    resulting_instructions.push(Instruction::Jump(address, None));
                }
            }
            _ => resulting_instructions.push(instruction)
        }
    }
    resulting_instructions
}

pub fn assemble(instructions: Vec<Instruction>) -> Vec<u8> {
    let processed_instructions = preprocess(instructions);
    vec!()
}