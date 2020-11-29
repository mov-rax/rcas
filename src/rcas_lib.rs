use rust_decimal::*;
use std::io::ErrorKind;
use std::fmt;
use crate::rcas_lib::SmartValue::Operator;
use std::str::FromStr;
use std::error;
use rust_decimal::prelude::ToPrimitive;
use std::ptr::replace;
use std::ops::Deref;
use crate::rcas_functions;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use std::fmt::{Display, Debug, Formatter};

//constants
const ADD:char = '+'; //addition
const SUB:char = '-'; //subtraction
const MUL:char = '*'; //multiplication
const DIV:char = '/'; //division
const MOD:char = '%'; //modulo
const POW:char = '^'; //power
const FAC:char = '!'; //factorial
const PHD:char = '█'; //placeholder
const FNC:char = 'ƒ'; //pre-defined (constant) function
const FNV:char = '⭒'; //variable function
const VAR:char = '⭑'; //variable
const PAR:char = '⮂'; //parameters
static SYM:&str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; //allowed symbols
static NUM:&str = "1234567890."; //allowed numbers
static OPR:&str = "+-*/"; //allowed operators

//Errors

#[derive(Debug, Clone, PartialEq)]
struct ParenthesesError{
    positive:bool, //too much or too little parentheses. true == too much, false == too little
    position:u32,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FormattingError{
    pub position:u32
}
#[derive(Debug, Clone, PartialEq)]
struct GenericError;
#[derive(Debug, Clone, PartialEq)]
struct UnknownIdentifierError{
    position:u32,
    identifier:String,
}
#[derive(Debug,Clone,PartialEq)]
pub struct TypeMismatchError;

#[derive(Debug,Clone,PartialEq)]
pub struct NegativeNumberError;

#[derive(Debug,Clone,PartialEq)]
pub struct DivideByZeroError;

#[derive(Debug,Clone,PartialEq)]
pub struct IncorrectNumberOfArgumentsError<'a>{
    pub name: &'a str,
    pub found: usize,
    pub requires: usize,
}

#[derive(Debug,Clone,PartialEq)]
pub struct OverflowError;

impl fmt::Display for OverflowError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "OVERFLOW ERROR.")
    }
}

impl<'a> fmt::Display for IncorrectNumberOfArgumentsError<'a>{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"INCORRECT NUMBER OF ARGUMENTS IN FUNCTION '{}'.\nFOUND {} ARGUMENTS.\nREQUIRES {} ARGUMENTS.", self.name, self.found, self.requires)
    }
}

impl fmt::Display for TypeMismatchError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "TYPE MISMATCH ERROR.")
    }
}

impl fmt::Display for ParenthesesError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PARENTHESES ERROR AT {}", self.position)
    }
}
impl fmt::Display for FormattingError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FORMATTING ERROR AT {}", self.position)
    }
}
impl fmt::Display for GenericError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "GENERIC ERROR")
    }
}
impl fmt::Display for UnknownIdentifierError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UNKNOWN IDENTIFIER {} AT {}",&self.identifier, self.position)
    }
}

impl fmt::Display for NegativeNumberError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NEGATIVE NUMBER ERROR")
    }
}

impl fmt::Display for DivideByZeroError{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "DIVIDE BY ZERO ERROR")
    }
}


impl error::Error for ParenthesesError{}
impl error::Error for FormattingError{}
impl error::Error for GenericError{}
impl error::Error for UnknownIdentifierError{}
impl error::Error for TypeMismatchError{}
impl error::Error for NegativeNumberError{}
impl error::Error for DivideByZeroError{}
impl<'a> error::Error for IncorrectNumberOfArgumentsError<'a>{}
impl error::Error for OverflowError{}

pub struct RCas{}

impl RCas{

    pub fn new() -> RCas{
        RCas {}
    }

    pub fn query(&self, input:&str) -> QueryResult {
        let time = Instant::now();
        match parser(input) {
            Ok(parsed) => {
                let time = time.elapsed().as_micros();
                let mut wrapper = Wrapper::compose(parsed);
                wrapper.solve();
                let result = wrapper.to_result();
                println!("PARSE TIME:\t {} µs", time);
                result
            },
            Err(error) => {
                Self::error_handle(error)
            }
        }
    }

