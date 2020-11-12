use fltk::{*, app, app::App, text::*, window::*, table::*};
//use std::ops::{Deref, DerefMut};
use crate::rcas_lib::{*, RCas, CalculationMode};
use std::ops::{Deref, DerefMut};
use fltk::browser::{BrowserScrollbar, Browser, MultiBrowser};
use fltk::group::{Tabs, Group};


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
#[derive(Debug, Clone)]
pub( crate) struct EnvironmentTable{
    env: MultiBrowser,
    pub lines: u32
}

impl EnvironmentTable{
    pub fn new(x:i32, y:i32, width:i32, height:i32, title:&str) -> EnvironmentTable{
        let env = MultiBrowser::new(x,y,width,height,title);
        EnvironmentTable {env, lines:0}
    }

    /// Adds an item onto the Environment Table with a given identifier.
    pub(crate) fn add(&mut self, id:String){
        self.env.add(&id);
        self.lines += 1;
    }

    /// Safely removes an item on the Environment Table when given its identifier.
    pub fn remove(&mut self, id:String){
        for i in 0..self.lines{
            let text = self.text(i+1);
            if let Some(text) = text{
                if text.contains(&id){
                    self.env.remove(i+1);
                }
            }
        }
        self.lines -= 1;
    }
    /// Removes a row given a number. Row starts at 1.
    pub fn remove_row(&mut self, row:u32){
        self.env.remove(row);
    }

    /// Returns the selected Row (if any selected)
    pub fn get_selected(&self) -> Option<u32>{
        for i in 0..self.lines{
            if self.selected(i+1){
                return Some(i+1);
            }
        }
        None
    }


}

impl Deref for EnvironmentTable{
    type Target = MultiBrowser;
    fn deref(&self) -> &Self::Target {&self.env }
}

impl DerefMut for EnvironmentTable{
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.env }
}

pub struct PlotViewer{
    env: Tabs
}

impl PlotViewer{
    pub fn new(x:i32,y:i32,width:i32,height:i32,title:&str) -> PlotViewer{
        let mut env = Tabs::new(x,y,width,height,title);
        let mut default_tab = Group::new(0,y-30,width,height,"DEFAULT?");
        default_tab.end();
        let mut default_tab2 = Group::new(0,y-30,width,height,"DEFAULT2?");
        default_tab2.end();
        let mut default_tab3 = Group::new(0,y-30,width,height,"DEFAULT3?");
        default_tab3.end();
        //env.set_value(&default_tab);
        //default_tab.set_color(Color::Red);
        println!("{:?}", env.client_area());
        //default_tab.end();
        PlotViewer {env}
    }
}

impl Deref for PlotViewer{
    type Target = Tabs;
    fn deref(&self) -> &Self::Target {&self.env}
}

impl DerefMut for PlotViewer{
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.env}
}
