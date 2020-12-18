use fltk::{app, app::App, text::*, window::*, table::*};
//use std::ops::{Deref, DerefMut};
use crate::rcas_lib::{*, RCas, CalculationMode};
use std::ops::{Deref, DerefMut, Range};
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
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::any::Any;
use fltk::input::{Input, FloatInput};
use fxhash::FxHashMap;

const COLOR_SELECTED_FILL:u32 = 0xB7C6E0;
const COLOR_SELECTED_BORDER:u32 = 0x0F0F0F;
const COLOR_UNSELECTED_BORDER:u32 = 0xC0C0C0;

#[derive(Debug, Clone)]
pub(crate) struct Shell{
    term: TextDisplay,
    sbuf: TextBuffer,
    pub(crate) mode: CalculationMode,
    pub(crate) query: String,
    history:Vec<String>, // holds the query history
    history_pos:usize, // location in history
    root_query_pos:u32, // holds root, or rather, the lowest index that is editable by the user
    pub( crate) cursor_pos:u32, //holds position of cursor
}

pub trait AsTerm {
    fn append(&mut self, txt: &str);
    fn text(&self) -> String;
}

impl AsTerm for TextDisplay {
    fn append(&mut self, txt: &str) {
        self.buffer().unwrap().append(txt);
        self.set_insert_position(self.buffer().unwrap().length());
        self.scroll(
            self.count_lines(0, self.buffer().unwrap().length(), true),
            0,
        );
    }
    fn text(&self) -> String {
        self.buffer().unwrap().text()
    }
}

impl Shell{
    /// Creates a new shell.
    pub fn new(x:i32,y:i32,length:i32,width:i32) -> Shell {
        let mode = CalculationMode::Radian; // default is set to radian
        let mut term = TextDisplay::new(x,y, length, width, ""); // Terminal
        let mut buf = TextBuffer::default();
        let mut sbuf = TextBuffer::default();
        let styles = vec![
            StyleTableEntry { // MODE COLOR
                color: Color::from_u32(0xEA4E95),
                font: Font::CourierBold,
                size: 16,
            },
            StyleTableEntry { // ERROR COLOR
                color: Color::Red,
                font: Font::CourierBoldItalic,
                size: 16,
            },
            StyleTableEntry { // TEXT ENTRY COLOR
                color: Color::from_u32(0x3F75EA),
                font: Font::CourierItalic,
                size: 16,
            },
            StyleTableEntry { // TEXT RESULT COLOR
                color: Color::White,
                font: Font::Courier,
                size: 16,
            }
        ];
        
        term.set_buffer(Some(buf));
        term.set_highlight_data(sbuf.clone(), styles);
        term.set_cursor_style(TextCursor::Caret);
        term.set_cursor_color(Color::Blue);
        term.show_cursor(true);
        term.set_color(Color::from_u32(0x212121));
        term.set_frame(FrameType::ShadowFrame);
        //term.set_ansi(true);

        //println!("{:?}", term.color());

        let mut shell = Shell {
            term,
            sbuf,
            mode,
            query: String::new(),
            history: Vec::new(),
            history_pos: 0,
            root_query_pos: 0,
            cursor_pos: 0,
        };

        shell.append_mode();
        shell
    }

    pub fn text(&self) -> String {
        self.term.text()
    }

    pub fn append(&mut self, txt: &str) {
        self.term.append(txt);
        self.cursor_pos = self.term.insert_position();
    }

    pub fn remove_query(&mut self){
        let len = self.text().len() as u32;
        let query_len = self.query.len() as u32;
        self.cursor_pos = self.term.insert_position();
        self.buffer().unwrap().remove(len-query_len, len);
        self.sbuf.remove(len-query_len, len);
        self.query.clear();
    }

    pub fn append_mode(&mut self) {
        self.term.append(&self.mode.to_string());
        self.sbuf.append(&"A".repeat(self.mode.to_string().len())); // uses the A style (the first one)
        self.root_query_pos = self.term.text().len() as u32; // saves the position where the cursor is after printing out the mode
    }

