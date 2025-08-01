pub mod bin_parser;
mod executor;

use std::io;
use std::time::Duration;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::prelude::*;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        Block,
        Paragraph,
        Widget,
        List,
        ListState
    },
    DefaultTerminal, Frame,
};
use crate::simulator::executor::Processor;
use crate::disassembler;
use crate::disassembler::symbols::SymbolTable;
use crate::simulator::bin_parser::Instruction;

#[derive(Debug, Default)]
pub struct App {
    cpu: Processor,
    exit: bool,
    binary_path: String,
    symbol_table: Option<SymbolTable>,
    instruction_state: ListState,
    stack_state: ListState,
    auto_run: bool,
    serial_buffer: Vec<char>
}
impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal, binary_path: String) -> io::Result<()> {
        self.binary_path = binary_path;
        self.symbol_table = None;
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
        self.symbol_table = Some(SymbolTable::from_bytes(&symbol_table_file));

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
        let instructions = Line::from(vec![
            " Start Auto Run ".into(),
            "<Up>".blue().bold(),
            " Stop Auto Run ".into(),
            "<Down>".blue().bold(),
            " Reset ".into(),
            "<Space>".blue().bold(),
            " Step ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let outer_block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        
        // make layout of stuff
        let [status_area, io_area, instruction_area, stack_area] =
            Layout::horizontal([Constraint::Fill(3), Constraint::Fill(3), Constraint::Fill(2), Constraint::Fill(2)]).areas(outer_block.inner(frame.area()));
        
        // instructions list
        let memory_list_start = self.cpu.program_counter.saturating_sub(16) as usize;
        let memory_list_end = self.cpu.program_counter.saturating_add(16) as usize;

        let instructions = self.cpu.memory[memory_list_start..=memory_list_end].to_vec();

        // ---------------------------- LIVE DISASSEMBLY ----------------------------
        // create variables
        let mut parsed_instructions: Vec<Instruction> = Vec::with_capacity(0xFFFF / 2);
        let mut bytes_to_skip: Vec<u8> = Vec::with_capacity(0xFFFF / 2);
        let mut program_counter_value: u32 = 0x0200;
        let mut disassemble_start: u32 = 0;
        let mut disassemble_end: u32 = 0;
        // parse instructions
        while program_counter_value < 0xFF00.max(memory_list_end as u32) {
            // parse the instruction
            let instruction = bin_parser::parse_instruction(&self.cpu.memory, program_counter_value as u16);
            if instruction.is_ok() {
                let (parsed_instruction, num_extra_bytes) = instruction.unwrap();

                // set the disassembly start
                if program_counter_value >= memory_list_start as u32 && disassemble_start == 0 {
                    disassemble_start = program_counter_value;
                }
                // increment the program counter
                program_counter_value += num_extra_bytes as u32;
                program_counter_value += 1;
                // if we're at a valid instruction after memory_list_start and before memory_list_end, add it to the list
                if program_counter_value >= memory_list_start as u32 && program_counter_value <= memory_list_end as u32 {
                    parsed_instructions.push(parsed_instruction);
                    bytes_to_skip.push(num_extra_bytes);
                    disassemble_end = program_counter_value;
                }
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

        // todo: right after the jsr, everything is misaligned and I don't know why.
        // everything's decoding properly and the disassembled lines are in the right spot, so maybe the memory index is screwed up?
        if (memory_list_end - memory_list_start) > disassembled_lines.len()  {
            for _ in 0..(disassemble_start as usize - memory_list_start) {
                disassembled_lines.insert(0, "".to_string());
            }
        }
        for _ in 0..(instructions.len() - (disassemble_end - disassemble_start) as usize) {
            disassembled_lines.push("".to_string());
        }

        let memory_strings: Vec<Line> = instructions.iter().enumerate().map(|(index, item)| -> Line {
            let disassembled_line = disassembled_lines[index].as_str();
            if disassembled_line.is_empty() {
                format!("0x{:04x?}: ", index + memory_list_start).yellow() + format!("0x{item:02x?}").green()
            } else {
                format!("0x{:04x?}: ", index + memory_list_start).yellow() + format!("0x{item:02x?}").green() + " -> ".light_green() + disassembled_line.light_green()
            }
        }).collect();
        // ---------------------------- END LIVE DISASSEMBLY ----------------------------
        
        let memory_list = List::new(memory_strings)
            .block(Block::bordered().title("Memory"))
            .highlight_symbol("-> ")
            .scroll_padding(32)
            .repeat_highlight_symbol(false);
        
        // stack list
        let stack = self.cpu.memory[0x0100..0x0200].to_vec();
        let stack_strings: Vec<Line> = stack.iter().enumerate().map(|(index, item)| -> Line {
            format!("0x{:04x?}: ", 0x0100 + index).yellow() + format!("0x{item:02x?}").green()
        }).collect();

        let stack_list = List::new(stack_strings)
            .block(Block::bordered().title("Stack"))
            .highlight_symbol("-> ")
            .repeat_highlight_symbol(false);

        // --------------------- IO Block ---------------------
        let serial_text = self.serial_buffer.iter().collect::<String>();
        let io_text = Text::from(serial_text.lines().map(|line| -> Line {
            Line::from(line.white())
        }).collect::<Vec<Line>>());
        let io_block = Block::bordered()
            .title("I/O");
        let io_paragraph = Paragraph::new(io_text).block(io_block);
        // --------------------- End IO Block ---------------------
        
        // render everything
        frame.render_widget(outer_block, frame.area());
        frame.render_widget(io_paragraph, io_area);
        frame.render_stateful_widget(memory_list, instruction_area, &mut self.instruction_state);
        frame.render_stateful_widget(stack_list, stack_area, &mut self.stack_state);
        frame.render_widget(self, status_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        // 10hz update rate
        if event::poll(Duration::from_millis(100))? {
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
        match key_event.code {
            KeyCode::Char(' ') => self.reset(),
            KeyCode::Char('q') => self.exit(),
            // left arrow
            KeyCode::Left => (),
            // right arrow
            KeyCode::Right => self.step(),
            // up arrow
            KeyCode::Up => self.auto_run = true,
            KeyCode::Down => self.auto_run = false,
            _ => {}
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
            self.serial_buffer.push(self.cpu.memory[0xFF00] as char);
        }
    }
    fn reset(&mut self) {
        self.cpu = Processor::default();
        let content = std::fs::read(&self.binary_path).expect("File not found.");
        for (index, byte) in content.iter().enumerate() {
            self.cpu.memory[index] = *byte;
        }
        self.cpu.reset();
        self.serial_buffer.clear();
    }
}

fn push_to_string(string: &mut String, value_to_add: &str) {
    if !string.is_empty() {
        string.push_str(&(", ".to_string() + value_to_add));
    } else {
        string.push_str(value_to_add);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        Paragraph::new(status_text)
            .block(block)
            .render(area, buf);
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