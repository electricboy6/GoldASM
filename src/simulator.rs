pub mod bin_parser;
mod executor;

use std::collections::VecDeque;
use std::io;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::{
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        Block,
        Paragraph,
        List,
        ListState,
        Clear,
        Wrap
    },
    DefaultTerminal, Frame,
};
use ratatui::layout::Flex;
use crate::simulator::executor::Processor;
use crate::disassembler;
use crate::disassembler::symbols::{SymbolTable, SymbolType};
use crate::simulator::bin_parser::Instruction;

#[derive(Debug, Default, Clone)]
pub struct App {
    cpu: Processor,
    exit: bool,
    binary_path: String,
    symbol_table: SymbolTable, // just empty if none
    instruction_state: ListState,
    stack_state: ListState,
    auto_run: bool,
    serial_text: Vec<char>,
    send_mode: bool,
    serial_tx_buffer: VecDeque<char>,
}
impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal, binary_path: String) -> io::Result<()> {
        self.binary_path = binary_path;
        self.symbol_table = SymbolTable::new();
        self.reset();
        self.instruction_state = ListState::default();
        self.stack_state = ListState::default();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
    pub fn run_with_symbol_table(&mut self, terminal: &mut DefaultTerminal, binary_path: String, table_path: String) -> io::Result<()> {
        self.binary_path = binary_path;

        let symbol_table_file = std::fs::read(table_path).expect("File not found.");
        self.symbol_table = SymbolTable::from_bytes(&symbol_table_file);

        self.reset();

        self.instruction_state = ListState::default();
        self.stack_state = ListState::default();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        // if auto run, then execute a step
        if self.auto_run {
            self.step();
        }
        
        // update memory lists
        *self.instruction_state.offset_mut() = 16 - 5;
        self.instruction_state.select(Some(16));

        *self.stack_state.offset_mut() = (self.cpu.stack_pointer as usize).saturating_sub(5);
        self.stack_state.select(Some(self.cpu.stack_pointer as usize));
        
        // create outer box
        let title = Line::from(" GoldCore Simulator ".bold());
        let tui_instructions = Line::from(vec![
            " Start Auto Run ".into(),
            "<A>".blue().bold(),
            " Stop Auto Run ".into(),
            "<P>".blue().bold(),
            " Send message ".into(),
            "<S>".blue().bold(),
            " Reset ".into(),
            "<Space>".blue().bold(),
            " Step ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Escape> ".blue().bold(),
        ]);
        let outer_block = Block::bordered()
            .title(title.centered())
            .title_bottom(tui_instructions.centered())
            .border_set(border::THICK);
        
        // make layout of stuff
        let [status_area, io_area, instruction_area, stack_area] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(40), Constraint::Percentage(10)]).areas(outer_block.inner(frame.area()));

        // ------------------------------ CPU STATE ------------------------------
        let title = Line::from(" CPU State ");
        let block = Block::bordered()
            .title(title);

        let mut status_register_text = String::new();
        if self.cpu.status_register & 0b100000_00 > 1 {
            push_to_string(&mut status_register_text, "carry");
        }
        if self.cpu.status_register & 0b010000_00 > 1 {
            push_to_string(&mut status_register_text, "zero");
        }
        if self.cpu.status_register & 0b001000_00 > 1 {
            push_to_string(&mut status_register_text, "greater than");
        }
        if self.cpu.status_register & 0b000100_00 > 1 {
            push_to_string(&mut status_register_text, "less than");
        }
        if self.cpu.status_register & 0b000010_00 > 1 {
            push_to_string(&mut status_register_text, "equal");
        }
        if self.cpu.status_register & 0b000001_00 > 1 {
            push_to_string(&mut status_register_text, "negative");
        }

        let status_text = Text::from(vec![Line::from(vec![
            "Accumulator: ".into(),
            format!("{:02x} ", self.cpu.accumulator).to_string().yellow(),]), Line::from(vec![
            "Registers: ".into(),
            format!("{:02x?} ", self.cpu.registers).to_string().yellow(),]), Line::from(vec![
            "Status register: ".into(),
            format!("{} ({:08b}) ", status_register_text, self.cpu.status_register).to_string().yellow(),]), Line::from(vec![
            "Operand 1: ".into(),
            format!("{:02x} ", self.cpu.operand1).to_string().yellow(),]), Line::from(vec![
            "Operand 2: ".into(),
            format!("{:02x} ", self.cpu.operand2).to_string().yellow(),]), Line::from(vec![
            "Program counter: ".into(),
            format!("{:04x} ", self.cpu.program_counter).to_string().yellow(),]), Line::from(vec![
            "Stack pointer: ".into(),
            format!("{:02x} ", self.cpu.stack_pointer).to_string().yellow(),]),
        ]);

        let cpu_state = Paragraph::new(status_text)
            .block(block);
        // ------------------------------ END CPU STATE ------------------------------
        
        // instructions list
        let memory_list_start = self.cpu.program_counter.saturating_sub(16) as usize;
        let memory_list_end = self.cpu.program_counter.saturating_add(48) as usize;

        let instructions = self.cpu.memory[memory_list_start..=memory_list_end].to_vec();

        // ------------------------------ LIVE DISASSEMBLY ------------------------------
        // create variables
        let mut parsed_instructions: Vec<Instruction> = Vec::with_capacity(instructions.len());
        let mut bytes_to_skip: Vec<u8> = Vec::with_capacity(instructions.len());
        let mut program_counter_value: u32 = 0x0200;
        let mut disassemble_start: u32 = 0;
        let mut disassemble_end: u32 = 0;
        // parse instructions
        while program_counter_value < 0xFF00.max(memory_list_end as u32) {
            // parse the instruction
            let instruction = bin_parser::parse_instruction(&self.cpu.memory, program_counter_value as u16);
            if instruction.is_ok() {
                let (parsed_instruction, num_extra_bytes) = instruction.unwrap();

                // if we're at an index after memory_list_start and before memory_list_end, add it to the list
                if program_counter_value >= memory_list_start as u32 && program_counter_value <= memory_list_end as u32 {
                    if disassemble_start == 0 {
                        disassemble_start = program_counter_value;
                    }
                    parsed_instructions.push(parsed_instruction);
                    bytes_to_skip.push(num_extra_bytes);
                    disassemble_end = program_counter_value;
                }

                // increment the program counter
                program_counter_value += num_extra_bytes as u32;
                program_counter_value += 1;
            } else {
                program_counter_value += 1;
            }
        }
        if disassemble_start < memory_list_start as u32 {
            disassemble_start = memory_list_start as u32;
        }
        if disassemble_end > memory_list_end as u32 {
            disassemble_end = memory_list_end as u32;
        }

        let mut disassembled_lines = disassembler::disassemble(parsed_instructions, bytes_to_skip);

        for _ in 0..(disassemble_start as usize - memory_list_start) {
            disassembled_lines.insert(0, "".to_string());
        }
        for _ in 0..(instructions.len() - (disassemble_end - disassemble_start) as usize) {
            disassembled_lines.push("".to_string());
        }

        let memory_strings: Vec<Line> = instructions.iter().enumerate().map(|(index, item)| -> Line {
            let disassembled_line = disassembled_lines[index].as_str();
            let memory_index = index + memory_list_start;

            let mut final_line = format!("0x{:04x?}: ", index + memory_list_start).yellow() + format!("0x{item:02x?}").green();

            if !disassembled_line.is_empty() {
                final_line = final_line + " -> ".light_green() + disassembled_line.light_green();
            }

            // ------------------------------ SYMBOL TABLE ------------------------------
            if let Some(symbol) = self.symbol_table.symbol_uses.get(&(memory_index as u16)) {
                let mut symbol = symbol.clone();
                // remove folder names
                symbol.name = symbol.name.rsplit_once('/').unwrap_or(("", &symbol.name)).1.to_string();
                // convert value to hex
                if let Ok(symbol_value) = symbol.value.parse::<u16>() {
                    symbol.value = format!("{symbol_value:04x}");
                }
                // add prefix based on type
                match symbol.symbol_type {
                    SymbolType::Pointer => {
                        symbol.name = "*".to_string() + &symbol.name;
                        // if you see an into iter on final_line, it's magic trickery so I can keep
                        // the styling while still doing operations on the string
                        let final_line_vec: Vec<Span> = final_line.into_iter().map(|span| {
                            span.clone().content(span.content.replace(&symbol.value, ""))
                        }).collect();
                        final_line = Line::from(final_line_vec);
                    }
                    SymbolType::Label => {
                        symbol.name = "~".to_string() + &symbol.name;
                        let final_line_vec: Vec<Span> = final_line.into_iter().map(|span| {
                            span.clone().content(span.content.replace(&("%".to_string() + &symbol.value), ""))
                        }).collect();
                        final_line = Line::from(final_line_vec);
                        symbol.value = "%".to_string() + &symbol.value;
                    }
                    SymbolType::Subroutine => {
                        symbol.name = "~".to_string() + symbol.name.strip_suffix("_SR").unwrap();
                        let final_line_vec: Vec<Span> = final_line.into_iter().map(|span| {
                            span.clone().content(span.content.replace(&("%".to_string() + &symbol.value), ""))
                        }).collect();
                        final_line = Line::from(final_line_vec);
                        symbol.value = "%".to_string() + &symbol.value;
                    }
                    _ => eprintln!("{symbol:?}")
                }

                final_line += format!("{}: {}", symbol.name, symbol.value).light_blue()
            }
            if let Some(symbol) = self.symbol_table.symbols.get(&(memory_index as u16)) {
                if symbol.name.ends_with("_EndSR") {
                    final_line += format!(" {}", symbol.clone().name.strip_suffix("_EndSR").unwrap()).light_blue()
                } else if symbol.name.ends_with("_SR") {
                    let mut final_line_vec: Vec<Span> = final_line.into_iter().collect::<Vec<Span>>();
                    final_line_vec.insert(0, format!("sr {}: ", symbol.clone().name.strip_suffix("_SR").unwrap()).blue());
                    final_line = Line::from(final_line_vec);
                } else {
                    let mut final_line_vec: Vec<Span> = final_line.into_iter().collect::<Vec<Span>>();
                    final_line_vec.insert(0, format!("{}: ", symbol.clone().name).blue());
                    final_line = Line::from(final_line_vec);
                }
            }
            // ------------------------------ END SYMBOL TABLE ------------------------------
            final_line
        }).collect();
        // ------------------------------ END LIVE DISASSEMBLY ------------------------------
        
        let memory_list = List::new(memory_strings)
            .block(Block::bordered().title(" Memory "))
            .highlight_symbol("-> ")
            .scroll_padding(32)
            .repeat_highlight_symbol(false);
        
        // stack list
        let stack = self.cpu.memory[0x0100..0x0200].to_vec();
        let stack_strings: Vec<Line> = stack.iter().enumerate().map(|(index, item)| -> Line {
            format!("0x{:04x?}: ", 0x0100 + index).yellow() + format!("0x{item:02x?}").green()
        }).collect();

        let stack_list = List::new(stack_strings)
            .block(Block::bordered().title(" Stack "))
            .highlight_symbol("-> ")
            .repeat_highlight_symbol(false);

        // ------------------------------ IO BLOCK ------------------------------
        let serial_text = self.serial_text.iter().collect::<String>();
        let io_block = Block::bordered()
            .title(" I/O ");

        let io_text_lines = serial_text.lines().map(|line| -> Line {
            Line::from(line.white())
        }).collect::<Vec<Line>>();
        let io_text = Paragraph::new(io_text_lines)
            .block(io_block)
            .wrap(Wrap { trim: false });
        // ------------------------------ END IO BLOCK ------------------------------
        
        // render everything
        frame.render_widget(outer_block, frame.area());
        frame.render_widget(io_text, io_area);
        frame.render_stateful_widget(memory_list, instruction_area, &mut self.instruction_state);
        frame.render_stateful_widget(stack_list, stack_area, &mut self.stack_state);
        frame.render_widget(cpu_state, status_area);
        if self.send_mode {
            let vertical = Layout::vertical([Constraint::Percentage(75)]).flex(Flex::Center);
            let horizontal = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);
            let [popup] = vertical.areas(frame.area());
            let [popup] = horizontal.areas(popup);

            let tx_buffer_string = self.serial_tx_buffer.iter().collect::<String>();
            let lines = tx_buffer_string.lines().map(|line| -> Line {
                Line::from(line.white())
            }).collect::<Vec<Line>>();
            let text_to_send = Paragraph::new(lines)
                .block(Block::bordered()
                    .title(Line::from(" Enter your message. Press <Escape> to add it to the buffer. ").centered()))
                .wrap(Wrap { trim: true })
                .on_dark_gray();
            frame.render_widget(Clear, popup);
            frame.render_widget(text_to_send, popup);
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // 100hz update rate
        // todo: make the auto run speed adjustable
        if event::poll(Duration::from_millis(10))? {
            match event::read()? {
                // events are registered on press and release
                Event::Key(key_event) if key_event.is_press() => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            };
            Ok(())
        } else {
            Ok(())
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if !key_event.is_press() {
            return;
        }
        if !self.send_mode {
            if key_event.code == KeyCode::Char('s') {
                self.send_mode = true;
                return;
            }
            match key_event.code {
                KeyCode::Char(' ') => self.reset(),
                KeyCode::Esc => self.exit(),
                // left arrow
                KeyCode::Left => (),
                // right arrow
                KeyCode::Right => self.step(),
                // up arrow
                KeyCode::Char('a') => self.auto_run = true,
                KeyCode::Char('p') => self.auto_run = false,
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Esc => {
                    self.send_mode = false;
                }
                KeyCode::Backspace => {
                    self.serial_tx_buffer.pop_back();
                }
                KeyCode::Enter => {
                    self.serial_tx_buffer.push_back('\n');
                }
                KeyCode::Char(character) => {
                    self.serial_tx_buffer.push_back(character);
                }
                _ => ()
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
    fn step(&mut self) {
        self.cpu.step();
        if self.cpu.memory[0xFF01] > 0 {
            // new data
            self.cpu.memory[0xFF01] = 0;
            self.serial_text.push(self.cpu.memory[0xFF00] as char);
        }
        if self.cpu.memory[0xFF09] == 0 && self.cpu.memory[0xFF0A] == 0 && !self.send_mode {
            // clear to send
            if let Some(character) = self.serial_tx_buffer.pop_front() {
                self.cpu.memory[0xFF08] = character as u8;
                self.cpu.memory[0xFF09] = 1; // new data
            }
        }
    }
    fn reset(&mut self) {
        self.cpu = Processor::default();
        let content = std::fs::read(&self.binary_path).expect("File not found.");
        for (index, byte) in content.iter().enumerate() {
            self.cpu.memory[index] = *byte;
        }
        self.cpu.reset();
        self.serial_text.clear();
        self.serial_tx_buffer.clear();
    }
}

fn push_to_string(string: &mut String, value_to_add: &str) {
    if !string.is_empty() {
        string.push_str(&(", ".to_string() + value_to_add));
    } else {
        string.push_str(value_to_add);
    }
}

pub fn run(source_file: String) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal, source_file);
    ratatui::restore();
    app_result
}
pub fn run_with_symbol_table(binary_file: String, symbol_table_file: String) -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run_with_symbol_table(&mut terminal, binary_file, symbol_table_file);
    ratatui::restore();
    app_result
}