    pub fn append_error(&mut self, text: &str){
        self.term.append("\n");
        self.term.append(text);
        self.term.append("\n");
        self.sbuf.append(&"B".repeat(text.len()+2)); // uses the B style (the second one)
        self.cursor_pos = self.insert_position();
    }

    pub fn insert_normal(&mut self, text: &str) {
        self.term.insert(text);
        //self.term.append(text);
        self.sbuf.insert(self.term.insert_position(), &"C".repeat(text.len()));
        self.cursor_pos = self.term.insert_position();
    }

    pub fn append_answer(&mut self, text: &str) {
        self.term.append("\n");
        self.term.append(text);
        self.sbuf.append(&"D".repeat(text.len()+1));
    }

    /// Removes a character at the cursor
    pub fn remove_at_cursor(&mut self) {
        let pos:usize = self.term.insert_position() as usize;
        let query_pos = pos-self.root_query_pos as usize;
        self.query.remove(query_pos-1);
        let pos:u32 = self.term.insert_position();
        self.buffer().unwrap().remove(pos - 1, pos);
        self.sbuf.remove(pos - 1, pos);
        self.cursor_pos = self.term.insert_position();
    }

    pub fn renew_query(&mut self){
        let query_copy = self.query.clone();
        self.history.push(query_copy);
        self.query.clear();
        self.history_pos = self.history.len();
        self.cursor_pos = self.term.insert_position();
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

    /// Doesn't work for some reason
    pub fn safe_move_cursor_left(&mut self){
        if self.term.insert_position() > self.root_query_pos{
            let result = match self.term.move_left() {
                Ok(()) => "SHOULD MOVE LEFT",
                Err(_) => "CANNOT MOVE LEFT"
            };
            println!("{}", result);
        }
        self.cursor_pos = self.term.insert_position();
    }

    /// Also doesn't work for some reason
    pub fn safe_move_cursor_right(&mut self){
        if self.term.insert_position() < self.term.text().len() as u32{
            self.term.move_right();
            println!("right");
        }
        self.cursor_pos = self.term.insert_position();
    }

    /// Sets the Shell's calculation mode.
    pub fn set_calculation_mode(&mut self, mode:CalculationMode) {
        self.mode = mode;
    }

    pub fn clear(&mut self){
        self.buffer().unwrap().set_text(""); //clears the shell
        self.sbuf.set_text(""); // clears the style buffer
    }

    pub fn fix_cursor(&mut self){
        let pos = self.cursor_pos;
        self.set_insert_position(pos);
    }
}

impl Deref for Shell{
    type Target = TextDisplay;
    fn deref(&self) -> &Self::Target {&self.term}
}

impl DerefMut for Shell{
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.term}
}
#[derive(Debug, Clone)]
pub( crate) struct EnvironmentTable{
    env: MultiBrowser,
    pub lines: Rc<RefCell<u32>>
}

impl EnvironmentTable{
    pub fn new(x:i32, y:i32, width:i32, height:i32, title:&str) -> EnvironmentTable{
        let env = MultiBrowser::new(x,y,width,height,title);
        EnvironmentTable {env, lines:Rc::from(RefCell::from(0))}
    }

    /// Adds an item onto the Environment Table with a given identifier.
    pub(crate) fn add(&mut self, id:String){
        self.env.add(&id);
        let mut lines = self.lines.borrow_mut();
        *lines += 1;
    }

    pub fn add_type(&mut self, id:&str, _type:&str){
        //let tabs = (0..self.env.width()).step_by(45).map(|_| '\t').collect::<String>();
        let tabs = " | ".to_string();
        //let separator = "\t|\t";
        let mut string = id.to_string();
        string.push_str(&tabs);
        //string.push_str(separator);
        string.push_str(_type);
        self.env.add(&string);
        let mut lines = self.lines.borrow_mut();
        *lines += 1;
    }

