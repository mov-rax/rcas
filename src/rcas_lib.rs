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
use crate::rcas_functions::SmartFunction;

//constants
const ADD:char = '+'; //addition
const SUB:char = '-'; //subtraction
const MUL:char = '*'; //multiplication
const DIV:char = '/'; //division
const MOD:char = '%'; //modulo
const PHD:char = '█'; //placeholder
const FNC:char = 'ƒ'; //pre-defined (constant) function
const FNV:char = '⭒'; //variable function
const VAR:char = '⭑'; //variable
const PAR:char = '⮂'; //parameters
//static STDFUNC:[&str; 4] = ["sin", "cos", "tan", "sec"]; //standard functions
static SYM:&str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"; //allowed symbols
static NUM:&str = "1234567890."; //allowed numbers
static OPR:&str = "+-*/"; //allowed operators

//Errors

#[derive(Debug, Clone)]
struct ParenthesesError{
    positive:bool, //too much or too little parentheses. true == too much, false == too little
    position:u32,
}
#[derive(Debug, Clone)]
struct FormattingError{
    position:u32
}
#[derive(Debug, Clone)]
struct GenericError;
#[derive(Debug, Clone)]
struct UnknownIdentifierError{
    position:u32,
    identifier:String,
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

impl error::Error for ParenthesesError{}
impl error::Error for FormattingError{}
impl error::Error for GenericError{}
impl error::Error for UnknownIdentifierError{}

pub struct RCas;

impl RCas{
    pub fn query(input:&str) -> String{
        match parser(input){
            Ok(parsed) => {
                let mut wrapper = Wrapper::compose(parsed);
                //print_sv_vec(&wrapper.values);
                wrapper.solve();
                wrapper.to_string()
            },
            Err(error) => {
                println!("Parsing Error :(");
                let error = error.deref();
                let mut info = String::new();
                //time to try all types of errors :)
                if let Some(error) = error.downcast_ref::<ParenthesesError>(){
                    let add_or_remove = {
                        let mut option = "REMOVING";
                        if error.positive {
                            option = "ADDING";
                        } option};
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
                info
            }
        }
    }

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

}

#[derive(Debug, PartialEq, Clone)]
pub enum SmartValue{
    Operator(char),
    Function(String), //holds the user-defined function identifier
    DedicatedFunction(String), //holds the function identifier
    TestDedicatedFunction(String, Vec<Vec<SmartValue>>), // Has identifier of function as well as parameters in a 2D Vector. Each parameter in a function can hold an expression to calculate.
    Parameters(SmartParameter), //holds parameters of functions
    Number(Decimal),
    LParen,
    RParen,
    LFuncBound, //separate types of parentheses are used to identify parentheses that contain expressions and those that do not.
    RFuncBound,
    Variable(String), //variable does not require ( ), therefore there exists Function and Variable
    Placeholder(Vec<SmartValue>),
    Range(Decimal,Decimal,Decimal), // Lower, Step, Upper
    Label(String,Vec<SmartValue>), // A label can contains an identifier and a possible expression.
    Comma
}

impl SmartValue{
    pub fn get_value(&self) -> String{
        let mut buf = String::new();
        match self{
            SmartValue::Operator(x) => buf.push(*x),
            SmartValue::Function(_) => buf.push(FNC),
            SmartValue::DedicatedFunction(x) => buf.push(FNC),
            SmartValue::Number(x) => {
                let num = format!("{}", x);
                for sus in num.chars(){
                    buf.push(sus)
                }

            },
            SmartValue::LParen => buf.push('('),
            SmartValue::RParen => buf.push(')'),
            SmartValue::Placeholder(_) => buf.push(PHD),
            SmartValue::Variable(_) => buf.push(VAR),
            SmartValue::Parameters(_) => buf.push(PAR),
            SmartValue::LFuncBound => buf.push('|'),
            SmartValue::RFuncBound => buf.push('|'),
            _ => buf.push('?')
        }
        buf
    }
}

/// Contains data of all passed-through parameters.
/// Each parameter is stored as a Wrapper.
#[derive(Debug, PartialEq, Clone)]
pub struct SmartParameter{
    params:Vec<Decimal>
}

impl SmartParameter{
    /// Takes in param1, param2, param2, ... , and converts it into
    /// a specialized datatype that can easily transverse parameters
    pub fn from_str(input:&str) -> Self{
        let magic:Vec<&str> = input.split(',').collect();
        let mut values:Vec<Decimal> = Vec::new();
        for x in magic{
            // this does a LOT of stuff. It parses and solves each entry.
            if let Ok(value) = parser(x){
                if let SmartValue::Number(num) = recurse_solve(value.clone())[0].clone(){
                    values.push(num);
                }
            }
        }
        Self {params: values}
    }

