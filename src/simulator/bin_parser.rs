pub struct Address {
    pub address: u16,
    pub index: Option<usize>
}
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
    Jump(Address),
    PushProgramCounter,
    PopProgramCounter,
    IncrementProgramCounter,
}

pub struct Parser {
    pub instructions: Vec<Instruction>,
    
}