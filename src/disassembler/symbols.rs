use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::asm_parser;

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub symbol_map: HashMap<u16, SymbolUse>,
}
impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: Vec::new(),
            symbol_map: HashMap::new(),
        }
    }
    pub fn add_label(&mut self, label: asm_parser::Label) {

    }
    pub fn add_subroutine(&mut self, subroutine: asm_parser::Subroutine) {

    }
    pub fn add_pointer(&mut self, pointer: asm_parser::Pointer) {

    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Symbol {
    pub name: String,
    pub address: u16,
    pub symbol_type: SymbolType
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SymbolUse {
    pub address: u16,
    pub symbol: Symbol,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SymbolType {
    Label(u16),
    Pointer(u16),
    Subroutine(u16),
}