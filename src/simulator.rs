pub mod bin_parser;
mod executor;

use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use crate::simulator::executor::CPU;

#[derive(Debug, Default)]
pub struct App {
    cpu: CPU,
    exit: bool,
    binary_path: String,
}
impl App {

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal, binary_path: String) -> io::Result<()> {
        self.binary_path = binary_path;
        self.reset();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // events are registered on press and release
            Event::Key(key_event) if key_event.is_press() => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(' ') => self.reset(),
            KeyCode::Char('q') => self.exit(),
            // left arrow
            KeyCode::Left => (),
            // right arrow
            KeyCode::Right => self.step(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
    fn step(&mut self) {
        self.cpu.step();
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
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" GoldISA Simulator ".bold());
        let instructions = Line::from(vec![
            //" Decrement ".into(),
            //"<Left>".blue().bold(),
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
