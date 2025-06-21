use crate::simulator::bin_parser::{Address, Instruction};


fn calculate_address(address: Address, cpu: &CPU) -> u16 {
    let real_address;
    if let Some(index) = address.index {
        real_address = address.address.wrapping_add(cpu.registers[index] as u16);
    } else {
        real_address = address.address;
    }
    real_address
}

pub struct CPU {
    pub accumulator: u8,
    pub registers: [u8; 8],
    pub status_register: u8,
    pub stack_pointer: u8,
    pub memory: [u8; 65536],
    pub program_counter: u16,
}
impl CPU {
    pub fn new(reset_vector: u16) -> CPU {
        let mut cpu = CPU {
            accumulator: 0,
            registers: [0; 8],
            status_register: 0b010000_00, // accumulator starts at zero, so zero flag is 1
            stack_pointer: 0x00,
            memory: [0; 65536],
            program_counter: 0x0000,
        };
        let reset_vector = reset_vector.to_be_bytes();
        cpu.memory[0xFFFC] = reset_vector[1];
        cpu.memory[0xFFFD] = reset_vector[0];
        cpu
    }
    pub fn run_instruction(&mut self, instruction: Instruction) {
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
                        self.status_register = self.status_register | 0b100000_00
                    } else {
                        self.status_register = self.status_register & 0b011111_00
                    }
                    self.update_status_one_operand(register_value as u8);
                } else if let Some((register1, register2)) = two_register {
                    let current_carry = (self.status_register & 0b100000_00).min(1) as u16;
                    let register1_value = self.registers[register1 as usize] as u16;
                    let register2_value = self.registers[register2 as usize] as u16;
                    let addition_result = register1_value + register2_value + current_carry;
                    let carry = addition_result & 0b0000_0001_0000_0000;
                    self.accumulator = (addition_result & 0b0000_0000_1111_1111) as u8;
                    if carry > 0 {
                        self.status_register = self.status_register | 0b100000_00
                    } else {
                        self.status_register = self.status_register & 0b011111_00
                    }
                    self.update_status_two_operands(register1_value as u8, register2_value as u8);
                }
            }
            Instruction::Subtract(one_register, two_register) => {
                // todo: fix carry logic (make carry in work as well as carry out)
                let current_carry = (self.status_register & 0b100000_00).min(1) as u16;
                let acc_value = self.accumulator as u16 | (current_carry << 9);
                if let Some(register) = one_register {
                    let (result, carry) = self.accumulator
                        .overflowing_sub(self.registers[register as usize]);
                    self.accumulator = result;
                    
                    if carry {
                        self.status_register = self.status_register | 0b100000_00
                    } else {
                        self.status_register = self.status_register & 0b011111_00
                    }
                    
                    self.update_status_one_operand(self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    let register1_value = self.registers[register1 as usize];
                    let register2_value = self.registers[register2 as usize];
                    
                    let (result, carry) = register1_value.overflowing_sub(register2_value);
                    self.accumulator = result;
                    
                    if carry {
                        self.status_register = self.status_register | 0b100000_00
                    } else {
                        self.status_register = self.status_register & 0b011111_00
                    }
                    
                    self.update_status_two_operands(register1_value, register2_value);
                }
            }
            Instruction::Xor(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = self.accumulator ^ self.registers[register as usize];
                    self.update_status_one_operand(self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = self.registers[register1 as usize] ^ self.registers[register2 as usize];
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Xnor(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = !(self.accumulator ^ self.registers[register as usize]);
                    self.update_status_one_operand(self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = !(self.registers[register1 as usize] ^ self.registers[register2 as usize]);
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::And(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = self.accumulator & self.registers[register as usize];
                    self.update_status_one_operand(self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = self.registers[register1 as usize] & self.registers[register2 as usize];
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Nand(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = !(self.accumulator & self.registers[register as usize]);
                    self.update_status_one_operand(self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = !(self.registers[register1 as usize] & self.registers[register2 as usize]);
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Or(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = self.accumulator | self.registers[register as usize];
                    self.update_status_one_operand(self.registers[register as usize]);
                } else if let Some((register1, register2)) = two_register {
                    self.accumulator = self.registers[register1 as usize] | self.registers[register2 as usize];
                    self.update_status_two_operands(self.registers[register1 as usize], self.registers[register2 as usize]);
                }
            }
            Instruction::Nor(one_register, two_register) => {
                if let Some(register) = one_register {
                    self.accumulator = !(self.accumulator | self.registers[register as usize]);
                    self.update_status_one_operand(self.registers[register as usize]);
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
            // todo: make carry in work as well as carry out
            Instruction::ShiftRight => {
                let carry = self.accumulator & 0b0000_0001;
                self.accumulator = self.accumulator << 1;
                if carry > 0 {
                    self.status_register = self.status_register | 0b100000_00;
                } else {
                    self.status_register = self.status_register & 0b011111_00;
                }
                self.update_status_no_operands();
            }
            Instruction::ShiftLeft => {
                let shifted_result = (self.accumulator as u16) >> 1;
                let carry = shifted_result & 0b0000_0001_0000_0000;
                self.accumulator = (shifted_result & 0b0000_0000_1111_1111) as u8;
                if carry > 0 {
                    self.status_register = self.status_register | 0b100000_00;
                } else {
                    self.status_register = self.status_register & 0b011111_00;
                }
                self.update_status_no_operands();
            }
            Instruction::Noop => {}
            Instruction::SetCarry => {
                self.status_register = self.status_register | 0b100000_00
            }
            Instruction::ClearCarry => {
                self.status_register = self.status_register & 0b011111_00
            }
            Instruction::PushRegisterToStack(register) => {
                self.memory[self.stack_pointer as usize] = self.registers[register as usize];
                self.stack_pointer = self.stack_pointer.wrapping_add(1);
            }
            Instruction::PopRegisterFromStack(register) => {
                self.registers[register as usize] = self.memory[self.stack_pointer as usize];
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
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
                if self.status_register & 0b100000_00 == 1 {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::BranchCarryClear(address) => {
                if self.status_register & 0b100000_00 == 0 {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::BranchNegative(address) => {
                if self.status_register & 0b000001_00 == 1 {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::BranchPositive(address) => {
                if self.status_register & 0b000001_00 == 0 {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::BranchEqual(register, address) => {
                if self.registers[register as usize] == self.accumulator {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::BranchNotEqual(register, address) => {
                if self.registers[register as usize] != self.accumulator {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::BranchZero(address) => {
                if self.status_register & 0b010000_00 == 1 {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::BranchNotZero(address) => {
                if self.status_register & 0b010000_00 == 0 {
                    self.program_counter = calculate_address(address, self);
                }
            }
            Instruction::Jump(address) => {
                self.program_counter = calculate_address(address, self);
            }
            Instruction::PushProgramCounter => {
                let program_counter = self.program_counter.to_be_bytes();
                self.memory[self.stack_pointer as usize] = program_counter[1];
                self.stack_pointer = self.stack_pointer.wrapping_add(1);
                self.memory[self.stack_pointer as usize] = program_counter[0];
                self.stack_pointer = self.stack_pointer.wrapping_add(1);
            }
            Instruction::PopProgramCounter => {
                let program_counter_big = self.memory[self.stack_pointer as usize];
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                let program_counter_small = self.memory[self.stack_pointer as usize];
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.program_counter = (program_counter_big as u16) << 8 | (program_counter_small as u16);
            }
            Instruction::IncrementProgramCounter => {
                self.program_counter = self.program_counter.wrapping_add(1);
            }
        }
        self.program_counter = self.program_counter.wrapping_add(1);
    }
    fn update_status_two_operands(&mut self, operand1: u8, operand2: u8) {
        if operand1 > operand2 {
            self.status_register = self.status_register | 0b001000_00;
        } else {
            self.status_register = self.status_register & 0b110111_00;
        }
        if operand1 < operand2 {
            self.status_register = self.status_register | 0b000100_00;
        } else {
            self.status_register = self.status_register & 0b111011_00;
        }
        if operand1 == operand2 {
            self.status_register = self.status_register | 0b000010_00;
        } else {
            self.status_register = self.status_register & 0b111101_00;
        }
        if self.accumulator == 0 {
            self.status_register = self.status_register | 0b010000_00;
        } else {
            self.status_register = self.status_register & 0b101111_00;
        }
        if self.accumulator & 0b1000_0000 > 0 {
            self.status_register = self.status_register | 0b000001_00;
        } else {
            self.status_register = self.status_register & 0b111110_00;
        }
    }
    /// note: cannot update the carry, that must be done manually
    fn update_status_one_operand(&mut self, other_operand: u8) {
        if self.accumulator > other_operand {
            self.status_register = self.status_register | 0b001000_00;
        } else {
            self.status_register = self.status_register & 0b110111_00;
        }
        if self.accumulator < other_operand {
            self.status_register = self.status_register | 0b000100_00;
        } else {
            self.status_register = self.status_register & 0b111011_00;
        }
        if self.accumulator == other_operand {
            self.status_register = self.status_register | 0b000010_00;
        } else {
            self.status_register = self.status_register & 0b111101_00;
        }
        if self.accumulator == 0 {
            self.status_register = self.status_register | 0b010000_00;
        } else {
            self.status_register = self.status_register & 0b101111_00;
        }
        if self.accumulator & 0b1000_0000 > 0 {
            self.status_register = self.status_register | 0b000001_00;
        } else {
            self.status_register = self.status_register & 0b111110_00;
        }
    }
    /// note: cannot update the carry, that must be done manually
    fn update_status_no_operands(&mut self) {
        // not greater than, not less than, not equal to (since there's no other operand)
        self.status_register = self.status_register & 0b110001_00;
        if self.accumulator == 0 {
            self.status_register = self.status_register | 0b010000_00;
        } else {
            self.status_register = self.status_register & 0b101111_00;
        }
        if self.accumulator & 0b1000_0000 > 0 {
            self.status_register = self.status_register | 0b000001_00;
        } else {
            self.status_register = self.status_register & 0b111110_00;
        }
    }
}