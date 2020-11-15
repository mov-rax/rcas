use fltk::{app, app::App, text::*, window::*, table::*};
//use std::ops::{Deref, DerefMut};
use crate::rcas_lib::{*, RCas, CalculationMode};
use std::ops::{Deref, DerefMut};
use fltk::browser::{BrowserScrollbar, Browser, MultiBrowser, FileBrowser};
use fltk::group::{Tabs, Group};
use fltk::image::{PngImage as FltkImage, SvgImage};
use fltk::frame::Frame;
use std::collections::HashMap;
//use fltk_sys::widget::Fl_Widget;
use fltk::menu::MenuItem;
use fltk::dialog::{FileDialog, FileDialogType, FileDialogOptions, HelpDialog, BeepType};
use std::fs::File;
use std::io::Write;
use crate::data::BakedData;


#[derive(Debug, Clone)]
pub(crate) struct Shell{
    term: SimpleTerminal,
    pub(crate) mode: CalculationMode,
    pub(crate) query: String,
    history:Vec<String>,
    history_pos:usize
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
            query: String::new(),
            history: Vec::new(),
            history_pos: 0
        };
        shell.append(&shell.mode.to_string());
        shell
    }

    pub(crate) fn append(&mut self, text: &str) { self.term.append(text); }

    pub fn renew_query(&mut self){
        let query_copy = self.query.clone();
        self.history.push(query_copy);
        self.query.clear();
        self.history_pos = self.history.len();
    }


    pub fn older_history(&mut self) -> String{
        if self.history_pos > 0{
            self.history_pos -= 1;
        } else {
            return self.history[0].clone();
        }
        self.history[self.history_pos].clone()
    }

    //makes you go up history, ya know?
    pub fn newer_history(&mut self) -> String{
        if self.history_pos < self.history.len()-1{
            self.history_pos += 1;
        } else{
            return "".to_string();
        }
        self.history[self.history_pos].clone()
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

#[derive(Debug, Clone)]
pub struct PlotViewer{
    env: Tabs,
    pub img_locations: HashMap<String, (i32,i32,i32,i32)> //x,y,width,height
}

impl PlotViewer{
    pub fn new(x:i32,y:i32,width:i32,height:i32,title:&str) -> Self{
        let mut env = Tabs::new(x,y,width,height,title);
        env.set_tab_align(Align::Center);
        PlotViewer {env, img_locations: HashMap::new()}
    }

    /// Used only for testing
    pub fn add_dummy_tab(&mut self, label:&str){
        let (x,y,width,height) = self.client_area();
        let mut dummy = Group::new(x,y-30,width,height, label);
        dummy.end();
    }

    pub fn add_test_img_tab(&mut self, label:&str){
        let (x, y) = self.get_base_coords_image();
        let mut dummy = self.gen_tab(label);
        let mut img = fltk::image::SvgImage::from_data(BakedData::get_test_svg()).unwrap();
        //let mut img = fltk::image::PngImage::from_data(&BakedData::get_test_png()).unwrap();
        img.scale(dummy.width(), ((dummy.height() as f32)*0.93).round() as i32, true, true);
        self.img_locations.insert(String::from(label), (dummy.x(), dummy.y(), dummy.width(), ((dummy.height() as f32)*0.93).round() as i32));
        //println!("dummy: {:?}", (dummy.width(),dummy.height()));
        //println!("img: {:?}", (img.width(),img.height()));

        let mut frame = Frame::new(x,y,dummy.width(),dummy.height(),"");
        frame.set_image(Some(img));
        dummy.end();
    }

    pub fn add_dummy_tab_with_text(&mut self, tab_label:&str, text:&str){
        let (x,y) = self.get_base_coords_text();
        let mut dummy = self.gen_tab(tab_label);
        let mut frame = Frame::new(x,y,10,2,text);
        dummy.end();
    }

    /// Used to get the coordinates for inserting text into a PlotViewer tab
    pub fn get_base_coords_text(&self) -> (i32, i32){
        (self.x()+20,self.y()+15)
    }

    /// Used to get the coordinates for inserting images into a PlotViewer tab
    pub fn get_base_coords_image(&self) -> (i32,i32){
        (self.x(),self.y()+5)
    }

    /// Generates a new group that conforms to PlotViewer tab
    fn gen_tab(&mut self, label:&str) -> Group{
        let (x,y,width,height) = self.client_area();
        Group::new(x,y-30,width,height,label)
    }

    pub fn save_visible_plot_prompt(&mut self){
        if let Some(group) = self.value(){
            if let Some(widget) = group.child(0){
                let ptr = unsafe {widget.as_widget_ptr()};
                let frame = unsafe {Frame::from_widget_ptr(ptr)};
                if let Some(image) = frame.image(){ // image is a Box<dyn ImageExt>
                    let ptr = unsafe {image.as_image_ptr()}; // turns that pesky box into a pointer to an image
                    let svg_image = unsafe{ SvgImage::from_image_ptr(ptr)}; // THIS IS THE IMAGE!!!! :)
                    let mut svg_img = svg_image.copy(); // Done for SAFETY reasons. We don't want the user removing the image from under us!! (Also, copy() does a deep copy of the image, unlike clone())
                    let mut dialog = FileDialog::new(FileDialogType::BrowseSaveFile);
                    dialog.set_title("Save Plot as...");
                    dialog.set_option(FileDialogOptions::SaveAsConfirm);
                    dialog.set_filter("PNG Image\t*.png\nJPEG Image\t*.{jpg,jpeg}\nSVG Image\t*.svg\nGIF Image\t*.gif\nBITMAP Image\t*.bmp");
                    dialog.set_preset_file("*.png");
                    dialog.show();
                    //fltk::dialog::alert(300,200,"This is a test of the ALERT system. All is fine.");
                    if let Some(error) = dialog.error_message(){
                        if error != "No error".to_string(){ // if there was an error
                            println!("ERROR: {}", error);
                            fltk::dialog::beep(BeepType::Error);
                            fltk::dialog::alert(300,200,&format!("ERROR: {}", error));
                        } else { // there was no error. PROCEED THE PLOT-SAVING PROCESS :)
                            println!("FILENAME:\t{:?}", dialog.filenames());
                            let path = dialog.filename().to_string_lossy().to_string();

                            // let mut opt = usvg::Options::default();
                            // opt.path = Some(dialog.filename().clone());
                            // opt.fontdb.load_system_fonts();
                            // let tree = usvg::Tree::from_str()


                            // let mut image_converted:Option<*const *const u8> = None;
                            // let mut image_type = String::new();
                            // let mut data_size:Option<u32> = None;
                            // if path.contains(".svg"){
                            //     image_converted = Some(svg_img.to_raw_data());
                            //     image_type.push_str(".str");
                            //     data_size = Some(svg_img.data_w()* svg_img.data_h());
                            // } else if path.contains(".jpg") || path.contains(".jpeg"){
                            //     let image = svg_img.into_jpeg().unwrap();
                            //     image_converted = Some(image.to_raw_data());
                            //     image_type.push_str(".jpeg");
                            //     data_size = Some(image.data_w()*image.data_h());
                            // } else if path.contains(".gif"){ // GIF WILL BE BMP FOR THE TIME BEING
                            //     let image = svg_img.into_bmp().unwrap();
                            //     image_converted = Some(image.to_raw_data());
                            //     image_type.push_str(".bmp");
                            //     data_size = Some(image.data_w()*image.data_h());
                            // } else if path.contains(".bmp"){
                            //     let image = svg_img.into_bmp().unwrap();
                            //     image_converted = Some(image.to_raw_data());
                            //     image_type.push_str(".bmp");
                            //     data_size = Some(image.data_w()*image.data_h());
                            // } else { //If the type is not defined by the user, or if .png is given to the user, then it will default to png
                            //     let image = svg_img.into_png().unwrap();
                            //     image_converted = Some(image.to_raw_data());
                            //     image_type.push_str(".png");
                            //     data_size = Some(image.data_w()*image.data_h());
                            // }
                            // if let Some(raw) = image_converted{
                            //     let size = data_size.unwrap();
                            //     println!("DATA: {:?}", raw);
                            //
                            // }
                            //let mut file = File::create(dialog.filename().to_string_lossy().to_string()); // Tries to create a new file
                            //if let Ok(file) = file{ // If file was successfully created, then we write to the file :)

                            }
                        }
                    }
                }
            }
        }
    }

impl Deref for PlotViewer{
    type Target = Tabs;
    fn deref(&self) -> &Self::Target {&self.env}
}

impl DerefMut for PlotViewer{
    fn deref_mut(&mut self) -> &mut Self::Target {&mut self.env}
}
