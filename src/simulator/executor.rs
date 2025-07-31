use crate::simulator::bin_parser;
use crate::simulator::bin_parser::{Address, Instruction};

fn calculate_address(address: Address, cpu: &Processor) -> u16 {
    let real_address;
    if let Some(index) = address.index {
        real_address = address.address.wrapping_add(cpu.registers[index as usize] as u16);
    } else {
        real_address = address.address;
    }
    real_address
}
#[derive(Debug)]
pub struct Processor {
    pub accumulator: u8,
    pub registers: [u8; 8],
    pub status_register: u8,
    pub stack_pointer: u8,
    pub memory: [u8; 65536],
    pub program_counter: u16,
    pub operand1: u8,
    pub operand2: u8,
}
impl Default for Processor {
    fn default() -> Self {
        Processor::new()
    }
}
impl Processor {
    pub fn new() -> Processor {
        let mut cpu = Processor {
            accumulator: 0,
            registers: [0; 8],
            status_register: 0b010000_00, // accumulator starts at zero, so zero flag is 1
            stack_pointer: 0x00,
            memory: [0; 65536],
            program_counter: 0x0000,
            operand1: 0x00,
            operand2: 0x00,
        };
        cpu.reset();
        cpu
    }
    pub fn reset(&mut self) {
        self.accumulator = 0;
        self.registers = [0; 8];
        self.status_register = 0b010000_00;
        self.stack_pointer = 0x00;
        let high_byte = self.memory[0xFFFC];
        let low_byte = self.memory[0xFFFD];
        self.program_counter = ((high_byte as u16) << 8) | (low_byte as u16);
    }
    pub fn step(&mut self) {
        let (instruction, instruction_extra_bytes) = bin_parser::parse_instruction(&self.memory, self.program_counter);
        match instruction {
            Instruction::Add(one_register, two_register) => {
                if let Some(register) = one_register {
                    let current_carry = (self.status_register & 0b100000_00).min(1) as u16;
                    let register_value = self.registers[register as usize] as u16;
                    let accumulator_value = self.accumulator as u16;
                    let addition_result = accumulator_value + register_value + current_carry;
                    let carry = addition_result & 0b0000_0001_0000_0000;
                    self.accumulator = (addition_result & 0b0000_0000_1111_1111) as u8;
                    if carry > 0 {
                        self.status_register |= 0b100000_00
                    } else {
                        self.status_register &= 0b011111_00
                    }
                    self.update_status_two_operands(self.accumulator, register_value as u8);
                } else if let Some((register1, register2)) = two_register {
                    let current_carry = (self.status_register & 0b100000_00).min(1) as u16;
                    let register1_value = self.registers[register1 as usize] as u16;
                    let register2_value = self.registers[register2 as usize] as u16;
                    let addition_result = register1_value + register2_value + current_carry;
                    let carry = addition_result & 0b0000_0001_0000_0000;
                    self.accumulator = (addition_result & 0b0000_0000_1111_1111) as u8;
                    if carry > 0 {
                        self.status_register |= 0b100000_00
                    } else {
                        self.status_register &= 0b011111_00
                    }
                    self.update_status_two_operands(register1_value as u8, register2_value as u8);
                }
            }
            Instruction::Subtract(one_register, two_register) => {
                let current_carry = (self.status_register & 0b100000_00).min(1) as u16;
                if let Some(register) = one_register {
                    let acc_value = self.accumulator as u16 | (current_carry << 9);
                    let (result, carry_out) = acc_value
                        .overflowing_sub(self.registers[register as usize] as u16);
                    self.accumulator = result as u8;
                    
                    if carry_out {
                        self.status_register |= 0b100000_00
                    } else {
                        self.status_register &= 0b011111_00
                    }

                    self.update_status_two_operands(self.accumulator, self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    let register1_value = self.registers[register1 as usize] as u16 | (current_carry << 9);
                    let register2_value = self.registers[register2 as usize];
                    
                    let (result, carry_out) = register1_value.overflowing_sub(register2_value as u16);
                    self.accumulator = result as u8;
                    
                    if carry_out {
                        self.status_register |= 0b100000_00
                    } else {
                        self.status_register &= 0b011111_00
                    }
                    
                    self.update_status_two_operands((register1_value & 0b0000_0000_1111_1111) as u8, register2_value);
                }
            }
            Instruction::Xor(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator ^= self.registers[register as usize];
                    self.update_status_two_operands(self.accumulator, self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = self.registers[register1 as usize] ^ self.registers[register2 as usize];
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Xnor(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = !(self.accumulator ^ self.registers[register as usize]);
                    self.update_status_two_operands(self.accumulator, self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = !(self.registers[register1 as usize] ^ self.registers[register2 as usize]);
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::And(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator &= self.registers[register as usize];
                    self.update_status_two_operands(self.accumulator, self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = self.registers[register1 as usize] & self.registers[register2 as usize];
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Nand(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = !(self.accumulator & self.registers[register as usize]);
                    self.update_status_two_operands(self.accumulator, self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = !(self.registers[register1 as usize] & self.registers[register2 as usize]);
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Or(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator |= self.registers[register as usize];
                    self.update_status_two_operands(self.accumulator, self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = self.registers[register1 as usize] | self.registers[register2 as usize];
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Nor(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = !(self.accumulator | self.registers[register as usize]);
                    self.update_status_two_operands(self.accumulator, self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = !(self.registers[register1 as usize] | self.registers[register2 as usize]);
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Not => {
                self.accumulator = !self.accumulator;
                self.update_status_no_operands();
            }
            Instruction::RotateRight => {
                self.accumulator = self.accumulator.rotate_right(1);
                self.update_status_no_operands();
            }
            Instruction::RotateLeft => {
                self.accumulator = self.accumulator.rotate_left(1);
                self.update_status_no_operands();
            }
            Instruction::ShiftRight => {
                let carry_out = self.accumulator & 0b0000_0001;
                let carry_in = self.status_register & 0b100000_00;
                
                self.accumulator >>= 1;
                if carry_in > 1 {
                    self.accumulator |= 0b1000_0000;
                }
                
                if carry_out > 0 {
                    self.status_register |= 0b100000_00;
                } else {
                    self.status_register &= 0b011111_00;
                }
                
                self.update_status_no_operands();
            }
            Instruction::ShiftLeft => {
                let carry_in = self.status_register & 0b100000_00;
                let shifted_result = (self.accumulator as u16) << 1;
                let carry_out = shifted_result & 0b0000_0001_0000_0000;
                
                self.accumulator = (shifted_result & 0b0000_0000_1111_1111) as u8;
                if carry_in > 0 {
                    self.accumulator |= 0b0000_0001;
                }
                
                if carry_out > 0 {
                    self.status_register |= 0b100000_00;
                } else {
                    self.status_register &= 0b011111_00;
                }
                
                self.update_status_no_operands();
            }
            Instruction::Noop => {}
            Instruction::SetCarry => {
                self.status_register |= 0b100000_00
            }
            Instruction::ClearCarry => {
                self.status_register &= 0b011111_00
            }
            Instruction::PushRegisterToStack(register) => {
                self.push_stack(self.registers[register as usize]);
            }
            Instruction::PopRegisterFromStack(register) => {
                self.registers[register as usize] = self.pop_stack();
            }
            Instruction::LoadAccumulator(address, immediate) => {
                if let Some(address) = address {
                    self.accumulator = self.memory[calculate_address(address, self) as usize];
                } else if let Some(immediate) = immediate {
                    self.accumulator = immediate;
                }
            }
            Instruction::StoreAccumulator(address) => {
                self.memory[calculate_address(address, self) as usize] = self.accumulator;
            }
            Instruction::CopyAccumulatorToRegister(register) => {
                self.registers[register as usize] = self.accumulator;
            }
            Instruction::CopyRegisterToAccumulator(register) => {
                self.accumulator = self.registers[register as usize];
            }
            Instruction::BranchCarrySet(address) => {
                if self.status_register & 0b100000_00 > 1 {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchCarryClear(address) => {
                if self.status_register & 0b100000_00 == 0 {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchNegative(address) => {
                if self.status_register & 0b000001_00 > 1 {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchPositive(address) => {
                if self.status_register & 0b000001_00 == 0 {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchEqual(register, address) => {
                let operand1 = self.accumulator;
                let operand2 = self.registers[register as usize];
                self.update_status_two_operands(operand1, operand2);
                if self.registers[register as usize] == self.accumulator {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchNotEqual(register, address) => {
                let operand1 = self.accumulator;
                let operand2 = self.registers[register as usize];
                self.update_status_two_operands(operand1, operand2);
                if self.registers[register as usize] != self.accumulator {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchZero(address) => {
                if self.status_register & 0b010000_00 > 1 {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchNotZero(address) => {
                if self.status_register & 0b010000_00 == 0 {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchGreater(register, address) => {
                let operand1 = self.accumulator;
                let operand2 = self.registers[register as usize];
                self.update_status_two_operands(operand1, operand2);
                if self.registers[register as usize] > self.accumulator {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::BranchLess(register, address) => {
                let operand1 = self.accumulator;
                let operand2 = self.registers[register as usize];
                self.update_status_two_operands(operand1, operand2);
                if self.registers[register as usize] < self.accumulator {
                    self.program_counter = calculate_address(address, self);
                    return;
                }
            }
            Instruction::Jump(address) => {
                self.program_counter = calculate_address(address, self);
                return;
            }
            Instruction::PushProgramCounter => {
                let program_counter = self.program_counter.to_be_bytes();
                self.push_stack(program_counter[1]);
                self.push_stack(program_counter[0]);
            }
            Instruction::PopProgramCounter => {
                let program_counter_big = self.pop_stack();
                let program_counter_small = self.pop_stack();
                self.program_counter = (program_counter_big as u16) << 8 | (program_counter_small as u16);
            }
            Instruction::IncrementProgramCounter => {
                self.program_counter = self.program_counter.wrapping_add(1);
            }
            Instruction::PopProgramCounterSubroutine => {
                let program_counter_big = self.pop_stack();
                let program_counter_small = self.pop_stack();
                self.program_counter = ((program_counter_big as u16) << 8 | (program_counter_small as u16)) + 3;
            }
        }
        self.program_counter = self.program_counter.wrapping_add(1 + instruction_extra_bytes as u16);
    }
    fn push_stack(&mut self, value: u8) {
        self.memory[(self.stack_pointer as u16 + 0x0100) as usize] = value;
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
    }
    fn pop_stack(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
        
        self.memory[(self.stack_pointer as u16 + 0x0100) as usize]
    }
    /// note: cannot update the carry, that must be done manually
    fn update_status_two_operands(&mut self, operand1: u8, operand2: u8) {
        if operand1 > operand2 {
            self.status_register |= 0b001000_00;
        } else {
            self.status_register &= 0b110111_00;
        }
        if operand1 < operand2 {
            self.status_register |= 0b000100_00;
        } else {
            self.status_register &= 0b111011_00;
        }
        if operand1 == operand2 {
            self.status_register |= 0b000010_00;
        } else {
            self.status_register &= 0b111101_00;
        }
        if self.accumulator == 0 {
            self.status_register |= 0b010000_00;
        } else {
            self.status_register &= 0b101111_00;
        }
        if self.accumulator & 0b1000_0000 > 0 {
            self.status_register |= 0b000001_00;
        } else {
            self.status_register &= 0b111110_00;
        }
        self.operand1 = operand1;
        self.operand2 = operand2;
    }
    /// note: cannot update the carry, that must be done manually
    fn update_status_one_operand(&mut self, other_operand: u8) {
        if self.accumulator > other_operand {
            self.status_register |= 0b001000_00;
        } else {
            self.status_register &= 0b110111_00;
        }
        if self.accumulator < other_operand {
            self.status_register |= 0b000100_00;
        } else {
            self.status_register &= 0b111011_00;
        }
        if self.accumulator == other_operand {
            self.status_register |= 0b000010_00;
        } else {
            self.status_register &= 0b111101_00;
        }
        if self.accumulator == 0 {
            self.status_register |= 0b010000_00;
        } else {
            self.status_register &= 0b101111_00;
        }
        if self.accumulator & 0b1000_0000 > 0 {
            self.status_register |= 0b000001_00;
        } else {
            self.status_register &= 0b111110_00;
        }
        self.operand1 = other_operand;
        self.operand2 = 0x00;
    }
    /// note: cannot update the carry, that must be done manually
    fn update_status_no_operands(&mut self) {
        // not greater than, not less than, not equal to (since there's no other operand)
        self.status_register &= 0b110001_00;
        if self.accumulator == 0 {
            self.status_register |= 0b010000_00;
        } else {
            self.status_register &= 0b101111_00;
        }
        if self.accumulator & 0b1000_0000 > 0 {
            self.status_register |= 0b000001_00;
        } else {
            self.status_register &= 0b111110_00;
        }
        self.operand1 = 0x00;
        self.operand2 = 0x00;
    }
}