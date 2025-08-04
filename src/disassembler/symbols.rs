use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::{asm_parser, assembler};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default, Clone)]
pub struct SymbolTable {
    pub symbols: HashMap<u16, Symbol>,
    // the u16 is the index of the line in the binary
    pub symbol_uses: HashMap<u16, Symbol>,
}
impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
            symbol_uses: HashMap::new(),
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        rmp_serde::to_vec(&self).unwrap()
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        rmp_serde::from_slice(bytes).unwrap()
    }
    pub fn add_define(&mut self, define: assembler::AssemblerDefine) {
        self.symbols.insert(define.value.parse().unwrap_or(0), Symbol {
            name: define.name,
            value: define.value,
            symbol_type: SymbolType::Define,
        });
    }
    pub fn add_define_use(&mut self, pointer_use: assembler::AssemblerDefineUse, pointer: assembler::AssemblerDefine) {
        self.symbol_uses.insert(
            pointer_use.index,
            Symbol {
                name: pointer.name,
                value: asm_parser::Address::from_str(&pointer.value).address.to_decimal().to_string(),
                symbol_type: SymbolType::Pointer,
            }
        );
    }
    pub fn add_label(&mut self, label: assembler::AssemblerLabel) {
        if label.name.ends_with("_EndSR") {
            self.symbols.insert(label.address - 1,
                                Symbol {
                                    name: label.name,
                                    value: label.address.to_string(),
                                    symbol_type: SymbolType::Label,
                                });
        } else {
            self.symbols.insert(label.address,
                                Symbol {
                                    name: label.name,
                                    value: label.address.to_string(),
                                    symbol_type: SymbolType::Label,
                                });
        }
    }
    pub fn add_label_use(&mut self, label_use: assembler::AssemblerLabelUse, label: assembler::AssemblerLabel) {
        if label.name.ends_with("_SR") {
            self.symbol_uses.insert(
                label_use.instruction_index - 1,
                Symbol {
                    name: label.name,
                    value: label.address.to_string(),
                    symbol_type: SymbolType::Subroutine,
                }
            );
        } else {
            self.symbol_uses.insert(
                label_use.instruction_index,
                Symbol {
                    name: label.name,
                    value: label.address.to_string(),
                    symbol_type: SymbolType::Label,
                }
            );
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Symbol {
    pub name: String,
    pub value: String,
    pub symbol_type: SymbolType
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymbolType {
    Label,
    Pointer,
    Define,
    Subroutine,
}