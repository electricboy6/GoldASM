use std::io::Write;
use crossterm::style::Stylize;
use crate::asm_parser;
use crate::asm_parser::{AddressMode, Instruction};
use crate::disassembler::symbols::{SymbolTable};
/*
Assembling overview:
pass 1: convert all subroutines, return from subroutines, and jump subroutines to just labels, jumps, and the stack
pass 2: convert everything to binary with placeholder addresses for labels and pointers
pass 3: calculate all addresses
 */

#[derive(Clone, PartialEq, Debug)]
pub struct AssemblerLabel {
    pub name: String,
    pub address: u16,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AssemblerLabelUse {
    pub name: String,
    // INVARIANT: there MUST be a two byte area reserved in the vector for the label
    pub index: u16
}
#[derive(Clone, PartialEq, Debug)]
pub struct AssemblerPointerUse {
    pub pointer: asm_parser::Pointer,
    // INVARIANT: there MUST be an area of the correct size reserved for the pointer's address
    pub index: u16
}
#[derive(Clone, PartialEq, Debug)]
pub struct AssemblerPointer {
    pub name: String,
    pub address: asm_parser::PointerAddress,
}


/// Replaces subroutines and rts's with their corresponding jumps and stack pushes/pops
// assembler pass 1
pub fn preprocess(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let original_instructions = instructions.clone();
    let mut resulting_instructions = Vec::with_capacity(original_instructions.len());
    for instruction in instructions {
        match instruction {
            // add in the start of a subroutine
            Instruction::Subroutine(label) => {
                resulting_instructions.push(Instruction::Jump(None, Some(
                    asm_parser::Label { name: label.clone() + "_EndSubroutine" }
                )));
                resulting_instructions.push(Instruction::Label(label + "_Subroutine"));
            }
            // add in the end of a subroutine
            Instruction::ReturnFromSubroutine(label) => {
                resulting_instructions.push(Instruction::PopProgramCounterSubroutine);
                // label is used to skip over subroutine, so we need it to be after the return code
                resulting_instructions.push(Instruction::Label(label.name)); // postfix is automatically added for us
            }
            // replace jump subroutine with jump
            Instruction::JumpSubroutine(address, label) => {
                resulting_instructions.push(Instruction::PushProgramCounter);
                if let Some(label_value) = label {
                    resulting_instructions.push(Instruction::Jump(
                        address, Some(asm_parser::Label { name: label_value.name + "_Subroutine" })
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

pub fn assemble(instructions: Vec<Instruction>, size: u16) -> (Vec<u8>, SymbolTable) {
    println!("INFO: Assembling combined files");
    // preprocess
    let processed_instructions = preprocess(instructions);
    // make the output vector
    let mut binary_instructions = Vec::with_capacity(size as usize);
    // plus 1 because len is 1 with one item, but addresses start at 0
    while binary_instructions.len() < size as usize + 1 {
        binary_instructions.push(0x00);
    }

    // make the vector of labels
    let mut labels = Vec::new();
    // make the vector of label usages
    let mut label_uses = Vec::new();
    
    // make the vector of pointers
    let mut pointers = Vec::new();
    // make the vector of pointer usages
    let mut pointer_uses = Vec::new();
    
    // make the vector of origin starts
    let mut origins = Vec::new();
    
    // point in memory where we insert
    let mut target_address: usize = 0;
    // max point in memory where we can insert
    let mut max_address: usize = size as usize;

    // self-evident
    let mut symbol_table = SymbolTable::new();

    // iterate through the instructions and insert as we go (assembler pass 2)
    // this is long not because it is complicated, but because there are a lot of instructions to parse
    for instruction in processed_instructions {
        match instruction {
            Instruction::Noop => {
                insert(&mut binary_instructions, 0x00, &mut target_address);
            }
            Instruction::Add(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x01, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x02, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::Subtract(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x03, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x04, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::SetCarry => {
                insert(&mut binary_instructions, 0x05, &mut target_address);
            }
            Instruction::ClearCarry => {
                insert(&mut binary_instructions, 0x06, &mut target_address);
            }
            Instruction::Xor(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x07, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x08, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::Xnor(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x09, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x0A, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::Or(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x0B, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x0C, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::Nor(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x0D, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x0E, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::And(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x0F, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x10, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::Nand(register, tworegister) => {
                if let Some(register) = register {
                    insert(&mut binary_instructions, 0x11, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                } else if let Some(tworegister) = tworegister {
                    insert(&mut binary_instructions, 0x12, &mut target_address);
                    insert(&mut binary_instructions, tworegister.0.address, &mut target_address);
                    insert(&mut binary_instructions, tworegister.1.address, &mut target_address);
                }
            }
            Instruction::Not => {
                insert(&mut binary_instructions, 0x13, &mut target_address);
            }
            Instruction::RotateRight => {
                insert(&mut binary_instructions, 0x14, &mut target_address);
            }
            Instruction::RotateLeft => {
                insert(&mut binary_instructions, 0x15, &mut target_address);
            }
            Instruction::ShiftRight => {
                insert(&mut binary_instructions, 0x16, &mut target_address);
            }
            Instruction::ShiftLeft => {
                insert(&mut binary_instructions, 0x17, &mut target_address);
            }
            Instruction::PushRegister(register) => {
                insert(&mut binary_instructions, 0x21, &mut target_address);
                insert(&mut binary_instructions, register.address, &mut target_address);
            }
            Instruction::PopRegister(register) => {
                insert(&mut binary_instructions, 0x22, &mut target_address);
                insert(&mut binary_instructions, register.address, &mut target_address);
            }
            Instruction::LoadAccumulator(address, immediate) => {
                if let Some(address) = address {
                    // address starts at the byte after the one byte opcode
                    // need to do it before pushing the opcode so we don't have to decode the number size
                    if let Some(pointer) = address.pointer {
                        pointer_uses.push(AssemblerPointerUse {
                            pointer,
                            index: (target_address + 2) as u16
                        });
                    }
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x23, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x24, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                            assert!(address.index.is_some());
                            insert(&mut binary_instructions, address.index.unwrap().address, &mut target_address)
                        }
                        AddressMode::ZeroPage => {
                            insert(&mut binary_instructions, 0x25, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::ZeroPageIndexed => {
                            insert(&mut binary_instructions, 0x26, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                            assert!(address.index.is_some());
                            insert(&mut binary_instructions, address.index.unwrap().address, &mut target_address);
                        }
                    }
                } else {
                    insert(&mut binary_instructions, 0x27, &mut target_address);
                    insert(&mut binary_instructions, immediate.unwrap().value.to_bytes()[0], &mut target_address);
                }
            }
            Instruction::StoreAccumulator(address) => {
                // address starts at the byte after the one byte opcode
                // need to do it before pushing the opcode so we don't have to decode the number size
                if let Some(pointer) = address.pointer {
                    pointer_uses.push(AssemblerPointerUse {
                        pointer,
                        index: (target_address + 2) as u16
                    });
                }
                match address.mode {
                    AddressMode::Absolute => {
                        insert(&mut binary_instructions, 0x28, &mut target_address);
                        append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                    }
                    AddressMode::Indexed => {
                        insert(&mut binary_instructions, 0x29, &mut target_address);
                        append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        assert!(address.index.is_some());
                        insert(&mut binary_instructions, address.index.unwrap().address, &mut target_address);
                    }
                    AddressMode::ZeroPage => {
                        insert(&mut binary_instructions, 0x2A, &mut target_address);
                        append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                    }
                    AddressMode::ZeroPageIndexed => {
                        insert(&mut binary_instructions, 0x2B, &mut target_address);
                        append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        assert!(address.index.is_some());
                        insert(&mut binary_instructions, address.index.unwrap().address, &mut target_address);
                    }
                }
            }
            Instruction::CopyAccumulatorToRegister(register) => {
                insert(&mut binary_instructions, 0x2C, &mut target_address);
                insert(&mut binary_instructions, register.address, &mut target_address);
            }
            Instruction::CopyRegisterToAccumulator(register) => {
                insert(&mut binary_instructions, 0x2D, &mut target_address);
                insert(&mut binary_instructions, register.address, &mut target_address);
            }
            Instruction::BranchIfCarrySet(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x42, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x43, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x42, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfCarryNotSet(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x44, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x45, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x44, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfNegative(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x46, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x47, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x46, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfPositive(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x48, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x49, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x48, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfEqual(register, address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x4A, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x4B, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x4A, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfNotEqual(register, address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x4C, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x4D, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x4C, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfZero(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x4E, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x4F, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x4E, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfNotZero(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x50, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x51, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x50, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfGreater(register, address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x58, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x59, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x58, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::BranchIfLess(register, address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x5A, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x5B, &mut target_address);
                            insert(&mut binary_instructions, register.address, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x5A, &mut target_address);
                    insert(&mut binary_instructions, register.address, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::Jump(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            insert(&mut binary_instructions, 0x52, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        AddressMode::Indexed => {
                            insert(&mut binary_instructions, 0x53, &mut target_address);
                            append(&mut binary_instructions, &mut address.address.to_bytes(), &mut target_address);
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    insert(&mut binary_instructions, 0x52, &mut target_address);
                    label_uses.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (target_address + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                    insert(&mut binary_instructions, 0x00, &mut target_address);
                }
            }
            Instruction::PushProgramCounter => {
                insert(&mut binary_instructions, 0x54, &mut target_address);
            }
            Instruction::PopProgramCounter => {
                insert(&mut binary_instructions, 0x55, &mut target_address);
            }
            Instruction::PopProgramCounterSubroutine => {
                insert(&mut binary_instructions, 0x57, &mut target_address);
            }
            // -------------------- assembler directives --------------------
            Instruction::Label(name) => {
                labels.push(AssemblerLabel {
                    name,
                    address: target_address as u16
                });
            }
            Instruction::Pointer(name, address) => {
                pointers.push(AssemblerPointer {
                    name,
                    address,
                });
            }
            Instruction::Word(value) => {
                insert(&mut binary_instructions, value.value.to_decimal() as u8, &mut target_address);
            }
            Instruction::SetOrigin(address) => {
                if let Some(address) = address {
                    // todo: check if things overlap in this code path as well as in the parameterless .org
                    origins.push(target_address as u16);
                    origins.push(address.address.to_decimal());
                    let target_size = address.address.to_decimal();
                    target_address = target_size as usize;
                } else {
                    assert!(origins.len() > 2, "Attempted to resume at empty segment after first origin when less than 2 origins were set!");
                    let start_point = origins[2];
                    let mut end_point = binary_instructions.len() as u16;
                    if let Some(max_end) = origins.get(3) {
                        end_point = *max_end - 1;
                    }
                    target_address = start_point as usize;
                    max_address = end_point as usize;
                }
            }
            _ => eprintln!("{}", format!("ERROR: Unimplemented instruction! ({instruction:?})").red().bold())
        }
        if target_address > max_address {
            panic!("Tried to overwrite code inside the binary (check your .orgs)!");
        }
    }

    // compute addresses of all labels and replace labels with addresses
    // part 1 of pass 3 in assembling sequence
    for label_use in label_uses {
        let mut target_label = &AssemblerLabel {
            name: "".to_string(),
            address: 0
        };
        for label in labels.iter() {
            if label.name == label_use.name {
                target_label = label;
                break;
            }
        }
        if target_label.name.is_empty() {
            if label_use.name.ends_with("_EndSubroutine") {
                // I totally didn't spend like half an hour trying to debug it when I just had the
                // syntax wrong on subroutines in my test file and added this to make it easier to tell
                panic!("Could not find label \"{}\"! Perhaps you forgot to return from a subroutine?", label_use.name);
            }
            panic!("Could not find label \"{}\"!", label_use.name);
        }

        symbol_table.add_label_use(label_use.clone(), target_label.clone());

        let label_address = target_label.address.to_be_bytes();
        binary_instructions[label_use.index as usize] = label_address[1];
        binary_instructions[(label_use.index - 1) as usize] = label_address[0];
    }
    // replace all pointer usages with the value of the pointer
    // part 2 of pass 3 in assembling sequence
    for pointer_use in pointer_uses {
        let mut target_pointer = &AssemblerPointer{ name: "".to_string(), address: asm_parser::PointerAddress::from_str("%0000") };
        for pointer in pointers.iter() {
            if pointer.name == pointer_use.pointer.name {
                target_pointer = pointer;
                break;
            }
        }
        if target_pointer.name.is_empty() {
            panic!("Could not find pointer \"{}\"!", pointer_use.pointer.name);
        }

        symbol_table.add_pointer_use(pointer_use.clone(), target_pointer.clone());

        let pointer_address = target_pointer.address.address.to_bytes();
        match target_pointer.address.address.size {
            asm_parser::NumberSize::EightBit => {
                binary_instructions[(pointer_use.index - 1) as usize] = pointer_address[0];
                binary_instructions.remove(pointer_use.index as usize);
            }
            asm_parser::NumberSize::SixteenBit => {
                binary_instructions[pointer_use.index as usize] = pointer_address[1];
                binary_instructions[(pointer_use.index - 1) as usize] = pointer_address[0];
            }
        }
    }
    // fill out the symbol table
    for label in labels {
        symbol_table.add_label(label);
    }
    for pointer in pointers {
        symbol_table.add_pointer(pointer);
    }
    
    if binary_instructions.len() > size as usize + 1 {
        panic!("Could not fit file in target size!");
    }

    (binary_instructions, symbol_table)
}

fn insert(array: &mut [u8], value: u8, index: &mut usize) {
    array[*index] = value;
    *index += 1;
}

fn append(array: &mut [u8], values: &mut [u8], index: &mut usize) {
    for value in values {
        array[*index] = *value;
        *index += 1;
    }
}

pub fn write(binary: &Vec<u8>, directory: &str, filename: &str) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.to_string() + filename);

    if file.is_err() {
        eprintln!("{}", format!("WARNING: Unable to create file \"{filename}\" (removing file, trying again)").yellow());
        std::fs::remove_file(directory.to_string() + filename).expect("Failed to remove existing target file!");
        // if it's something else, the call stack will just fill up
        // todo: yes this is lazy error handling, what do you want me to do? this is still unfinished
        write(binary, directory, filename);
        return;
    }
    let mut file = file.unwrap();
    file.write_all(binary).unwrap_or_else(|_| panic!("Unable to write to file \"{filename}\" in directory \"{directory}\"!"));
    file.sync_all().unwrap_or_else(|_| panic!("Unable to write to file \"{filename}\" in directory \"{directory}\"!"));
    println!("Successfully wrote binary to file \"{filename}\"");
}