pub mod symbols;

use crate::simulator::bin_parser::Instruction;

// todo: add in debug symbols
pub fn disassemble(instructions: Vec<Instruction>, bytes_to_skip: Vec<u8>) -> Vec<String> {
    let mut result = Vec::with_capacity(instructions.len());
    for (index, instruction) in instructions.iter().enumerate() {
        match instruction {
            Instruction::Noop => {
                result.push("noop".to_string());
            }
            Instruction::Add(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("add {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("add {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::Subtract(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("sub {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("sub {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::SetCarry => {
                result.push("sc".to_string());
            }
            Instruction::ClearCarry => {
                result.push("clc".to_string());
            }
            Instruction::Xor(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("xor {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("xor {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::Xnor(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("xnor {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("xnor {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::Or(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("or {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("or {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::Nor(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("nor {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("nor {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::And(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("and {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("and {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::Nand(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("nand {:02x?}", one_operand));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("nand {:02x?}, {:02x?}", operand1, operand2));
                }
            }
            Instruction::Not => {
                result.push("not".to_string());
            }
            Instruction::RotateRight => {
                result.push("ror".to_string());
            }
            Instruction::RotateLeft => {
                result.push("rol".to_string());
            }
            Instruction::ShiftRight => {
                result.push("shr".to_string());
            }
            Instruction::ShiftLeft => {
                result.push("shl".to_string());
            }
            Instruction::PushRegisterToStack(register) => {
                result.push(format!("phr {:02x?}", register));
            }
            Instruction::PopRegisterFromStack(register) => {
                result.push(format!("plr {:02x?}", register));
            }
            Instruction::LoadAccumulator(address, immediate) => {
                if let Some(address) = address {
                    result.push(format!("lda {}", address));
                } else if let Some(immediate) = immediate {
                    result.push(format!("lda #{:02x?}", immediate));
                }
            }
            Instruction::StoreAccumulator(address) => {
                result.push(format!("sta {}", address));
            }
            Instruction::CopyAccumulatorToRegister(register) => {
                result.push(format!("cpa {:02x?}", register));
            }
            Instruction::CopyRegisterToAccumulator(register) => {
                result.push(format!("cpr {:02x?}", register));
            }
            Instruction::BranchCarrySet(address) => {
                result.push(format!("bcs {}", address));
            }
            Instruction::BranchCarryClear(address) => {
                result.push(format!("bcc {}", address));
            }
            Instruction::BranchNegative(address) => {
                result.push(format!("bn {}", address));
            }
            Instruction::BranchPositive(address) => {
                result.push(format!("bp {}", address));
            }
            Instruction::BranchEqual(register, address) => {
                result.push(format!("beq {:02x?}, {}", register, address));
            }
            Instruction::BranchNotEqual(register, address) => {
                result.push(format!("bne {:02x?}, {}", register, address));
            }
            Instruction::BranchZero(address) => {
                result.push(format!("bz {}", address));
            }
            Instruction::BranchNotZero(address) => {
                result.push(format!("bnz {}", address));
            }
            Instruction::BranchGreater(register, address) => {
                result.push(format!("bg {:02x?}, {}", register, address));
            }
            Instruction::BranchLess(register, address) => {
                result.push(format!("bl {:02x?}, {}", register, address));
            }
            Instruction::Jump(address) => {
                result.push(format!("jmp {}", address));
            }
            Instruction::PushProgramCounter => {
                result.push("phpc".to_string());
            }
            Instruction::PopProgramCounter => {
                result.push("plpc".to_string());
            }
            Instruction::IncrementProgramCounter => {
                unimplemented!("instruction has been removed but i'm too lazy to actually remove it")
            }
            Instruction::PopProgramCounterSubroutine => {
                result.push("rts".to_string());
            }
        }
        let num_to_skip = bytes_to_skip[index];
        for _ in 0..num_to_skip {
            result.push("".to_string());
        }
    }

    for (index, window) in result.clone().windows(2).enumerate() {
        if let Some(first) = window.get(0) {
            let first = first.as_str();
            if let Some(second) = window.get(1) {
                let second = second.as_str();

                if first.starts_with("phpc") && second.starts_with("jmp %") {
                    result[index] = format!("jsr {}", second.splitn(2, ' ').nth(1).unwrap());
                    result[index + 1] = "".to_string();
                }
            }
        }
    }

    result
}

fn parse_immediate(immediate: u8) -> String {
    format!("{:02x?}", immediate)
}