    pub fn new() -> Self{
        Self{params:Vec::new()}
    }
    ///Will try to solve and insert the result as a parameter
    pub fn push(&mut self, input:Vec<SmartValue>){
        if let SmartValue::Number(number) = recurse_solve(input)[0].clone(){
            self.params.push(number);
        }
    }
    /// Returns true if the number of parameters given conforms with the type of function
    /// associated with the given identifier.
    pub fn test_params_conformance(&self, identifier:&str) -> bool{
        let func = rcas_functions::SmartFunction::get(identifier);
        println!("{}", self.params.len());
        match func{
            SmartFunction::Mono(_) => { self.params.len() == 1},
            SmartFunction::Binary(_) => {self.params.len() == 2},
            SmartFunction::Poly(_) => {self.params.len() >= 1},
            SmartFunction::PolyPoly(_) => {self.params.len() >= 1},
            SmartFunction::MonoOpt(_) => {self.params.len() == 1},
            SmartFunction::BinaryOpt(_) => {self.params.len() == 2},
            SmartFunction::PolyOpt(_) => {self.params.len() >= 1},
            SmartFunction::PolyPolyOpt(_) => {self.params.len() >= 1},
            _ => {false}
        }
    }
}

pub struct Params{
    params:Vec<Vec<SmartValue>>
}

#[derive(Debug,Clone)]
pub enum CalculationMode{
    Radian,
    Degree
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
                let value = recurse_solve(holder.clone())[0].clone();
                input[x] = value;
            }
            //println!("X: {}", &input[x].get_value());
        }
        get_calculated(input)
    } else {
        get_calculated(input)
    }
}

fn recurse_function_solve(mut input:Vec<SmartValue>){
    if has_function(&input){
        for x in 0..input.len(){
            if let Some(SmartValue::Function(identifier)) = input.get(x){

            }
        }
    }
    if has_placeholder(&input){

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
    let mut flag = false;
    for x in input{
        if let SmartValue::Placeholder(_) = x{
            flag = true;
        }
    }
    flag
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
            right= x;
            found_left = true;
        }
        if input[x] == SmartValue::LParen{
            counter += 1;
        }

        if input[x] == SmartValue::RParen{
            counter -= 1;
        }
        if input[x] == SmartValue::RParen && counter == 0{
            right = x;
            break;
        }


    }
    (left, right)
}