    fn error_handle(err: Box<dyn std::error::Error>) -> QueryResult{
        println!("Parsing Error :(");
        let error = err.deref();
        let mut info = String::new();
        //time to try all types of errors :)
        if let Some(error) = error.downcast_ref::<ParenthesesError>(){

            let add_or_remove = { // returns REMOVING or ADDING depending on whether it is suggested to add or remove a parentheses
                let mut option = "REMOVING";
                if error.positive {
                    option = "ADDING";
                }
                option
            };

            info = format!("PARENTHESES ERROR detected at character {}. Have you tried {} \
                    a parentheses?", &error.position, add_or_remove);
        }

        if let Some(error) = error.downcast_ref::<FormattingError>(){
            info = format!("FORMATTING ERROR detected at character {}.", &error.position);
        }

        if let Some(error) = error.downcast_ref::<GenericError>(){
            info = format!("GENERIC ERROR detected. Please report what was done for this \
                    to appear. Thanks!");
        }

        if let Some(error) = error.downcast_ref::<UnknownIdentifierError>(){
            info = format!("UNKNOWN IDENTIFIER detected at character {}. {} is NOT A VALID \
                    variable or function name.", &error.position, &error.identifier);
        }
        QueryResult::Error(info)
    }
}



pub enum QueryResult{
    Simple(String), // Common arithmetic query results will appear here
    Assign(QueryAssign), // Query result that assigns a value to a variable or function identifier
    Image(QueryImage), // Query result that returns an Image
    Execute(Command), // Query result that requires the GUI to execute
    Error(String) // Returned in case of parsing or function error
}
/// Commands that interface with the GUI.
#[derive(Debug, PartialEq, Clone)]
pub enum Command{
    ClearScreen, //cls()
    RemoveCurrentPlot, //clear("current")
    RemovePlots, //clear("*")
    ClearEnvironment, //clear("env")
    ClearAll, //clear("all")
    SavePlot, //saveplot("magic.png")
}
/// A structure that contains a raster (PNG) and vector (SVG) versions of a plot.
/// The vector is used for displaying a plot in high-resolution.
/// The raster is used in case of saving the plot to any format.
pub struct QueryImage{
    pub raster: Vec<u8>, // raster image (A PNG FILE)
    pub vector: String, //  vector iamge (AN SVG FILE)
}
/// A structure that contains the identifier to data and the data itself.
pub struct QueryAssign{
    pub id: String,
    pub data: DataType,
}

/// Used to facilitate the transfer of information.
pub enum DataType{
    Number(Decimal), // A Number
    Matrix(Rc<RefCell<SmartMatrix>>), // A reference to a vector of numbers (A reference is used to avoid copying data)
    Image(QueryImage) // Assigning to a query image
}

pub struct SmartMatrix{
    data: Vec<Decimal>,
    rows: u64,
    columns: u64,
}

#[derive(PartialEq, Clone)]
pub struct Wrapper{
    values: Vec<SmartValue>,
}

impl Wrapper{
    pub fn new() -> Self{
        Wrapper {values:Vec::new()}
    }
    ///Composes a Wrapper.
    pub fn compose(input: Vec<SmartValue>) -> Self{
        Wrapper {values: composer(input)}
    }
    ///Solves a Wrapper.
    pub fn solve(&mut self){
        self.values = recurse_solve(self.values.clone())
    }

    pub fn print_raw(&self){
        print_sv_vec(&self.values);
    }

    pub fn to_string(&self) -> String{ sv_vec_string(&self.values) }

    pub fn to_result(&self) -> QueryResult{
        if self.values.len() == 1{

            return match &self.values[0]{
                SmartValue::Error(err) => QueryResult::Error(err.clone()),
                SmartValue::Cmd(cmd) => QueryResult::Execute(cmd.clone()),
                _ => QueryResult::Simple(self.to_string()),
            };

        } else if self.values.len() == 0{
            return QueryResult::Simple("".to_string())
        }
        return QueryResult::Error("FUNCTION ERROR".to_string())
    }
}


#[derive(Debug, PartialEq, Clone)]
pub enum SmartValue{
    Operator(char),
    Function(String), //holds a function identifier
    Number(Decimal),
    LParen,
    RParen,
    //Variable(String), //variable does not require ( ), therefore there exists Function and Variable
    Placeholder(Vec<SmartValue>),
    Range(Decimal,Decimal,Decimal), // Lower, Step, Upper
    Label(String,Vec<SmartValue>), // A label can contains an identifier and a possible expression.
    Comma, // A special character used to separate parameters
    Assign, // A special equals = operator. It is used to show that a value is being assigned to an identifier.
    Error(String), // An error returned from a function call. The Stringified version of the error is returned.
    Cmd(Command), // A command that is returned from a function call. May exist when solving
}

