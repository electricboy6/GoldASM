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
                    result.push(format!("add {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("add {operand1:02x?}, {operand2:02x?}"));
                }
            }
            Instruction::Subtract(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("sub {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("sub {operand1:02x?}, {operand2:02x?}"));
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
                    result.push(format!("xor {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("xor {operand1:02x?}, {operand2:02x?}"));
                }
            }
            Instruction::Xnor(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("xnor {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("xnor {operand1:02x?}, {operand2:02x?}"));
                }
            }
            Instruction::Or(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("or {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("or {operand1:02x?}, {operand2:02x?}"));
                }
            }
            Instruction::Nor(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("nor {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("nor {operand1:02x?}, {operand2:02x?}"));
                }
            }
            Instruction::And(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("and {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("and {operand1:02x?}, {operand2:02x?}"));
                }
            }
            Instruction::Nand(one_operand, two_operands) => {
                if let Some(one_operand) = one_operand {
                    result.push(format!("nand {one_operand:02x?}"));
                } else if let Some((operand1, operand2)) = two_operands {
                    result.push(format!("nand {operand1:02x?}, {operand2:02x?}"));
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
                result.push(format!("phr {register:02x?}"));
            }
            Instruction::PopRegisterFromStack(register) => {
                result.push(format!("plr {register:02x?}"));
            }
            Instruction::LoadAccumulator(address, immediate) => {
                if let Some(address) = address {
                    result.push(format!("lda {address}"));
                } else if let Some(immediate) = immediate {
                    result.push(format!("lda #{immediate:02x?}"));
                }
            }
            Instruction::StoreAccumulator(address) => {
                result.push(format!("sta {address}"));
            }
            Instruction::CopyAccumulatorToRegister(register) => {
                result.push(format!("cpa {register:02x?}"));
            }
            Instruction::CopyRegisterToAccumulator(register) => {
                result.push(format!("cpr {register:02x?}"));
            }
            Instruction::BranchCarrySet(address) => {
                result.push(format!("bcs {address}"));
            }
            Instruction::BranchCarryClear(address) => {
                result.push(format!("bcc {address}"));
            }
            Instruction::BranchNegative(address) => {
                result.push(format!("bn {address}"));
            }
            Instruction::BranchPositive(address) => {
                result.push(format!("bp {address}"));
            }
            Instruction::BranchEqual(register, address) => {
                result.push(format!("beq {register:02x?}, {address}"));
            }
            Instruction::BranchNotEqual(register, address) => {
                result.push(format!("bne {register:02x?}, {address}"));
            }
            Instruction::BranchZero(address) => {
                result.push(format!("bz {address}"));
            }
            Instruction::BranchNotZero(address) => {
                result.push(format!("bnz {address}"));
            }
            Instruction::BranchGreater(register, address) => {
                result.push(format!("bg {register:02x?}, {address}"));
            }
            Instruction::BranchLess(register, address) => {
                result.push(format!("bl {register:02x?}, {address}"));
            }
            Instruction::Jump(address) => {
                result.push(format!("jmp {address}"));
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
        if let Some(first) = window.first() {
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
    format!("{immediate:02x?}")
}