fn number_of_parentheses_sections(input: &Vec<SmartValue>) -> usize{
    let mut count:usize = 0;
    for x in input{
        if *x == SmartValue::LParen{
            count += 1;
        }
    }
    count
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
    if number_of_parentheses_sections(&input) == 0{ //no parentheses, therefore, no need to compose.
        return input;
    } else {
        //print_sv_vec(&input);
        //This will replace all parentheses within the same depth.
        while number_of_parentheses_sections(&input) != 0{
            let parentheses_locations = get_outermost_parentheses(&input);
            //println!("{}, {}", parentheses_locations.0, parentheses_locations.1);
            //gets value inside first parentheses and puts it into a Placeholder
            let subsection = input[parentheses_locations.0+1 .. parentheses_locations.1]
                .to_vec();
            let placeholder = SmartValue::Placeholder(composer(subsection));
            placeholder_locations.push(parentheses_locations.0);
            for x in parentheses_locations.0 ..parentheses_locations.1{
                input.remove(parentheses_locations.0);
            }
            if parentheses_locations.0 == input.len(){
                input.push(placeholder)
            } else{
                input[parentheses_locations.0] = placeholder;
            }

            //print_sv_vec(&input)

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
    let input = input.chars().filter(|x| *x != ' ').collect::<String>();
    let mut temp:Vec<SmartValue> = Vec::new(); //temp value that will be returned
    let mut buf:Vec<char> = Vec::new(); //buffer for number building
    let mut func_buffer:Vec<Vec<SmartValue>> = Vec::new(); //holds function parameters
    let mut counter = 0; //used to keep track of parentheses
    let mut number = false; //used to keep track of number building
    let mut position = 0; //used to keep track of error position
    let mut dec = false; //used to keep track of decimal usage
    let mut operator = false; //used to keep track of operator usage
    let mut function = false; //used to keep track of functions
    let mut function_name = String::new(); //used to keep track of identifier of last function
    let to_dec = |x:&Vec<char>| {
        let mut buffer = String::new();
        for i in x{
            buffer.push(*i);
        }
        Decimal::from_str(buffer.as_str())
    };

    for i in 0..input.len(){
        let nth:char = input.chars().nth(i).ok_or(GenericError)?;
        let next_nth:Option<char> = input.chars().nth(i + 1);


        //function parsing is special :)
        if function{

            let check_and_push_to_func_buffer = {
                if buf.len() != 0 {
                    println!("BuffFER: {:?}", &buf);
                    let buffer = buf.iter().collect::<String>();
                    println!("BUFFER: {}", &buffer);
                    //check the parsing of the parameters that are passed in
                    let result = parser(buffer.as_str());
                    if let Ok(values) = &result{ //if all went well, then...
                        func_buffer.push(values.clone());
                    }
                }
            };

            if nth == '('{
                temp.push(SmartValue::LFuncBound);
                continue
            }
            if nth == ')'{
                function = false;
                if buf.len() != 0{ //ensures that last parameter gets taken.

                    check_and_push_to_func_buffer;
                    buf.clear();
                }

                let mut params = SmartParameter::new();
                for entry in &func_buffer{
                    println!("EE {:?}", &entry);
                    print_sv_vec(&entry);
                    println!("Entry Printed...");
                    params.push(entry.clone());
                }
                temp.push(SmartValue::Parameters(params.clone()));
                if !params.test_params_conformance(&function_name.as_str()){
                    //User-input does not conform to function definition
                    return Err(Box::new(FormattingError{position}))
                }

                temp.push(SmartValue::RFuncBound);
                continue
            }

            if NUM.contains(nth) || SYM.contains(nth) || OPR.contains(nth){
                buf.push(nth);
                continue
            }
            //the all-important separator
            if nth == ','{
                check_and_push_to_func_buffer;
                buf.clear();
            }


        }

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
            counter -= 1;
            position += 1;
            continue
        }
        //check if it is an operator
        if OPR.contains(nth){
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

//            if let Some(x) = next_nth{
//                if x == '(' || SYM.contains(x){
//
//                }
//            }

            buf.push(nth);
            number = true;
            operator = false;
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
                        temp.push(SmartValue::DedicatedFunction(foo.clone()));
                        function_name = foo.clone();
                        function = true;
                        buf.clear();
                        continue
                    }
                    temp.push(SmartValue::Operator('*')); //multiplication is inferred if it is a variable
                    //TODO: This if statement should also include variable and function
                    if !rcas_functions::STANDARD_FUNCTIONS.contains(&foo.as_str()){
                        return Err(Box::new(UnknownIdentifierError{position, identifier:foo.clone()}))
                    }
                }
            } else { //symbols are at the end. (can't be a function, has to be a variable)
                //TODO: Add stuff for variables here :)
                let foo = &buf.iter().collect::<String>();
                return Err(Box::new(UnknownIdentifierError{position, identifier:foo.clone()}))
            }

            number = false;
            operator = false;
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