    /// Safely removes an item on the Environment Table when given its identifier.
    pub fn remove(&mut self, id:String){
        let mut lines = self.lines.borrow_mut();
        for i in 0..*lines{
            let text = self.text(i+1);
            if let Some(text) = text{
                if text.contains(&id){
                    self.env.remove(i+1);
                }
            }
        }
        *lines -= 1;
    }
    /// Removes a row given a number. Row starts at 1.
    pub fn remove_row(&mut self, row:u32, rcas_environment: Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>){
        if let Some(text) = self.env.text(row){
            let mut rcas_environment = rcas_environment.borrow_mut();
            let text:String = text;
            let text = text.chars().take_while(|c| *c != ' ').collect::<String>();
            rcas_environment.remove(&*text);
            self.env.remove(row);
        }
    }

    /// Returns the selected Row (if any selected)
    pub fn get_selected(&self) -> Option<u32>{
        let lines = *self.lines.borrow();
        for i in 0..lines{
            if self.selected(i+1){
                return Some(i+1);
            }
        }
        None
    }

    pub fn clear_table(&mut self){
        let mut lines = self.lines.borrow_mut();
        self.env.clear();
        *lines = 0;
    }

    pub fn update_table(&mut self, internal:Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>){
        self.clear_table(); //clears the table of all values, and resets the line count back to 0
        let internal = internal.borrow();
        // this gets all of necessary information from the environment table
        let values = internal.iter().filter_map(|(k,v)| {
            let data_type = v.iter().map(|x| x.get_value()).collect::<String>();
            Some((k.clone(), data_type))
        }).collect::<Vec<(String,String)>>();
        // adds each value that is in the internal environment table to the GUI
        for (id, _type) in values{
            self.add_type(&*id, &*_type);
        }
        self.env.sort();
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
    pub img_locations: HashMap<String, (i32,i32,i32,i32)>, //x,y,width,height
    tabs: Vec<Group>
}

impl PlotViewer{
    pub fn new(x:i32,y:i32,width:i32,height:i32,title:&str) -> Self{
        let mut env = Tabs::new(x,y,width,height,title);
        env.set_tab_align(Align::Center);
        PlotViewer {env, img_locations: HashMap::new(), tabs: Vec::new()}
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
        self.tabs.push(dummy); // keeps track of all the tabs
    }

    pub fn remove_visible_tab(&mut self){
        let mut env = self.env.clone(); // done because borrow checker reasons, cannot find value() with &mut, and there was already a borrow.
        self.tabs = (0..self.tabs.len()).filter_map(|i| {
            if unsafe {self.tabs[i].as_widget_ptr() as u64 != env.value().unwrap().as_widget_ptr() as u64}{ // unsafe required to find the tab efficiently.
                return Some(self.tabs[i].clone());
            } else {
                println!("REMOVING PLOT ON INDEX {}", i);
            }
            None
        }).collect(); // this iteration removes from the tabs vec the plot that is about to be removed.
        app::delete_widget(self.value().unwrap());
        self.redraw();
    }

