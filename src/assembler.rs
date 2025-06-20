use std::io::Write;
use crate::asm_parser;
use crate::asm_parser::{AddressMode, Instruction};
/*
Assembling overview:
pass 1: convert all subroutines, return from subroutines, and jump subroutines to just labels, jumps, and the stack
pass 2: convert everything to binary with placeholder addresses for labels
pass 3: calculate all addresses
 */

#[derive(Clone, PartialEq, Debug)]
struct AssemblerLabel {
    pub name: String,
    pub index: u16,
}

#[derive(Clone, PartialEq, Debug)]
struct AssemblerLabelUse {
    pub name: String,
    // INVARIANT: there MUST be a two byte area reserved in the vector for the label
    pub index: u16
}

/**
Replaces subroutines and rts's with their corresponding jumps and stack pushes/pops
*/
pub fn preprocess(instructions: Vec<Instruction>) -> Vec<Instruction> {
    let original_instructions = instructions.clone();
    let mut resulting_instructions = Vec::with_capacity(original_instructions.len());
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
    // preprocess
    let processed_instructions = preprocess(instructions);
    // make the output vector
    let mut binary_instructions = Vec::with_capacity(processed_instructions.len());

    // make the vector of labels
    let mut labels = Vec::new();
    // make the vector of label usages
    let mut label_usages = Vec::new();

    // iterate through the instructions and insert as we go (assembler pass 2)
    for instruction in processed_instructions {
        match instruction {
            Instruction::Noop => {
                binary_instructions.push(0x00);
            }
            Instruction::Add(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x01);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x02);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::Subtract(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x03);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x04);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::SetCarry => {
                binary_instructions.push(0x05);
            }
            Instruction::ClearCarry => {
                binary_instructions.push(0x06);
            }
            Instruction::Xor(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x07);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x08);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::Xnor(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x09);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x0A);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::Or(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x0B);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x0C);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::Nor(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x0D);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x0E);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::And(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x0F);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x10);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::Nand(register, tworegister) => {
                if let Some(register) = register {
                    binary_instructions.push(0x11);
                    binary_instructions.push(register.address);
                } else if let Some(tworegister) = tworegister {
                    binary_instructions.push(0x12);
                    binary_instructions.push(tworegister.0.address);
                    binary_instructions.push(tworegister.1.address);
                }
            }
            Instruction::Not => {
                binary_instructions.push(0x13);
            }
            Instruction::RotateRight => {
                binary_instructions.push(0x14);
            }
            Instruction::RotateLeft => {
                binary_instructions.push(0x15);
            }
            Instruction::ShiftRight => {
                binary_instructions.push(0x16);
            }
            Instruction::ShiftLeft => {
                binary_instructions.push(0x17);
            }
            Instruction::PushRegister(register) => {
                binary_instructions.push(0x21);
                binary_instructions.push(register.address);
            }
            Instruction::PopRegister(register) => {
                binary_instructions.push(0x22);
                binary_instructions.push(register.address);
            }
            Instruction::LoadAccumulator(address, immediate) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x23);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x24);
                            binary_instructions.append(&mut address.address.to_bytes());
                            assert!(address.offset.is_some());
                            binary_instructions.push(address.offset.unwrap().address)
                        }
                        AddressMode::ZeroPage => {
                            binary_instructions.push(0x25);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::ZeroPageIndexed => {
                            binary_instructions.push(0x26);
                            binary_instructions.append(&mut address.address.to_bytes());
                            assert!(address.offset.is_some());
                            binary_instructions.push(address.offset.unwrap().address)
                        }
                    }
                } else {
                    binary_instructions.push(0x27);
                    binary_instructions.push(immediate.unwrap().value.to_bytes()[0])
                }
            }
            Instruction::StoreAccumulator(address) => {
                match address.mode {
                    AddressMode::Absolute => {
                        binary_instructions.push(0x28);
                        binary_instructions.append(&mut address.address.to_bytes());
                    }
                    AddressMode::Indexed => {
                        binary_instructions.push(0x29);
                        binary_instructions.append(&mut address.address.to_bytes());
                        assert!(address.offset.is_some());
                        binary_instructions.push(address.offset.unwrap().address)
                    }
                    AddressMode::ZeroPage => {
                        binary_instructions.push(0x2A);
                        binary_instructions.append(&mut address.address.to_bytes());
                    }
                    AddressMode::ZeroPageIndexed => {
                        binary_instructions.push(0x2B);
                        binary_instructions.append(&mut address.address.to_bytes());
                        assert!(address.offset.is_some());
                        binary_instructions.push(address.offset.unwrap().address)
                    }
                }
            }
            Instruction::CopyAccumulatorToRegister(register) => {
                binary_instructions.push(0x2C);
                binary_instructions.push(register.address);
            }
            Instruction::CopyRegisterToAccumulator(register) => {
                binary_instructions.push(0x2D);
                binary_instructions.push(register.address);
            }
            Instruction::BranchIfCarrySet(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x42);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x43);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x42);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::BranchIfCarryNotSet(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x44);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x45);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x44);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::BranchIfNegative(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x46);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x47);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x46);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::BranchIfPositive(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x48);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x49);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x48);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::BranchIfEqual(register, address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x4A);
                            binary_instructions.push(register.address);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x4B);
                            binary_instructions.push(register.address);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x4A);
                    binary_instructions.push(register.address);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::BranchIfNotEqual(register, address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x4C);
                            binary_instructions.push(register.address);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x4D);
                            binary_instructions.push(register.address);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x4C);
                    binary_instructions.push(register.address);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::BranchIfZero(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x4E);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x4F);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x4E);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::BranchIfNotZero(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x50);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x51);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x50);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::Jump(address, label) => {
                if let Some(address) = address {
                    match address.mode {
                        AddressMode::Absolute => {
                            binary_instructions.push(0x52);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        AddressMode::Indexed => {
                            binary_instructions.push(0x53);
                            binary_instructions.append(&mut address.address.to_bytes());
                        }
                        _ => unreachable!("NonZeroPageAddress had zero paged address mode!")
                    }
                } else {
                    assert!(label.is_some());
                    // absolute address mode for labels
                    binary_instructions.push(0x52);
                    label_usages.push(AssemblerLabelUse {
                        name: label.unwrap().name,
                        index: (binary_instructions.len() + 1) as u16
                    });
                    // allocate space for the address to be replaced
                    binary_instructions.push(0x00);
                    binary_instructions.push(0x00);
                }
            }
            Instruction::Label(name) => {
                labels.push(AssemblerLabel {
                    name,
                    index: binary_instructions.len() as u16
                });
            }
            Instruction::PushProgramCounter => {
                binary_instructions.push(0x54);
            }
            Instruction::PopProgramCounter => {
                binary_instructions.push(0x55);
            }
            Instruction::IncrementProgramCounter => {
                binary_instructions.push(0x56);
            }
            _ => eprintln!("Unimplemented instruction! ({instruction:?})")
        }
    }
    // make sure that even if there is a label at the end of the program, it won't crash
    // only sacrifices a byte of space, so it's worth it
    binary_instructions.push(0x00);

    // compute addresses of all labels and replace labels with addresses
    // pass 3 in assembling sequence
    for label_use in label_usages {
        let mut target_label = &AssemblerLabel {
            name: "".to_string(),
            index: 0
        };
        for label in labels.iter() {
            if label.name == label_use.name {
                target_label = label;
                break;
            }
        }
        assert_ne!(target_label.name, "".to_string());
        let label_address = target_label.index.to_be_bytes();
        binary_instructions[(label_use.index + 0) as usize] = label_address[1];
        binary_instructions[(label_use.index - 1) as usize] = label_address[0];
    }


    binary_instructions
}

pub fn write(binary: &Vec<u8>, directory: String, filename: &str) {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.clone() + filename);

    if file.is_err() {
        eprintln!("WARNING: Unable to create file \"{filename}\" (removing file, trying again)");
        std::fs::remove_file(directory.clone() + filename).expect("Failed to remove existing target file!");
        write(binary, directory, filename);
        return;
    }
    let mut file = file.unwrap();
    file.write_all(binary).expect(&format!("Unable to write to file \"{filename}\" in directory \"{directory}\"!"));
    file.sync_all().expect(&format!("Unable to write to file \"{filename}\" in directory \"{directory}\"!"));
    println!("Successfully wrote binary to file \"{filename}\"");
}