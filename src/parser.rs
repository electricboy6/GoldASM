#[derive(Debug, PartialEq)]
pub enum NumberType { 
    Binary,
    Hex
}

#[derive(Debug, PartialEq)]
pub enum NumberSize {
    EightBit,
    SixteenBit,
}

#[derive(Debug, PartialEq)]
pub enum AddressMode {
    Absolute,
    Indexed,
    ZeroPage,
    ZeroPageIndexed
}

#[derive(Debug, PartialEq)]
pub struct Address {
    pub address: Number,
    pub offset: Option<Number>,
    pub mode: AddressMode,
}
impl Address {
    pub fn from_str(value: &str) -> Address {
        let address_value;
        let address;
        let mut offset = None;
        let mode;
        if value.contains("$") {
            // indexed
            address_value = value.split_whitespace().nth(0).unwrap()
                .strip_prefix("$").unwrap().strip_suffix(",").unwrap();
            address = Number::from_str(address_value);
            
            let offset_value = value.split_whitespace().nth(1).unwrap();
            offset = Some(Number::from_str(offset_value));
            if address.size == NumberSize::EightBit {
                mode = AddressMode::ZeroPageIndexed;
            } else {
                mode = AddressMode::Indexed;
            }
        } else if value.contains("%") {
            // absolute
            address_value = value.strip_prefix("%").unwrap();
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
            offset,
            mode,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct NonZeroPageAddress {
    pub address: Number,
    pub offset: Option<Number>,
    pub mode: AddressMode,
}
impl NonZeroPageAddress {
    pub fn from_str(value: &str) -> NonZeroPageAddress {
        let address = Address::from_str(value);
        match address.mode {
            AddressMode::Absolute => {
                NonZeroPageAddress {
                    address: address.address,
                    offset: None,
                    mode: AddressMode::Absolute,
                }
            },
            AddressMode::Indexed => {
                NonZeroPageAddress {
                    address: address.address,
                    offset: address.offset,
                    mode: AddressMode::Indexed,
                }
            },
            _ => panic!("Attempted to parse a zero paged address as non zero paged! ({value})")
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Number {
    pub value: String,
    pub size: NumberSize,
    pub number_type: NumberType
}
impl Number {
    pub fn from_str(value: &str) -> Number {
        let number_type;
        let number_size;
        if value.starts_with("^") {
            number_type = NumberType::Binary;
            if value.trim().len() == 9 { // there's still the carat to deal with
                number_size = NumberSize::EightBit;
            } else {
                number_size = NumberSize::SixteenBit;
            }
        } else {
            number_type = NumberType::Hex;
            if value.trim().len() == 2 {
                number_size = NumberSize::EightBit;
            } else {
                number_size = NumberSize::SixteenBit;
            }
        }
        Number {
            value: value.to_string(),
            size: number_size,
            number_type
        }
    }
    pub fn to_decimal(&self) -> i16 {
        let mut final_num = 0;
        match self.number_type {
            NumberType::Binary => {
                let stripped_value = self.value.trim().strip_prefix("^").unwrap();
                for (index, character) in stripped_value.chars().enumerate() {
                    final_num += character.to_digit(2).unwrap() * index as u32;
                }
            },
            NumberType::Hex => {
                let stripped_value = self.value.trim();
                for (index, character) in stripped_value.chars().enumerate() {
                    final_num += character.to_digit(16).unwrap() * index as u32;
                }
            }
        }
        final_num as i16
    }
}

#[derive(Debug, PartialEq)]
struct Register {
    pub address: u8,
}
impl Register {
    pub fn from_str(value: &str) -> Register {
        Register { address: value.parse::<u8>().unwrap() }
    }
}
#[derive(Debug, PartialEq)]
struct Immediate {
    pub value: Number,
}
impl Immediate {
    pub fn from_str(value: &str) -> Immediate {
        let number = Number::from_str(value.strip_prefix("#").unwrap());
        Immediate { value: number }
    }
}

#[derive(Debug, PartialEq)]
struct Label {
    pub name: String
}
#[derive(Debug, PartialEq)]
struct Subroutine {
    pub name: String
}

#[derive(Debug, PartialEq)]
enum Instruction {
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
    BranchIfCarrySet(NonZeroPageAddress),
    BranchIfCarryNotSet(NonZeroPageAddress),
    BranchIfNegative(NonZeroPageAddress),
    BranchIfPositive(NonZeroPageAddress),
    BranchIfEqual(Register, NonZeroPageAddress),
    BranchIfNotEqual(Register, NonZeroPageAddress),
    BranchIfZero(NonZeroPageAddress),
    BranchIfNotZero(NonZeroPageAddress),
    Jump(Option<NonZeroPageAddress>, Option<Label>), // The instruction MUST be a label instruction
    JumpSubroutine(Option<NonZeroPageAddress>, Option<Subroutine>),
    ReturnFromSubroutine,
    Label(String),
    Subroutine(String),
}

pub fn parse(path: &str) {
    println!("Parsing file {}", path);
    let content = std::fs::read_to_string(path).expect("File not found.");
    let mut instructions: Vec<Instruction> = Vec::new();
    for raw_line in content.lines() {
        // strip out leading and trailing whitespace, as well as comments
        let line = raw_line.splitn(2, '/').nth(0).unwrap_or("").trim();
        if line == "" {
            continue;
        }
        
        // label/subroutine logic
        if line.contains(':') {
            if line.contains("sr") {
                // this is a subroutine
                instructions.push(Instruction::Subroutine(
                    line.strip_suffix(':').unwrap().strip_prefix("sr").unwrap().trim()
                        .to_string()
                ));
                continue;
            }
            // this is a label
            instructions.push(Instruction::Label(
                line.strip_suffix(':').unwrap().to_string()
            ));
            continue;
        }
        
        // line split by whitespace
        let words =  line.split_whitespace().collect::<Vec<&str>>();
        // rest of line after the instruction is interpreted
        // defaults to empty strings so instructions with no parameters won't panic
        let parameter_str = line.split_once(" ").unwrap_or(("", "")).1;
        
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
                instructions.push(Instruction::BranchIfCarrySet(
                    NonZeroPageAddress::from_str(parameter_str)
                ));
            },
            "bcc" => {
                instructions.push(Instruction::BranchIfCarryNotSet(
                    NonZeroPageAddress::from_str(parameter_str)
                ));
            },
            "bn" => {
                instructions.push(Instruction::BranchIfNegative(
                    NonZeroPageAddress::from_str(parameter_str)
                ));
            },
            "bp" => {
                instructions.push(Instruction::BranchIfPositive(
                    NonZeroPageAddress::from_str(parameter_str)
                ));
            },
            "beq" => {
                instructions.push(Instruction::BranchIfEqual(
                    Register::from_str(words[1].strip_suffix(',').unwrap()),
                    NonZeroPageAddress::from_str(parameter_str.split_once(' ').unwrap().1)
                ));
            },
            "bne" => {
                instructions.push(Instruction::BranchIfNotEqual(
                    Register::from_str(words[1].strip_suffix(',').unwrap()),
                    NonZeroPageAddress::from_str(parameter_str.split_once(' ').unwrap().1)
                ));
            },
            "bze" => {
                instructions.push(Instruction::BranchIfZero(
                    NonZeroPageAddress::from_str(parameter_str)
                ));
            },
            "bnz" => {
                instructions.push(Instruction::BranchIfNotZero(
                    NonZeroPageAddress::from_str(parameter_str)
                ));
            },
            "jmp" => {
                if parameter_str.contains('~') {
                    instructions.push(Instruction::Jump(
                        None, Some(Label {
                            name: parameter_str.strip_prefix('~').unwrap().to_string()
                        })
                    ));
                } else {
                    instructions.push(Instruction::Jump(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "jsr" => {
                if parameter_str.contains('~') {
                    instructions.push(Instruction::JumpSubroutine(
                        None, Some(Subroutine {
                            name: parameter_str.strip_prefix('~').unwrap().to_string()
                        })
                    ));
                } else {
                    instructions.push(Instruction::JumpSubroutine(
                        Some(NonZeroPageAddress::from_str(parameter_str)), None
                    ));
                }
            },
            "rts" => {
                instructions.push(Instruction::ReturnFromSubroutine);
            },
            "//" => continue,
            "" => continue,
            _ => {
                println!("Not an instruction");
            }
        }
    }
    println!("Instructions: {instructions:?}");
}

fn parse_register_or_2_register_instruction(words: Vec<&str>) -> (Option<Register>, Option<(Register, Register)>) {
    if words.len() == 2 {
        (Some(Register::from_str(words[1])), None)
    } else {
        (None, Some((
            Register::from_str(words[1]),
            Register::from_str(words[2])
        )))
    }
}