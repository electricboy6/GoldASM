use std::collections::HashSet;
use ratatui::crossterm::style::Stylize;

#[derive(Debug, PartialEq)]
pub struct Includes {
    files: HashSet<String>,
    instructions: Vec<Vec<Instruction>>,
}
impl Includes {
    pub fn new() -> Includes {
        Includes {
            files: HashSet::new(),
            instructions: Vec::new(),
        }
    }
    pub fn parse_include(&mut self, line: &str, directory: &str) {
        let target_file = line.strip_prefix("#include ").unwrap().trim();
        if self.files.insert(target_file.to_string()) {
            let parsed_file = parse(directory, target_file);
            // deal with this file
            let parsed_instructions = parsed_file.0;
            self.instructions.push(parsed_instructions);
            for file in parsed_file.1.files {
                self.files.insert(file);
            }
            // deal with the includes of this file (hopefully my logic is correct here)
            for instructions in parsed_file.1.instructions {
                self.instructions.push(instructions);
            }
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NumberType { 
    Binary,
    Hex
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum NumberSize {
    EightBit,
    SixteenBit,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum AddressMode {
    Absolute,
    Indexed,
    ZeroPage,
    ZeroPageIndexed
}

#[derive(Debug, PartialEq, Clone)]
pub struct Address {
    pub address: Number,
    pub index: Option<Register>,
    pub mode: AddressMode,
    pub pointer: Option<Pointer>,
}
impl Address {
    pub fn from_str(value: &str) -> Address {
        let address_value;
        let address;
        let mut offset = None;
        let mode;
        if value.contains('*') {
            let pointer_name = value.splitn(2, '*').nth(1).unwrap()
                .split_whitespace().nth(0).unwrap().trim_end_matches(',');
            let modified_value = value.replace(&("*".to_string() + pointer_name), "0000");
            let address_skeleton = Address::from_str(&modified_value);
            return Address {
                address: address_skeleton.address,
                index: address_skeleton.index,
                mode: address_skeleton.mode,
                pointer: Some(Pointer {
                    name: pointer_name.to_string(),
                    address: None,
                })
            }
        } else if value.contains('$') {
            // indexed
            address_value = value.split_whitespace().nth(0).unwrap()
                .strip_prefix('$').unwrap().strip_suffix(',').unwrap();
            address = Number::from_str(address_value);
            
            let offset_value = value.split_whitespace().nth(1).unwrap();
            offset = Some(Register::from_str(offset_value));
            if address.size == NumberSize::EightBit {
                mode = AddressMode::ZeroPageIndexed;
            } else {
                mode = AddressMode::Indexed;
            }
        } else if value.contains('%') {
            // absolute
            address_value = value.strip_prefix('%').unwrap();
            address = Number::from_str(address_value);
            if address.size == NumberSize::EightBit {
                mode = AddressMode::ZeroPage;
            } else {
                mode = AddressMode::Absolute;
            }
        } else {
            panic!("Attempted to parse an address with no mode signifier! ({value})");
        }
        Address {
            address,
            index: offset,
            mode,
            pointer: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NonZeroPageAddress {
    pub address: Number,
    pub index: Option<Register>,
    pub mode: AddressMode,
}
impl NonZeroPageAddress {
    pub fn from_str(value: &str) -> NonZeroPageAddress {
        let address = Address::from_str(value);
        match address.mode {
            AddressMode::Absolute => {
                NonZeroPageAddress {
                    address: address.address,
                    index: None,
                    mode: AddressMode::Absolute,
                }
            },
            AddressMode::Indexed => {
                NonZeroPageAddress {
                    address: address.address,
                    index: address.index,
                    mode: AddressMode::Indexed,
                }
            },
            _ => panic!("Attempted to parse a zero paged address as non zero paged! ({value})")
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Number {
    pub value: String,
    pub size: NumberSize,
    pub number_type: NumberType
}
impl Number {
    pub fn from_str(value: &str) -> Number {
        let number_type;
        let number_size;
        let final_value;
        if value.starts_with("^") {
            number_type = NumberType::Binary;
            if value.trim().len() == 9 { // there's still the carat to deal with
                number_size = NumberSize::EightBit;
            } else {
                number_size = NumberSize::SixteenBit;
            }
            final_value = value.strip_prefix('^').unwrap();
        } else {
            number_type = NumberType::Hex;
            if value.trim().len() == 2 {
                number_size = NumberSize::EightBit;
            } else {
                number_size = NumberSize::SixteenBit;
            }
            final_value = value;
        }
        Number {
            value: final_value.to_string(),
            size: number_size,
            number_type
        }
    }
    pub fn to_decimal(&self) -> u16 {
        let stripped_value = self.value.trim();
        match self.number_type {
            NumberType::Binary => {
                u16::from_str_radix(stripped_value, 2).unwrap()
            },
            NumberType::Hex => {
                u16::from_str_radix(stripped_value, 16).unwrap()
            }
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let decimal_value = self.to_decimal();
        // the isa is big endian, so we use big endian here
        //                                              |
        //                                              V
        let mut bytes = decimal_value.to_be_bytes().to_vec();
        if self.size == NumberSize::EightBit {
            bytes.remove(0);
        }
        bytes
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Register {
    pub address: u8,
}
impl Register {
    pub fn from_str(value: &str) -> Register {
        let target_register = Number::from_str(value);
        assert_eq!(target_register.size, NumberSize::EightBit);
        Register { address: target_register.to_decimal() as u8 }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct Immediate {
    pub value: Number,
}
impl Immediate {
    pub fn from_str(value: &str) -> Immediate {
        let number = Number::from_str(value.strip_prefix("#").unwrap());
        Immediate { value: number }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Label {
    pub name: String
}
#[derive(Debug, PartialEq, Clone)]
pub struct Subroutine {
    pub name: String
}
#[derive(Debug, PartialEq, Clone)]
pub struct Pointer {
    pub name: String,
    pub address: Option<PointerAddress>
}
// goofy hack to get around infinite size types
#[derive(Debug, PartialEq, Clone)]
pub struct PointerAddress {
    pub address: Number,
    pub mode: AddressMode,
    pub index: Option<Register>
}
impl PointerAddress {
    pub fn from_str(value: &str) -> PointerAddress {
        let address = Address::from_str(value);
        PointerAddress {
            address: address.address,
            mode: address.mode,
            index: address.index
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Noop,
    Add(Option<Register>, Option<(Register, Register)>),
    Subtract(Option<Register>, Option<(Register, Register)>),
    SetCarry,
    ClearCarry,
    Xor(Option<Register>, Option<(Register, Register)>),
    Xnor(Option<Register>, Option<(Register, Register)>),
    Or(Option<Register>, Option<(Register, Register)>),
    Nor(Option<Register>, Option<(Register, Register)>),
    And(Option<Register>, Option<(Register, Register)>),
    Nand(Option<Register>, Option<(Register, Register)>),
    Not,
    RotateRight,
    RotateLeft,
    ShiftRight,
    ShiftLeft,
    PushRegister(Register),
    PopRegister(Register),
    LoadAccumulator(Option<Address>, Option<Immediate>),
    StoreAccumulator(Address),
    CopyAccumulatorToRegister(Register),
    CopyRegisterToAccumulator(Register),
    BranchIfCarrySet(Option<NonZeroPageAddress>, Option<Label>),
    BranchIfCarryNotSet(Option<NonZeroPageAddress>, Option<Label>),
    BranchIfNegative(Option<NonZeroPageAddress>, Option<Label>),
    BranchIfPositive(Option<NonZeroPageAddress>, Option<Label>),
    BranchIfEqual(Register, Option<NonZeroPageAddress>, Option<Label>),
    BranchIfNotEqual(Register, Option<NonZeroPageAddress>, Option<Label>),
    BranchIfZero(Option<NonZeroPageAddress>, Option<Label>),
    BranchIfNotZero(Option<NonZeroPageAddress>, Option<Label>),
    BranchIfGreater(Register, Option<NonZeroPageAddress>, Option<Label>),
    BranchIfLess(Register, Option<NonZeroPageAddress>, Option<Label>),
    Jump(Option<NonZeroPageAddress>, Option<Label>),
    JumpSubroutine(Option<NonZeroPageAddress>, Option<Subroutine>),
    ReturnFromSubroutine(Subroutine),
    Label(String),
    Subroutine(String),
    PushProgramCounter,
    PopProgramCounter,
    Pointer(String, PointerAddress),
    SetOrigin(Option<Address>),
    Word(Immediate),
    PopProgramCounterSubroutine,
}

pub fn postprocess(instructions: Vec<Instruction>, includes: Includes) -> Vec<Instruction> {
    let mut final_instructions = instructions;
    let mut included_instructions = includes.instructions;
    
    for included_instructions in included_instructions.iter_mut() {
        final_instructions.append(included_instructions);
    }
    
    final_instructions
}

pub fn parse(directory: &str, filename: &str) -> (Vec<Instruction>, Includes) {
    println!("INFO: Parsing file {}", directory.to_string() + filename);
    let module_name_dot = &*(filename.strip_suffix(".gasm").unwrap().to_string() + ".");
    
    let content = std::fs::read_to_string(directory.to_string() + filename).expect("File not found.");
    
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut includes = Includes::new();
    
    for raw_line in content.lines() {
        // strip out leading and trailing whitespace, as well as comments
        let line = raw_line.splitn(2, "//").nth(0).unwrap_or("").trim();
        if line == "" {
            continue;
        }
        
        if line.contains("#include") {
            includes.parse_include(line, directory);
            continue;
        }
        
        // label/subroutine logic
        if line.contains(':') {
            if line.contains("sr") {
                // this is a subroutine
                let target_name = module_name_dot.to_string() + line.strip_suffix(':')
                    .unwrap().strip_prefix("sr").unwrap().trim();
                
                if instructions.contains(&Instruction::Label(target_name.clone())) {
                    panic!("Subroutine already exists as a label! ({target_name})");
                }
                if instructions.contains(&Instruction::Subroutine(target_name.clone())) {
                    panic!("Subroutine already exists! ({target_name})");
                }
                
                instructions.push(Instruction::Subroutine(
                    target_name
                ));
                continue;
            }
            // this is a label
            let target_name = module_name_dot.to_string() + line.strip_suffix(':').unwrap();
            
            if instructions.contains(&Instruction::Label(target_name.clone())) {
                panic!("Label already exists! ({target_name})");
            }
            if instructions.contains(&Instruction::Subroutine(target_name.clone())) {
                panic!("Label already exists as a subroutine! ({target_name})");
            }
            
            instructions.push(Instruction::Label(
                target_name
            ));
            continue;
        }
        
        // line split by whitespace
        let words =  line.split_whitespace().collect::<Vec<&str>>();
        // rest of line after the instruction is interpreted
        // defaults to empty strings so instructions with no parameters won't panic
        let parameter_str = line.split_once(" ").unwrap_or(("", "")).1;

        // pointer definition logic
        if line.contains("#define") {
            // pointer creation
            instructions.push(Instruction::Pointer( words[1].to_string(), PointerAddress::from_str(words[2])));
            continue;
        }

        // origin logic
        // if no origin address set, continue at lowest unused address above the first .org
        if line.contains(".org") {
            if words.len() == 2 {
                let address = Address::from_str(words[1]);
                assert_eq!(address.mode, AddressMode::Absolute);
                instructions.push(Instruction::SetOrigin(Some(address)));
            } else {
                instructions.push(Instruction::SetOrigin(None));
            }
            continue;
        }

        // word logic
        if line.contains(".word") {
            instructions.push(Instruction::Word(Immediate::from_str(words[1])));
            continue;
        }
        
        // normal instruction
        match words[0].trim().to_lowercase().as_str() {
            "noop" => instructions.push(Instruction::Noop),
            "add" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::Add(parameters.0, parameters.1));
            },
            "sub" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::Subtract(parameters.0, parameters.1));
            },
            "sc" => instructions.push(Instruction::SetCarry),
            "clc" => instructions.push(Instruction::ClearCarry),
            "xor" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::Xor(parameters.0, parameters.1));
            },
            "xnor" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::Xnor(parameters.0, parameters.1));
            },
            "or" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::Or(parameters.0, parameters.1));
            },
            "nor" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::Nor(parameters.0, parameters.1));
            },
            "and" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::And(parameters.0, parameters.1));
            },
            "nand" => {
                let parameters = parse_register_or_2_register_instruction(words);
                instructions.push(Instruction::Nand(parameters.0, parameters.1));
            },
            "not" => instructions.push(Instruction::Not),
            "ror" => instructions.push(Instruction::RotateRight),
            "rol" => instructions.push(Instruction::RotateLeft),
            "shr" => instructions.push(Instruction::ShiftRight),
            "shl" => instructions.push(Instruction::ShiftLeft),
            "phr" => {
                instructions.push(Instruction::PushRegister(
                    Register::from_str(parameter_str)
                ));
            },
            "plr" => {
                instructions.push(Instruction::PopRegister(
                    Register::from_str(parameter_str)
                ));
            },
            "lda" => {
                if parameter_str.contains("#") {
                    instructions.push(Instruction::LoadAccumulator(
                        None, Some(Immediate::from_str(parameter_str))
                    ))
                } else {
                    instructions.push(
                        Instruction::LoadAccumulator(Some(Address::from_str(parameter_str)), None)
                    )
                }
            },
            "sta" => {
                instructions.push(
                    Instruction::StoreAccumulator(Address::from_str(parameter_str))
                )
            },
            "cpa" => {
                instructions.push(Instruction::CopyAccumulatorToRegister(
                    Register::from_str(parameter_str)
                ));
            },
            "cpr" => {
                instructions.push(Instruction::CopyRegisterToAccumulator(
                    Register::from_str(parameter_str)
                ));
            },
            "bcs" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfCarrySet(
                            None, Some(Label {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfCarrySet(
                            None, Some(Label {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfCarrySet(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "bcc" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfCarryNotSet(
                            None, Some(Label {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfCarryNotSet(
                            None, Some(Label {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfCarryNotSet(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "bn" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfNegative(
                            None, Some(Label {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfNegative(
                            None, Some(Label {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfNegative(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "bp" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfPositive(
                            None, Some(Label {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfPositive(
                            None, Some(Label {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfPositive(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "beq" => {
                let register = Register::from_str(words[1].strip_suffix(',').unwrap());
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfEqual(
                            register,
                            None, Some(Label {
                                name: words[2].strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfEqual(
                            register,
                            None, Some(Label {
                                name: module_name_dot.to_string() + words[2].strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfEqual(
                        register,
                        Some(NonZeroPageAddress::from_str(words[2])), None
                    ));
                }
            },
            "bne" => {
                let register = Register::from_str(words[1].strip_suffix(',').unwrap());
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfNotEqual(
                            register,
                            None, Some(Label {
                                name: words[2].strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfNotEqual(
                            register,
                            None, Some(Label {
                                name: module_name_dot.to_string() + words[2].strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfNotEqual(
                        register,
                        Some(NonZeroPageAddress::from_str(words[2])), None
                    ));
                }
            },
            "bze" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfZero(
                            None, Some(Label {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfZero(
                            None, Some(Label {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfZero(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "bnz" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfNotZero(
                            None, Some(Label {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfNotZero(
                            None, Some(Label {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfNotZero(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "bg" => {
                let register = Register::from_str(words[1].strip_suffix(',').unwrap());
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfGreater(
                            register,
                            None, Some(Label {
                                name: words[2].strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfGreater(
                            register,
                            None, Some(Label {
                                name: module_name_dot.to_string() + words[2].strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfGreater(
                        register,
                        Some(NonZeroPageAddress::from_str(words[2])), None
                    ));
                }
            },
            "bl" => {
                let register = Register::from_str(words[1].strip_suffix(',').unwrap());
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::BranchIfLess(
                            register,
                            None, Some(Label {
                                name: words[2].strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::BranchIfLess(
                            register,
                            None, Some(Label {
                                name: module_name_dot.to_string() + words[2].strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::BranchIfLess(
                        register,
                        Some(NonZeroPageAddress::from_str(words[2])), None
                    ));
                }
            },
            "jmp" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::Jump(
                            None, Some(Label {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::Jump(
                            None, Some(Label {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::Jump(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "jsr" => {
                if parameter_str.contains('~') {
                    // using a name
                    if parameter_str.contains('.') {
                        // in another file, don't add our filename
                        instructions.push(Instruction::JumpSubroutine(
                            None, Some(Subroutine {
                                name: parameter_str.strip_prefix('~').unwrap().to_string(),
                            })
                        ));
                    } else {
                        // in our file, add our filename
                        instructions.push(Instruction::JumpSubroutine(
                            None, Some(Subroutine {
                                name: module_name_dot.to_string() + parameter_str.strip_prefix('~').unwrap()
                            })
                        ));
                    }
                } else {
                    instructions.push(Instruction::JumpSubroutine(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "rts" => {
                let label = words[1].trim();
                instructions.push(Instruction::ReturnFromSubroutine(
                    Subroutine { name: module_name_dot.to_string() + label + "_EndSubroutine" }
                ));
            },
            "phpc" => instructions.push(Instruction::PushProgramCounter),
            "plpc" => instructions.push(Instruction::PopProgramCounter),
            "//" => continue,
            "" => continue,
            _ => {
                eprintln!("{}", format!("WARNING: While parsing \"{}\", got line \"{line}\", which is not an instruction!", directory.to_string() + filename).yellow());
            }
        }
    }
    (instructions, includes)
}

fn parse_register_or_2_register_instruction(words: Vec<&str>) -> (Option<Register>, Option<(Register, Register)>) {
    if words.len() == 2 {
        (Some(Register::from_str(words[1])), None)
    } else {
        (None, Some((
            Register::from_str(words[1].strip_suffix(',').unwrap()),
            Register::from_str(words[2])
        )))
    }
}