impl SmartValue{
    pub fn get_value(&self) -> String{
        let mut buf = String::new();
        match self{
            SmartValue::Operator(x) => buf.push(*x),
            SmartValue::Function(_) => buf.push(FNC),
            SmartValue::Number(x) => {
                let num = format!("{}", x);
                for sus in num.chars(){
                    buf.push(sus)
                }

            },
            SmartValue::LParen => buf.push('('),
            SmartValue::RParen => buf.push(')'),
            SmartValue::Placeholder(_) => buf.push(PHD),
            //SmartValue::Variable(_) => buf.push(VAR),
            _ => buf.push('?')
        }
        buf
    }
}


#[derive(Debug,Clone)]
pub enum CalculationMode{
    Radian,
    Degree
}

impl CalculationMode{
    pub fn to_string(&self) -> String{
        match &self{
            CalculationMode::Radian => format!("RAD -> "),
            CalculationMode::Degree => format!("DEG -> ")
        }
    }

}

fn expression_clone(input:&Vec<SmartValue>, lower:usize, upper:usize) -> Vec<SmartValue>{
    let mut clone = Vec::new();
    clone.clone_from_slice(&input[lower..upper]);
    clone
}

fn recurse_solve(mut input:Vec<SmartValue>) -> Vec<SmartValue>{
    //print_sv_vec(&input);
    return if has_placeholder(&input) {
        for x in 0..input.len() {
            if let Some(SmartValue::Placeholder(holder)) = input.get(x) {
                //print_sv_vec(&holder);
                let solved = recurse_solve(holder.clone());
                if let Some(SmartValue::Function(_)) = input.get(x-1){
                    //NOTHING HERE.
                } else if solved.len() == 0 { // if the solve returned NOTHING, and there is no function before this placeholder, then the entire answer must be nothing.
                    return Vec::new()
                }
                // a nice way to get the resolved value. If the previous value is a not function, then it is just a Number.
                // Otherwise, it is probably parameters to a function, and as such should be in
                // A placeholder.
                let value = if let Some(SmartValue::Function(_)) = input.get(x-1){ // if previous was a function, then put the solution in a placeholder.
                    SmartValue::Placeholder(solved)
                } else {
                   solved[0].clone()
                };
                input[x] = value;
            }
            //println!("X: {}", &input[x].get_value());
        }
        get_calculated(input)
    } else {
        get_calculated(input)
    }
}


fn has_function(input:&Vec<SmartValue>) -> bool{
    let mut value = false;
    for x in input{
        if let SmartValue::Function(_) = x{
            value = true;
        }
    }
    value
}

fn has_placeholder(input:&Vec<SmartValue>) -> bool{
    for x in input{
        if let SmartValue::Placeholder(_) = x{
            return true;
        }
    }
    false
}

fn recurse_check_paren(input:&Vec<SmartValue>, left:usize, right:usize, counter:u32) -> (usize, usize)
{
    if left < input.len() && right < input.len(){
        if input[left] != SmartValue::LParen{
            return recurse_check_paren(input, left + 1, left + 1, counter)
        }
        if input[right] == SmartValue::LParen{
            return recurse_check_paren(input, left, right + 1, counter + 1)
        }
        if input[right] == SmartValue::RParen && counter != 0{ //subtracts one from counter
            return recurse_check_paren(input, left, right + 1, counter - 1)
        }
        if counter != 0{ //advances
            return recurse_check_paren(input, left, right + 1, counter)
        }
    }

    //returns left and right if counter is 0, and both left and right are on parentheses
    (left, right)
}

fn get_outermost_parentheses(input:&Vec<SmartValue>) -> (usize,usize){
    let mut left = 0;
    let mut right= 0;
    let mut counter = 0;
    let mut found_left = false;

    for x in 0..input.len(){ //gets leftmost
        if input[x] == SmartValue::LParen && !found_left{
            left = x;
            found_left = true;
        }
        if input[x] == SmartValue::LParen{
            counter += 1;
        }
        if input[x] == SmartValue::RParen{
            counter -= 1;
            if counter == 0{ // in case this is the last loop and this is the outermost parentheses.
                right = x;
            }
        }
        if input[x] == SmartValue::RParen && counter == 0{
            right = x;
            break;
        }
    }
    (left, right)
}

