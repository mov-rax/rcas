use std::ops::Deref;
use crate::rcas_lib::{composer, calculate, Wrapper, RCas, SmartValue, QueryResult};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromStr, ToPrimitive, FromPrimitive};
use crate::rcas_gui::{Shell, EnvironmentTable, PlotViewer};
use fltk::{*, app, app::App, text::*, window::*, group::Tabs, group::Group, frame::Frame};
use std::time::Instant;
use std::collections::{HashMap, HashSet};
use fltk::menu::MenuItem;
use fltk::image::PngImage;
use std::env;
use std::rc::Rc;
use std::cell::RefCell;
use clipboard::{ClipboardProvider, ClipboardContext};

use std::borrow::Borrow;
use std::sync::Mutex;
use fltk::app::event_key;

mod rcas_lib;
mod rcas_functions;
mod rcas_gui;
mod data;


fn main() {

    //let expression = "4(3-1.5*(6+4/10.5)*3)+4";
    //let cas = rcas_lib::RCas::new();
    //let result = cas.query(expression);
    //println!("{}", result);
    //Comment by Mario

    // let exp = 2.7182818284590452353602874713527;
    // let 2(exp);
    //println!("{}", result);

    //let name_mario= String::from("Mario Vega");

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

    let mut plot_viewer_clone = plot_viewer.clone();
    let plot_viewer = Rc::from(RefCell::from(plot_viewer));
    let pvc = plot_viewer.clone();
    let window = Rc::from(RefCell::from(window));
    let win = window.clone();
    plot_viewer_clone.handle(move |ev:Event| {
        match ev{
            Event::Push => {
                let click = app::event_button() == 1; // true if left click, false if right
                let mut pvc = pvc.borrow_mut(); // gets a mutable reference to the plot viewer, it is necessary for removing a plot
                let mut win = win.borrow_mut(); // gets a mutable reference to the window, which is necessary for refreshing the window.
                if let Some(image_frame) = pvc.value(){ // Gets the currently visible group
                    if let Some(locations) = pvc.img_locations.get(&image_frame.label()){ //get the location of the image
                        let (i_x,i_y,i_w,i_h) = *locations;
                        if !click && app::event_inside(i_x,i_y,i_w,i_h){ // Checks to see if the click is within the image's bounds
                            let choices = ["Save Plot", "Remove Plot"];
                            let mut item = MenuItem::new(&choices);
                            let (x,y) = app::event_coords(); //coordinates of the click
                            if let Some(choice) = item.popup(x,y){ // Shows the menu and gets the choice (if any was chosen)
                                match &*choice.label().unwrap(){
                                    //TODO - IMPLEMENT THE SAVING FUNCTION
                                    "Remove Plot" => {
                                        app::delete_widget(pvc.value().unwrap()); // REMOVES THE PLOT
                                        pvc.redraw();
                                        let (width,height) = (win.width(), win.height());
                                        win.set_size(width+1,height+1);
                                        win.set_size(width,height);
                                    },
                                    "Save Plot" => {
                                        pvc.save_visible_plot_prompt();
                                    }
                                    _ => {return false}
                                }
                            }
                            pvc.redraw();
                            return true;
                        }
                    }
                }
                pvc.redraw();
                false
            },
            _ => false
        }
    });

    let mut controlled = false;
    let pvc = plot_viewer.clone(); // a nice reference to the plot viewer
    let mut shell_clone = shell.clone();
    shell_clone.handle( move |ev:Event| {
        match ev{
            Event::KeyDown => match app::event_key(){ // gets a keypress
                Key::Enter | Key::KPEnter => {
                    let mut pvc = pvc.borrow_mut(); // Gets a mutable reference to the PlotViewer

                    shell.append("\n"); // newline character
                    //let now = Instant::now();
                    let result = cas.query(&shell.query); // gets the result
                    //println!("QUERY DURATION: {} µs", now.elapsed().as_micros());
                    let mut answer = String::new();
                    match result{
                        QueryResult::Simple(result) => {answer = result},
                        QueryResult::Error(err) => {
                            answer = err;
                            fltk::dialog::beep(fltk::dialog::BeepType::Error); // a nice beep to show that you did something wrong
                        },
                        _ => {}
                    }
                    pvc.begin();
                    pvc.add_test_img_tab("OOGA"); // TODO - THIS SHOULD BE CHANGED TO AN ACTUAL PLOT
                    pvc.redraw();
                    pvc.end();

                    shell.append(&format!("{}\n{}", answer, &shell.mode.to_string())); // appends the result to the shell
                    shell.renew_query(); // clears the current query and puts its value into history

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
                Key::Up => {
                    let len = shell.text().len() as u32;
                    let query_len = shell.query.len() as u32;
                    shell.buffer().unwrap().remove(len-query_len, len);
                    shell.query.clear();
                    let text = shell.older_history();
                    shell.append(&*text);
                    shell.query = text;
                    true
                },
                Key::Down => {
                    let len = shell.text().len() as u32;
                    let query_len = shell.query.len() as u32;
                    shell.buffer().unwrap().remove(len-query_len, len);
                    shell.query.clear();
                    let text = shell.newer_history();
                    shell.append(&*text);
                    shell.query = text;
                    true
                },
                k => {
                    //println!("{:?}", &k);
                    if k == Key::ControlL { controlled = true;}
                    if k == Key::from_i32(0x76) && controlled{ //CONTROL-V (PASTE)
                        controlled = false;
                        let mut cb:ClipboardContext = clipboard::ClipboardProvider::new().unwrap(); // Object that lets us get text in the clipboard :)
                        if let Ok(text) = cb.get_contents(){
                            println!("{}", &text);
                            shell.append(&*text);
                            shell.query.push_str(&*text);
                        }
                        return true;
                    }
                    if k == Key::from_i32(0x63) && controlled{ //CONTROL-C (COPY)

                        return true;
                    }
                    // ANY OTHER KEY
                    let key = app::event_text();
                    shell.append(&key);
                    shell.query.push_str(&key);
                    true
                }
            },
            _ => false, //any other event that is not needed
        }
    });

    shell_clone.set_callback(|| println!("EEE"));

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