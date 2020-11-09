use fltk::{*, app, app::App, text::*, window::*};
//use std::ops::{Deref, DerefMut};
use crate::rcas_lib;
use crate::rcas_lib::CalculationMode;


// pub struct GUIController{
//     cas: Rcas,
//     shell: Shell
// }
//
// impl GUIController{
//     pub fn new() -> GUIController{
//         GUIController {cas: Rcas{}, shell: Shell::new()}
//     }
//
//     pub fn run(&mut self){
//         let app = App::default().with_scheme(app::Scheme::Base);
//         let mut window = Window::default()
//             .with_size(750, 1000)
//             .center_screen()
//             .with_label("RCAS 1.0");
//     }
// }
// #[derive(Debug, Clone)]
// struct Shell{
//     term: SimpleTerminal,
//     mode: CalculationMode,
//     query: String
// }
//
// impl Shell{
//     /// Creates a new shell.
//     pub fn new() -> Shell {
//         let mut term:SimpleTerminal = SimpleTerminal::new(5,5, 500, 1000, ""); // Terminal
//         let mode = CalculationMode::Radian; // default is set to radian
//
//         let style = vec!{
//             StyleTableEntry{
//                 color: Color::Black,
//                 font: Font::Courier,
//                 size: 16
//             },
//             StyleTableEntry{
//                 color: Color::Blue,
//                 font: Font::Courier,
//                 size: 16
//             }
//         };
//
//         term.set_highlight_data(TextBuffer::default(), style);
//
//         Shell {
//             term,
//             mode,
//             query: String::new()
//         }
//     }
//
//     fn append(&mut self, text: &str){
//         self.term.append(text);
//     }
//
//     /// Sets the Shell's calculation mode.
//     pub fn set_calculation_mode(&mut self, mode:CalculationMode) {
//         self.mode = mode;
//     }
// }