fn number_of_parentheses_sections(input: &Vec<SmartValue>) -> usize{
    input.iter().filter(|x| **x == SmartValue::LParen).count()
}

//for debugging parser result
pub fn print_sv_vec(sv:&Vec<SmartValue>){
    let mut buf = String::new();
    for value in sv{
        buf.push_str(value.get_value().as_str());
        //buf.push('|');
    }
    println!("{}", buf)
}

/// Converts a &Vec<SmartValue> to a String
pub fn sv_vec_string(sv:&Vec<SmartValue>) -> String {
    let mut buf = String::new();
    for value in sv{
        buf.push_str(value.get_value().as_str());
    }
    buf
}

///Takes a Vec<SmartValue> and composes it such that no LParen or RParen
/// exists. Sections wrapped by parentheses are stored in a series of
/// SmartValue::Placeholder
pub fn composer(mut input: Vec<SmartValue>) -> Vec<SmartValue>{
    let mut placeholder_locations:Vec<usize> = Vec::new();
    let sections = number_of_parentheses_sections(&input);
    if sections == 0{ //no parentheses, therefore, no need to compose.
        return input;
    } else {
        //This will replace all parentheses within the same depth.
        for _ in 0..sections{
            let parentheses_locations = get_outermost_parentheses(&input);
            //gets value inside first parentheses and puts it into a Placeholder
            if parentheses_locations.0 == 0 && parentheses_locations.1 == 0{
                break;
            }
            let subsection = input[parentheses_locations.0+1 .. parentheses_locations.1].to_vec();

            let placeholder = SmartValue::Placeholder(composer(subsection));
            placeholder_locations.push(parentheses_locations.0);
            for _ in parentheses_locations.0 ..parentheses_locations.1{
                input.remove(parentheses_locations.0);
            }

            if parentheses_locations.0 == input.len(){
                input.push(placeholder)
            } else{
                //input.insert(parentheses_locations.0, placeholder);
                input[parentheses_locations.0] = placeholder;
            }
        }

        for location in placeholder_locations{
            if let SmartValue::Placeholder(mut subsection) = input[location].clone(){
                subsection = composer(subsection); //does it all over again :)
            }
        }

        input //the magic has been done.
    }
}

