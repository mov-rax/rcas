use fltk::{*, app, app::App, text::*, window::*};
//use std::ops::{Deref, DerefMut};
use crate::rcas_lib::{*, RCas, CalculationMode};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub(crate) struct Shell{
    term: SimpleTerminal,
    pub(crate) mode: CalculationMode,
    pub(crate) query: String
}

impl Shell{
    /// Creates a new shell.
    pub fn new(x:i32,y:i32,length:i32,width:i32) -> Shell {
        let mut term:SimpleTerminal = SimpleTerminal::new(x,y, length, width, ""); // Terminal
        let mode = CalculationMode::Radian; // default is set to radian

        let style = vec!{
            StyleTableEntry{
                color: Color::Black,
                font: Font::Courier,
                size: 16
            },
            StyleTableEntry{
                color: Color::Blue,
                font: Font::Courier,
                size: 16
            },
            StyleTableEntry{
                color: Color::Red,
                font: Font::Courier,
                size:16
            }
        };

        term.set_highlight_data(TextBuffer::default(), style);

        let mut shell = Shell {
            term,
            mode,
            query: String::new()
        };
        shell.append(&shell.mode.to_string());
        shell
    }

    pub(crate) fn append(&mut self, text: &str){
        self.term.append(text);
    }

    /// Sets the Shell's calculation mode.
    pub fn set_calculation_mode(&mut self, mode:CalculationMode) {
        self.mode = mode;
    }
}

impl Deref for Shell{
    type Target = SimpleTerminal;
    fn deref(&self) -> &Self::Target {&self.term}
}

impl DerefMut for Shell{
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.term}
}
