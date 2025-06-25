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
use ratatui::style::Style;
use crate::simulator::executor::CPU;

#[derive(Debug, Default)]
pub struct App {
    cpu: CPU,
    exit: bool,
    binary_path: String,
    instruction_state: ListState,
    stack_state: ListState,
    auto_run: bool,
}
impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal, binary_path: String) -> io::Result<()> {
        self.binary_path = binary_path;
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
        // todo: nest the memory blocks in the status blocks or something like that
        if self.auto_run {
            self.step();
        }
        let title = Line::from(" GoldISA Simulator ".bold());
        let instructions = Line::from(vec![
            //" Decrement ".into(),
            //"<Left>".blue().bold(),
            " Auto run ".into(),
            "<Up>".blue().bold(),
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
        //frame.render_widget(outer_block, frame.area());
        let [status_area, instruction_area, stack_area] =
            Layout::horizontal([Constraint::Fill(6), Constraint::Fill(2), Constraint::Fill(2)]).areas(frame.area());
        let instructions = self.cpu.memory[self.cpu.program_counter.saturating_sub(128) as usize..(self.cpu.program_counter.saturating_add(128)) as usize]
            .to_vec();
        let memory_strings: Vec<String> = instructions.iter().map(|item| -> String {
            format!("0x{:02x?}", item)
        }).collect();
        
        let memory_list = List::new(memory_strings)
            .block(Block::bordered().title("Memory"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol("-> ")
            .repeat_highlight_symbol(false);
        
        let stack = self.cpu.memory[0x0100..0x0200].to_vec();
        let stack_strings: Vec<String> = stack.iter().map(|item| -> String {
            format!("0x{:02x?}", item)
        }).collect();
        let stack_list = List::new(stack_strings)
            .block(Block::bordered().title("Stack"))
            .highlight_style(Style::new().reversed())
            .highlight_symbol("-> ")
            .repeat_highlight_symbol(false);
        
        
        frame.render_stateful_widget(memory_list, instruction_area, &mut self.instruction_state);
        frame.render_stateful_widget(stack_list, stack_area, &mut self.stack_state);
        frame.render_widget(self, status_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
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
        match key_event.code {
            KeyCode::Char(' ') => self.reset(),
            KeyCode::Char('q') => self.exit(),
            // left arrow
            KeyCode::Left => (),
            // right arrow
            KeyCode::Right => self.step(),
            // up arrow
            KeyCode::Up => self.auto_run = true,
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
    fn step(&mut self) {
        self.cpu.step();
        
        *self.instruction_state.offset_mut() = (self.cpu.program_counter as usize).saturating_sub(5);
        self.instruction_state.select(Some(self.cpu.program_counter as usize));
        
        *self.stack_state.offset_mut() = (self.cpu.stack_pointer as usize).saturating_sub(5);
        self.stack_state.select(Some(self.cpu.stack_pointer as usize));
    }
    fn reset(&mut self) {
        self.cpu = CPU::default();
        let content = std::fs::read(&self.binary_path).expect("File not found.");
        for (index, byte) in content.iter().enumerate() {
            self.cpu.memory[index] = *byte;
        }
        self.cpu.reset();
    }
}
impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" GoldISA Simulator ".bold());
        let instructions = Line::from(vec![
            //" Decrement ".into(),
            //"<Left>".blue().bold(),
            " Auto run ".into(),
            "<Up>".blue().bold(),
            " Reset ".into(),
            "<Space>".blue().bold(),
            " Step ".into(),
            "<Right>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let status_text = Text::from(vec![Line::from(vec![
            "Accumulator: ".into(),
            format!("{:02x} ", self.cpu.accumulator).to_string().yellow(),
            "Registers: ".into(),
            format!("{:02x?} ", self.cpu.registers).to_string().yellow(),
            "Status register: ".into(),
            format!("{:08b} ", self.cpu.status_register).to_string().yellow(),
            "Program counter: ".into(),
            format!("{:04x} ", self.cpu.program_counter).to_string().yellow(),
            "Instruction values: ".into(),
            format!(
                "[{:02x}, {:02x}, {:02x}, {:02x}]",
                self.cpu.memory[self.cpu.program_counter as usize],
                self.cpu.memory[self.cpu.program_counter as usize + 1],
                self.cpu.memory[self.cpu.program_counter as usize + 2],
                self.cpu.memory[self.cpu.program_counter as usize + 3],
            ).to_string().yellow(),
        ])]);

        Paragraph::new(status_text)
            .centered()
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

fn render_list(area: Rect, buf: &mut Buffer, list: List, list_state: &mut ListState) {
    StatefulWidget::render(list, area, buf, list_state);
}