fn get_calculated(input: Vec<SmartValue>) -> Vec<SmartValue>{
    let mut input = input;
    calculate(&mut input);
    input
}
///Only takes slices with NO PARENTHESES.
pub fn calculate(input: &mut Vec<SmartValue>){
    let mut count:usize = 0;
    let mut last_comma_location:usize = 0;

    // does magic with Vec<SmartValue> that have Commas, i.e., are parameters in functions
    loop{
        if input.get(count) == None { // All indices have been looked through
            break;
        }

        if let Some(SmartValue::Error(err)) = input.get(count){ // if there exists an error, then all other values removed and only error exists.
            let val = (0..input.len()).filter_map(|i| { // filters out all values that do not have an error. Leaving only the error.
                 if i == count{
                    return Some(input[i].clone())
                }
                None
            }).collect::<Vec<SmartValue>>();

            *input = val;
            return;
        }

        if let Some(SmartValue::Comma) = input.get(count){ // if it has a comma, it will compute all the values that were before it
            let range = (last_comma_location..count); // a range of important information
            let range_len = range.len();
            //let mut value = range.map(|x| input[x]).collect::<Vec<SmartValue>>();
            calculate(&mut input[range].to_vec());
            //input.insert(count,value[0].clone());
            count += 1 - range_len;
            last_comma_location = count;
            input.remove(count); // removes the comma.
            // This if let block checks to see if there is a final parameter being passed through, and ensures that it is counted.
            // if let Some(val) = input.get(count){ // if there exists a SmartValue at the count index
            //     if let None = input.get(count+1){ // if there is no Next value
            //
            //     }
            // }
            continue;
        }
        count += 1;
    }
    count = 0;


        // Calculates functions!!
        loop {
            if input.get(count) == None{
                break;
            }

            if let Some(SmartValue::Function(name)) = input.get(count){
                if let Some(val) = input.get(count+1){ //if there is a value in front of a function, it is not a handle to a function!
                    if let SmartValue::Placeholder(parameters) = val{ // A placeholder MUST be in front of a function, otherwise it will not be executed.

                        let function = rcas_functions::Function::get(name.as_str()); // gets the function from its identifier

                        let value:Result<Vec<SmartValue>, Box<dyn std::error::Error>> = match function{
                            rcas_functions::Function::Standard(func) => {
                                func(parameters.clone()) // Executes the function!!
                            },
                            rcas_functions::Function::Nil => { // Doesn't exist in standard functions, therefore it could be a user-defined function.
                                // TODO - IMPLEMENT USER-DEFINED FUNCTIONS HERE
                                Err(Box::new(GenericError {})) // MUST BE REPLACED LATER.
                            }
                        };
                        match value {
                            Ok(val) => {
                                input.remove(count+1);
                                input.remove(count);
                                for i in 0..val.len(){ // INSERTS EVERY VALUE RETURNED INTO INPUT
                                    input.insert(count+i, val[i].clone())
                                }
                            },
                            Err(err) => { // IF THERE IS AN ERROR, EVERY VALUE IS REMOVED.
                                input.clear();
                                input.push(SmartValue::Error(err.to_string()));
                                return;
                            }
                        }
                    }
                } else {

                }
            }
            count += 1;
        }
    count = 0;

    //loop for multiplication and division
    loop{
        if input.get(count) == None{ //All indices have been looked through
            //No need to loop again, therefore it breaks.
            break;
        }

        if let Some(SmartValue::Operator(operator)) = input.get(count){
            if *operator == '*' || *operator == '/'{
                if let Some(SmartValue::Number(left)) = input.get(count - 1){
                    if let Some(SmartValue::Number(right)) = input.get(count + 1){
                        //multiplication and division
                        if *operator == '*'{
                            let replacement = *left * *right;
                            input[count] = SmartValue::Number(replacement);
                            input.remove(count + 1);
                            input.remove(count - 1);
                        } else{
                            if *right == Decimal::from(0){ // check divide by zero
                                input.clear();
                                input.push(SmartValue::Error(DivideByZeroError.to_string()));
                                return;
                            }
                            let replacement = *left / *right;
                            input[count] = SmartValue::Number(replacement);
                            input.remove(count + 1);
                            input.remove(count - 1);
                        }
                        count = 0; //resets the counter
                        //resetting the counter ensures that operations are successfully applied
                    }
                }
            }
        }
        count += 1; //increment so that each index can be calculated
    }
    count = 0;
    //loop for addition and subtraction
    loop{
        if input.get(count) == None{ //all indices have been looked through
            //No need to loop again, therefore it breaks.
            break;
        }

        if let Some(SmartValue::Operator(operator)) = input.get(count){
            if *operator == '+' || *operator == '-'{
                if let Some(SmartValue::Number(left)) = input.get(count - 1){
                    if let Some(SmartValue::Number(right)) = input.get(count + 1){
                        //multiplication and division
                        if *operator == '+'{
                            let replacement = *left + *right;
                            input[count] = SmartValue::Number(replacement);
                            input.remove(count + 1);
                            input.remove(count - 1);
                        } else{
                            let replacement = *left - *right;
                            input[count] = SmartValue::Number(replacement);
                            input.remove(count + 1);
                            input.remove(count - 1);
                        }
                        count = 0; //resets the counter
                        //resetting the counter ensures that operations are successfully applied
                    }
                }
            }
        }
        count += 1; //increment so that each index can be calculated
    }

}

