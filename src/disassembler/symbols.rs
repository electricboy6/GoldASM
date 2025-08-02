use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::{asm_parser, assembler};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    // the u16 is the index of the line in the binary
    pub symbol_map: HashMap<u16, Symbol>,
}
impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: Vec::new(),
            symbol_map: HashMap::new(),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).unwrap()
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        rmp_serde::from_slice(bytes).unwrap()
    }
    pub fn add_define(&mut self, define: assembler::AssemblerDefine) {
        self.symbols.push(Symbol {
            name: define.name,
            value: define.value,
            symbol_type: SymbolType::Define,
        });
    }
    pub fn add_pointer_use(&mut self, pointer_use: assembler::AssemblerPointerUse, pointer: assembler::AssemblerDefine) {
        self.symbol_map.insert(
            pointer_use.index,
            Symbol {
                name: pointer.name,
                value: asm_parser::Address::from_str(&pointer.value).address.to_decimal().to_string(),
                symbol_type: SymbolType::Pointer,
            }
        );
    }
    pub fn add_label(&mut self, label: assembler::AssemblerLabel) {
        self.symbols.push(Symbol {
            name: label.name,
            value: label.address.to_string(),
            symbol_type: SymbolType::Label,
        });
    }
    pub fn add_label_use(&mut self, label_use: assembler::AssemblerLabelUse, label: assembler::AssemblerLabel) {
        if label.name.ends_with("_Subroutine") {
            self.symbol_map.insert(
                label_use.index,
                Symbol {
                    name: label.name,
                    value: label.address.to_string(),
                    symbol_type: SymbolType::Subroutine,
                }
            );
        } else {
            self.symbol_map.insert(
                label_use.index,
                Symbol {
                    name: label.name,
                    value: label.address.to_string(),
                    symbol_type: SymbolType::Label,
                }
            );
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Symbol {
    pub name: String,
    pub value: String,
    pub symbol_type: SymbolType
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum SymbolType {
    Label,
    Pointer,
    Define,
    Subroutine,
}