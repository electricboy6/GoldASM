use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub struct Address {
    pub address: u16,
    pub index: Option<u8>
}
impl Address {
    pub fn new_absolute(big_part: u8, small_part: u8) -> Address {
        let address_value = ((big_part as u16) << 8) | (small_part as u16);
        Address {
            address: address_value,
            index: None
        }
    }
    pub fn new_indexed(big_part: u8, small_part: u8, index: u8) -> Address {
        let address_value = ((big_part as u16) << 8) | (small_part as u16);
        Address {
            address: address_value,
            index: Some(index)
        }
    }
    pub fn new_zeropage(address: u8) -> Address {
        Address {
            address: address as u16,
            index: None
        }
    }
    pub fn new_zeropage_indexed(address: u8, index: u8) -> Address {
        Address {
            address: address as u16,
            index: Some(index)
        }
    }
}
impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.address <= 0x00ff {
            // zero page
            if let Some(index) = self.index {
                // zero page indexed
                write!(f, "${:02x}, {:02x}", self.address, index)
            } else {
                // zero page
                write!(f, "%{:02x}", self.address)
            }
        } else {
            // not zero page
            if let Some(index) = self.index {
                // indexed
                write!(f, "${:04x}, {:02x}", self.address, index)
            } else {
                // absolute
                write!(f, "%{:04x}", self.address)
            }
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Noop,
    Add(Option<u8>, Option<(u8, u8)>),
    Subtract(Option<u8>, Option<(u8, u8)>),
    SetCarry,
    ClearCarry,
    Xor(Option<u8>, Option<(u8, u8)>),
    Xnor(Option<u8>, Option<(u8, u8)>),
    Or(Option<u8>, Option<(u8, u8)>),
    Nor(Option<u8>, Option<(u8, u8)>),
    And(Option<u8>, Option<(u8, u8)>),
    Nand(Option<u8>, Option<(u8, u8)>),
    Not,
    RotateRight,
    RotateLeft,
    ShiftRight,
    ShiftLeft,
    PushRegisterToStack(u8),
    PopRegisterFromStack(u8),
    LoadAccumulator(Option<Address>, Option<u8>),
    StoreAccumulator(Address),
    CopyAccumulatorToRegister(u8),
    CopyRegisterToAccumulator(u8),
    BranchCarrySet(Address),
    BranchCarryClear(Address),
    BranchNegative(Address),
    BranchPositive(Address),
    BranchEqual(u8, Address),
    BranchNotEqual(u8, Address),
    BranchZero(Address),
    BranchNotZero(Address),
    BranchGreater(u8, Address),
    BranchLess(u8, Address),
    Jump(Address),
    PushProgramCounter,
    PopProgramCounter,
    IncrementProgramCounter,
    PopProgramCounterSubroutine,
}