    pub fn resize_image(&mut self){
        for i in 0..self.tabs.len(){
            if let Some(frame) = self.tabs[i].child(0){
                let ptr = unsafe{frame.as_widget_ptr()};
                let frame = unsafe {Frame::from_widget_ptr(ptr)};
                if let Some(image) = frame.image(){
                    let ptr = unsafe {image.as_image_ptr()};
                    let mut svg_image = unsafe{ SvgImage::from_image_ptr(ptr)}; // THIS IS THE IMAGE!!!! :)
                    svg_image.scale(self.tabs[i].width(), ((self.tabs[i].height() as f32)*0.93).round() as i32, true, true);
                }

            }
        }
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

                            // TODO - THE BACKEND NEEDS TO BE FLESHED OUT FIRST. THIS CANNOT BE FINISHED MADE UNTIL THEN.
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

// a = b.clone();
// c = b.clone();

pub struct MatrixView{
    ranges: Rc<RefCell<Option<(Range<i32>, Range<i32>)>>>, // An optional range for (x,y)
    table: Table,
    input: FloatInput
// TODO - Make it work with a mutable reference to a matrix
}

impl MatrixView{
    pub fn new(title:&str) -> Self{
        let mut win = DoubleWindow::default()
            .with_size(600, 300)
            .center_screen()
            .with_label(title);
        let mut table:Table = Table::new(0,0,win.width(),win.height(), "");
        let mut input:FloatInput = FloatInput::new(0,0,0,0,"");
        input.set_text_font(Font::Courier);
        input.set_frame(FrameType::BorderFrame);
        table.set_rows(50);
        table.set_row_header(true);
        table.set_cols(50);
        table.set_col_header(true);
        table.set_col_width_all(80);
        table.set_col_resize(true);
        table.set_color(Color::White);
        table.end();
        win.set_color(Color::White);
        win.end();
        win.make_resizable(true);
        win.show();


        Self {ranges: Rc::from(RefCell::from(None)), table, input}
    }

    pub fn show(&mut self){
        let mut table = self.table.clone();
        let wrapped = self.ranges.clone();
        table.draw_cell2(move |table, table_context, row, col, x, y, w, h| match table_context {
            TableContext::StartPage => fltk::draw::set_font(Font::Courier, 14),
            TableContext::ColHeader => {
                Self::draw_header(&((col + 65) as u8 as char).to_string(), x, y, w, h);
            }
            TableContext::RowHeader => Self::draw_header(&format!("{}", row+1), x, y, w, h),
            TableContext::Cell => {
                Self::draw_data(
                    wrapped.borrow_mut(),
                    &format!("{}", row + col),
                    x, y, w, h,
                    table.is_selected(row, col),
                );

            },
            _ => {}
        });
    }
    /// Taken from fltk-rs table.rs example.
    fn draw_header(txt: &str, x: i32, y: i32, w: i32, h: i32){
        fltk::draw::push_clip(x,y,w,h);
        fltk::draw::draw_box(FrameType::ThinUpBox, x, y, w, h, Color::from_u32(0xF8F8F9));
        fltk::draw::set_draw_color(Color::Black);
        fltk::draw::draw_text2(txt, x, y, w, h, Align::Center);
        fltk::draw::pop_clip();
    }


    fn draw_data(mut ranges: RefMut<Option<(Range<i32>, Range<i32>)>>, txt: &str, x:i32, y:i32, w: i32, h: i32, selected:bool) {
        fltk::draw::push_clip(x, y, w, h);

        if app::event() == Event::Push{ //if you click anywhere, the ranges are reset and they are no longer colored :)
            *ranges = None;
        }

        if app::event() == Event::Drag || app::event() == Event::MouseWheel{ // occurs when you drag your mouse or use your mouse wheel
            if *ranges == None{
                *ranges = Some(((x..x), (y..y))); // creating a range
            } else {
                let (xr, yr) = ranges.as_ref().unwrap();
                *ranges = Some(((xr.start..x), (yr.start..y))); // extending the range :)
            }
            fltk::draw::set_draw_color(Color::from_u32(COLOR_SELECTED_FILL));
        } else if let Some((xr, yr)) = ranges.as_ref(){
            if xr.contains(&x) && yr.contains(&y){
                fltk::draw::set_draw_color(Color::from_u32(COLOR_SELECTED_FILL));
            } else { // if coords are not in ranges, then white it is
                fltk::draw::set_draw_color(Color::White);
            }
        } else { // if ranges don't exist, then white it is
            fltk::draw::set_draw_color(Color::White);
        }
        fltk::draw::draw_rectf(x, y, w, h); //draws the filled rectangle
        if selected {
            fltk::draw::set_draw_color(Color::from_u32(COLOR_SELECTED_BORDER));
        } else {
            fltk::draw::set_draw_color(Color::White);
            fltk::draw::draw_rectf(x,y,w,h);
            fltk::draw::set_draw_color(Color::from_u32(COLOR_UNSELECTED_BORDER));
        }
        fltk::draw::draw_rect(x, y, w, h);
        fltk::draw::set_draw_color(Color::Black);
        fltk::draw::draw_text2(txt, x-10, y, w, h, Align::Right);
        fltk::draw::pop_clip();
    }
}

impl Deref for MatrixView{
    type Target = Table;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl DerefMut for MatrixView{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.table
    }
}

