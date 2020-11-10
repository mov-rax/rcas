use std::ops::Deref;
use crate::rcas_lib::{composer, calculate, Wrapper, RCas};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromStr, ToPrimitive, FromPrimitive};
use crate::rcas_gui::{Shell};
use fltk::{*, app, app::App, text::*, window::*};
use std::time::Instant;

mod rcas_lib;
mod rcas_functions;
mod rcas_gui;



fn main() {
    //let expression = "4(3-1.5*(6+4/10.5)*3)+4";
    //let cas = rcas_lib::RCas::new();
    //let result = cas.query(expression);
    //println!("{}", result);



    let app = App::default().with_scheme(app::Scheme::Gtk);
    let mut window:Window = Window::default()
        .with_size(510, 1000)
        .center_screen()
        .with_label("RCAS 1.0");
    let mut shell = Shell::new(5,5,500,990);
    let mut cas = RCas::new();
    //let mut controller = GUIController::new();


    window.make_resizable(true);
    window.end();
    window.show();

    let mut shell_clone = shell.clone();
    shell_clone.handle( move |ev:Event| {
        match ev{
            Event::KeyDown => match app::event_key(){ // gets a keypress
                Key::Enter => {
                    shell.append("\n"); // newline character
                    let now = Instant::now();
                    let result = cas.query(&shell.query); // gets the result
                    println!("QUERY DURATION: {} µs", now.elapsed().as_micros());
                    shell.append(&format!("{}\n{}", result, &shell.mode.to_string())); // appends the result to the shell
                    shell.query.clear(); // clears the current query

                    true
                },
                Key::BackSpace => { // BACKSPACE TO REMOVE CHARACTER FROM SHELL AND THE QUERY
                    if !shell.query.is_empty(){
                        let len = shell.text().len() as u32;
                        shell.buffer().unwrap().remove(len - 1, len); // removes the last character in the buffer
                        shell.query.pop().unwrap(); // removes the last character from the query
                        true
                    } else {
                        false
                    }
                },
                _ => { // ANY OTHER KEY
                    let key = app::event_text();
                    shell.append(&key);
                    shell.query.push_str(&key);
                    true
                }
            }
            _ => false, //any other event that is not needed
        }
    });

    app.run().unwrap();

}