/// returns the instruction, its parameters, and the number of additional bytes to skip (we automatically increment the program counter, so this is the number of bytes of parameters)
pub fn parse_instruction(memory: &[u8; 65536], program_counter: u16) -> Result<(Instruction, u8), Box<dyn std::error::Error>> {
    let program_counter = program_counter as usize;
    match memory[program_counter] {
        0x00 => Ok((Instruction::Noop, 0)),
        0x01 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::Add(Some(parameter), None), 1))
        }
        0x02 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Add(None, Some((parameter1, parameter2))), 2))
        }
        0x03 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::Subtract(Some(parameter), None), 1))
        }
        0x04 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Subtract(None, Some((parameter1, parameter2))), 2))
        }
        0x05 => Ok((Instruction::SetCarry, 0)),
        0x06 => Ok((Instruction::ClearCarry, 0)),
        0x07 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::Xor(Some(parameter), None), 1))
        }
        0x08 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Xor(None, Some((parameter1, parameter2))), 2))
        }
        0x09 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::Xnor(Some(parameter), None), 1))
        }
        0x0A => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Xnor(None, Some((parameter1, parameter2))), 2))
        }
        0x0B => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::Or(Some(parameter), None), 1))
        }
        0x0C => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Or(None, Some((parameter1, parameter2))), 2))
        }
        0x0D => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::Nor(Some(parameter), None), 1))
        }
        0x0E => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Nor(None, Some((parameter1, parameter2))), 2))
        }
        0x0F => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::And(Some(parameter), None), 1))
        }
        0x10 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::And(None, Some((parameter1, parameter2))), 2))
        }
        0x11 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::Nand(Some(parameter), None), 1))
        }
        0x12 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Nand(None, Some((parameter1, parameter2))), 2))
        }
        0x13 => Ok((Instruction::Not, 0)),
        0x14 => Ok((Instruction::RotateRight, 0)),
        0x15 => Ok((Instruction::RotateLeft, 0)),
        0x16 => Ok((Instruction::ShiftRight, 0)),
        0x17 => Ok((Instruction::ShiftLeft, 0)),
        0x21 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::PushRegisterToStack(parameter), 1))
        }
        0x22 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::PopRegisterFromStack(parameter), 1))
        }
        0x23 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::LoadAccumulator(Some(Address::new_absolute(parameter1, parameter2)), None), 2))
        }
        0x24 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::LoadAccumulator(Some(Address::new_indexed(parameter1, parameter2, parameter3)), None), 3))
        }
        0x25 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::LoadAccumulator(Some(Address::new_zeropage(parameter)), None), 1))
        }
        0x26 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::LoadAccumulator(Some(Address::new_zeropage_indexed(parameter1, parameter2)), None), 2))
        }
        0x27 => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::LoadAccumulator(None, Some(parameter)), 1))
        }
        0x28 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::StoreAccumulator(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x29 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::StoreAccumulator(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x2A => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::StoreAccumulator(Address::new_zeropage(parameter)), 1))
        }
        0x2B => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::StoreAccumulator(Address::new_zeropage_indexed(parameter1, parameter2)), 2))
        }
        0x2C => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::CopyAccumulatorToRegister(parameter), 1))
        }
        0x2D => {
            let parameter = memory[program_counter + 1];
            Ok((Instruction::CopyRegisterToAccumulator(parameter), 1))
        }
        0x42 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::BranchCarrySet(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x43 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchCarrySet(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x44 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::BranchCarryClear(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x45 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchCarryClear(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x46 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::BranchNegative(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x47 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchNegative(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x48 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::BranchPositive(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x49 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchPositive(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x4A => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchEqual(parameter1, Address::new_absolute(parameter2, parameter3)), 3))
        }
        0x4B => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            let parameter4 = memory[program_counter + 4];
            Ok((Instruction::BranchEqual(parameter1, Address::new_indexed(parameter2, parameter3, parameter4)), 3))
        }
        0x4C => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchNotEqual(parameter1, Address::new_absolute(parameter2, parameter3)), 3))
        }
        0x4D => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            let parameter4 = memory[program_counter + 4];
            Ok((Instruction::BranchNotEqual(parameter1, Address::new_indexed(parameter2, parameter3, parameter4)), 3))
        }
        0x4E => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::BranchZero(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x4F => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchZero(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x50 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::BranchNotZero(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x51 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchNotZero(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x52 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            Ok((Instruction::Jump(Address::new_absolute(parameter1, parameter2)), 2))
        }
        0x53 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::Jump(Address::new_indexed(parameter1, parameter2, parameter3)), 3))
        }
        0x54 => Ok((Instruction::PushProgramCounter, 0)),
        0x55 => Ok((Instruction::PopProgramCounter, 0)),
        //0x56 => (Instruction::IncrementProgramCounter, 0),
        0x57 => Ok((Instruction::PopProgramCounterSubroutine, 0)),
        0x58 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchGreater(parameter1, Address::new_absolute(parameter2, parameter3)), 3))
        }
        0x59 => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            let parameter4 = memory[program_counter + 4];
            Ok((Instruction::BranchGreater(parameter1, Address::new_indexed(parameter2, parameter3, parameter4)), 3))
        }
        0x5A => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            Ok((Instruction::BranchLess(parameter1, Address::new_absolute(parameter2, parameter3)), 3))
        }
        0x5B => {
            let parameter1 = memory[program_counter + 1];
            let parameter2 = memory[program_counter + 2];
            let parameter3 = memory[program_counter + 3];
            let parameter4 = memory[program_counter + 4];
            Ok((Instruction::BranchLess(parameter1, Address::new_indexed(parameter2, parameter3, parameter4)), 3))
        }
        _ => {
            Err(Box::from("parse_error"))
            //panic!("Found invalid byte while parsing at index {program_counter}! ({:02x?})\n This means that we're probably off by some value, so don't trust the results.", &memory[program_counter-2..program_counter+2]);
        }
    }
}