///Checks to see if rules were correctly followed. Returns Result.
pub fn parser(input:&str) -> Result<Vec<SmartValue>, Box<dyn error::Error>>{
    //ONE RULE MUST FOLLOWED, WHICH IS THAT EACH NTH IN THE LOOP CAN ONLY SEE
    //THE NTH IN FRONT OF IT. GOING BACK TO CHECK VALUES IS NOT ALLOWED.

    // GETS THE INPUT
    let input = { // [[4 2 3] [4 3 2]]
        let mut flag = false;
        let mut count = 0;
        let mut dq = false;
        let mut sq = false;

        let tinput = input.chars().filter(|x|{
            // UNICODE: 0022 -> "
            // UNICODE: 0027 -> ' '
            if count < 0{
                return false;
            }

            if *x == '\u{0022}' || *x == '\u{0027}'{
                flag = !flag; // flips flag
            }
            if *x == '\u{0022}'{
                dq = !dq;
            }
            if *x == '\u{0027}'{
                sq = !sq;
            }

            if *x == '[' {
                count += 1;
            } else if *x == ']' {
                count -= 1;
            }
            if flag || count != 0{
                return true;
            }
            if *x != ' '{
                return true;
            }
            false
        }).collect::<String>();

        let mut result = Err(FormattingError{position: tinput.len() as u32 });
        // (A + B + C + (COUNT != 0))' = A'B'C'(COUNT == 0)
        if !flag && !dq && !sq && count == 0{ // if one of these flags are true, there there was either " ', ' ", present, or a missing closing quotation
            result = Ok(tinput);
        }
        result
    };

    let input = match input{
        Ok(val) => val,
        Err(err) => return Err(Box::from(err))
    };

    let mut temp:Vec<SmartValue> = Vec::with_capacity(30); //temp value that will be returned
    let mut buf:Vec<char> = Vec::new(); //buffer for number building
    let mut func_buffer:Vec<Vec<SmartValue>> = Vec::new(); //holds function parameters
    let mut counter = 0; //used to keep track of parentheses
    let mut number = false; //used to keep track of number building
    let mut position = 0; //used to keep track of error position
    let mut dec = false; //used to keep track of decimal usage
    let mut operator = false; //used to keep track of operator usage
    let mut function = false; //used to keep track of functions
    let mut function_name = String::new(); //used to keep track of identifier of last function
    let mut paren_open = false; //used to keep track of a prior open
    let mut is_void = false;
    let mut func_open = false;

    // Converts a vector of characters into a Result<Decimal, Err>
    let to_dec = |x:&Vec<char>| {
        let buffer = (0..x.len()).map(|i| x[i]).collect::<String>(); // turns a vector of characters into a String
        Decimal::from_str(buffer.as_str())
    };

    for i in 0..input.len(){
        let nth:char = input.chars().nth(i).ok_or(GenericError)?;
        let next_nth:Option<char> = input.chars().nth(i + 1);

        // if function{
        //     if let SmartValue::NamedFunction(id, params) = &mut temp[temp.len()-1]{
        //
        //     }
        // }
        // //function parsing is special :)
        // if function{
        //     let check_and_push_to_func_buffer = {
        //         if buf.len() != 0 {
        //             println!("BuffFER: {:?}", &buf);
        //             let buffer = buf.iter().collect::<String>();
        //             println!("BUFFER: {}", &buffer);
        //             //check the parsing of the parameters that are passed in
        //             let result = parser(buffer.as_str());
        //             if let Ok(values) = &result{ //if all went well, then...
        //                 func_buffer.push(values.clone());
        //             }
        //         }
        //     };
        //
        //     if nth == '('{
        //         temp.push(SmartValue::LFuncBound);
        //         continue
        //     }
        //     if nth == ')'{
        //         function = false;
        //         if buf.len() != 0{ //ensures that last parameter gets taken.
        //
        //             check_and_push_to_func_buffer;
        //             buf.clear();
        //         }
        //
        //         let mut params = SmartParameter::new();
        //         for entry in &func_buffer{
        //             println!("EE {:?}", &entry);
        //             print_sv_vec(&entry);
        //             println!("Entry Printed...");
        //             params.push(entry.clone());
        //         }
        //         temp.push(SmartValue::Parameters(params.clone()));
        //         if !params.test_params_conformance(&function_name.as_str()){
        //             //User-input does not conform to function definition
        //             return Err(Box::new(FormattingError{position}))
        //         }
        //
        //         temp.push(SmartValue::RFuncBound);
        //         continue
        //     }
        //
        //     if NUM.contains(nth) || SYM.contains(nth) || OPR.contains(nth){
        //         buf.push(nth);
        //         continue
        //     }
        //     //the all-important separator
        //     if nth == ','{
        //         check_and_push_to_func_buffer;
        //         buf.clear();
        //     }
        //
        //
        // }

        //check parentheses
        if nth == '('{
            if number{ //if a number is being built, then assume that it will multiply
                temp.push(SmartValue::Number(to_dec(&buf)?));
                temp.push(SmartValue::Operator('*'));
            }
            temp.push(SmartValue::LParen);
            buf.clear();
            number = false;
            dec = false;
            counter += 1;
            position += 1;
            operator = false;
            paren_open = true;
            continue
        }

        if nth == ')'{
            if number{
                temp.push(SmartValue::Number(to_dec(&buf)?))
            }
            temp.push(SmartValue::RParen);
            //check if the next character is the start of a number, or parentheses then
            //multiplication is implies
            if let Some(x) = next_nth{
                if NUM.contains(x) || x == '(' || SYM.contains(x){
                    temp.push(SmartValue::Operator('*'))
                }
            }

            buf.clear();
            number = false;
            operator = false;
            dec = false;
            paren_open = false;
            counter -= 1;
            position += 1;
            continue
        }

        //check if it is an operator
        if OPR.contains(nth){

            if paren_open && NUM.contains(next_nth.ok_or(GenericError)?){
                buf.push('-');
                continue;
            }

            // takes care of -(num)
            if nth == '-' && next_nth.ok_or(GenericError)? == '(' && !number{
                buf.push('-');
                buf.push('1');
                number = true;
                operator = false;
                continue;
            }

            //if buf currently isn't building a number and the next char isn't a number,
            //and it is not a - sign, then something is wrong.
            if !number && (!NUM.contains(next_nth.ok_or(GenericError)?) && next_nth.ok_or(GenericError)? != '(') && nth != '-'{
                return Err(Box::new(FormattingError {position}))
            }
            //if there was already an operator, and the the next operator is not negative
            // (for setting negative values) then something is wrong
            if operator && nth != '-'{
                return Err(Box::new(FormattingError {position}))
            }
            if let Some(x) = next_nth{ //can't be +) or *)
                if x == ')'{
                    return Err(Box::new(FormattingError{position}))
                }
            }
            //if nth - and an operator was already written, then number is negative
            if nth == '-' && operator{
                buf.push('-');
                continue
            }
            //if nth is - and is first to appear, then it must be a negative
            if nth == '-' && i == 0{
                buf.push('-');
                continue
            }

            if number{
                temp.push(SmartValue::Number(to_dec(&buf)?))
            }

            operator = true;
            number = false;
            dec = false;
            temp.push(Operator(nth));
            buf.clear();
            position += 1;
            continue
        }
        //check if it is a number
        if NUM.contains(nth){
            if nth == '.'{ //cant be 4.237.
                if dec{
                    return Err(Box::new(FormattingError{position}))
                }
                dec = true; //sets dec to true, as a decimal was inserted
            }

            buf.push(nth);
            number = true;
            operator = false;
            paren_open = false;
            position += 1;
            continue
        }


        //check if it contains symbols
        if SYM.contains(nth){
            if number{
                temp.push(SmartValue::Number(to_dec(&buf)?));
                buf.clear();
            }

            buf.push(nth); //push symbol onto the buffer

            if let Some(x) = next_nth{
                if x == '(' || NUM.contains(x) || OPR.contains(x){ //next nth is (, or a number, or an operator
                    //TODO: Add variable and custom function identification
                    let foo = &buf.iter().collect::<String>();
                    if rcas_functions::STANDARD_FUNCTIONS.contains(&foo.as_str()){
                        temp.push(SmartValue::Function(foo.clone()));
                        //temp.push(SmartValue::DedicatedFunction(foo.clone()));
                       // function_name = foo.clone();
                        //function = true;
                        buf.clear();
                        continue
                    }

                    //temp.push(SmartValue::Operator('*')); //multiplication is inferred if it is a variable
                    //TODO: This if statement should also include variable and function
                    if !rcas_functions::STANDARD_FUNCTIONS.contains(&foo.as_str()){
                        return Err(Box::new(UnknownIdentifierError{position, identifier:foo.clone()}))
                    }
                }
            } else { //symbols are at the end.
                //TODO: Add stuff for variables here :)
                let foo = &buf.iter().collect::<String>();
                return Err(Box::new(UnknownIdentifierError{position, identifier:foo.clone()}))
            }

            number = false;
            operator = false;
            paren_open = false;
            position += 1;
        }

        if nth == ','{
            if counter == 0{ // if a comma is not within parentheses, something is wrong.
                return Err(Box::new(FormattingError {position}));
            }
            if number {
                temp.push(SmartValue::Number(to_dec(&buf)?));
                buf.clear();
                number = false;
                operator = false;
            }
            temp.push(SmartValue::Comma);
            position += 1;
        }

    }
    //now, at the end of the road, do some final checking.
    if operator{ //shouldn't be a lone operator at the end of some input
        return Err(Box::new(FormattingError{position}))
    }
    if number{
        temp.push(SmartValue::Number(to_dec(&buf)?))
    }



    Ok(temp) //sends back the parsed information.
}


