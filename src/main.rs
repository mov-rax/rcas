use std::ops::Deref;
use crate::rcas_lib::{composer, calculate, Wrapper, RCas, SmartValue};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromStr, ToPrimitive, FromPrimitive};
use crate::rcas_gui::{Shell, EnvironmentTable, PlotViewer};
use fltk::{*, app, app::App, text::*, window::*, group::Tabs, group::Group, frame::Frame};
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use fltk::menu::MenuItem;
use fltk::image::PngImage;
use std::env;

mod rcas_lib;
mod rcas_functions;
mod rcas_gui;


fn main() {

    //let expression = "4(3-1.5*(6+4/10.5)*3)+4";
    //let cas = rcas_lib::RCas::new();
    //let result = cas.query(expression);
    //println!("{}", result);
    //Comment by Mario

    //let name_mario= String::from("Mario Vega");

    //println!("DIR {}", env::current_dir().unwrap().as_path().to_string_lossy().to_string());

    let app = App::default().with_scheme(app::Scheme::Gtk);
    let mut window:Window = Window::default()
        .with_size(1005, 800)
        .center_screen()
        .with_label("RCAS 1.0");
    let mut shell = Shell::new(5,5,490,790);
    let mut environment = EnvironmentTable::new(500, 5, 500, 407, "Environment");

    let mut plot_viewer = PlotViewer::new(500, 450, 500, 333, "Plot Viewer");


    let mut cas = RCas::new();
    //let mut controller = GUIController::new();

    window.make_resizable(true);
    window.end();
    window.show();

    //this should be removed. It is only for testing purposes
    environment.add("ans\t\t\t\t\tMatrix".to_string());
    environment.add("A\t\t\t\t\t4".to_string());
    environment.add("F\t\t\t\t\tFunction".to_string());
    //end of testing

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

    let mut environment_clone = environment.clone();
    environment_clone.handle(move |ev:Event|{
        match ev{
            Event::Push => {
                let click = app::event_button() == 1; // It is 1 if it is left click, 3 if it is right click
                if click{ //LEFT CLICK
                    if app::event_clicks(){ // DOUBLE LEFT CLICK!
                         // TODO - IMPLEMENT EDITOR HERE
                    }

                } else { //RIGHT CLICK
                    // Tooltip popup
                    let choices = ["Remove", "Edit"];
                    let mut item = MenuItem::new(&choices); //creates a new menu item
                    let (x, y) = app::event_coords(); // gets the coordinates of the click
                    if let Some(row) = environment.get_selected(){ //gets the selected row
                        if let Some(choice) = item.popup(x,y){ //tooltip pops up and the choice selected gets recieved
                            //TODO - IMPLEMENT EDIT AND REMOVE
                            match &*choice.label().unwrap(){
                                "Remove" => environment.remove_row(row),
                                _ => println!("NOT IMPLEMENTED YET")
                            }
                        }
                    }
                }
                true
            },
            _ => false
        }
    });


    app.run().unwrap();

}
