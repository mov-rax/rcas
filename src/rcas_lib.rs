use rust_decimal::*;

use std::fmt;
use crate::rcas_lib::SmartValue::Operator;
use std::str::FromStr;
use std::error;


use std::ops::Deref;
use crate::rcas_functions;
use std::rc::Rc;
use std::cell::RefCell;
use std::time::Instant;
use std::fmt::{Debug, Formatter};
use fxhash::FxHashMap;

use crate::rcas_functions::{FunctionController, Function};
use crate::rcas_constants::ConstantController;
use fltk::table::TableRowSelectMode::SelectMulti;

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

pub struct RCas{
    // Environment that holds user-defined variables and functions. It is an FxHashMap instead of a HashMap for speed purposes.
    // The Environment is encapsulated in an Rc<RefCell<>> In order for it to freely and safely shared to other processes.
    environment: Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>,
    // The Function Controller is used for any pre-defined functions that are given for users. It is able to modify the RCas environment.
    function_controller: FunctionController,
}

impl RCas{

    pub fn new() -> RCas{
        let environment = Rc::from(RefCell::from(FxHashMap::default()));
        let function_controller = FunctionController::new(environment.clone());
        RCas {environment, function_controller}
    }

    pub fn query(&mut self, input:&str) -> QueryResult {
        let time = Instant::now();
        match self.parser(input) {
            Ok(mut parsed) => {
                Wrapper::recurse_print(&parsed, 0);
                let time = time.elapsed().as_micros();
                let mut assignment = None; // used to check if there was an assignment
                if let Some(value) = parsed.get(0){
                    if let SmartValue::Assign(id, value) = value{
                        assignment = Some((true, id.clone()));
                        parsed = value.clone();
                    }
                }
                let mut wrapper = Wrapper::compose(parsed);
                //Wrapper::recurse_print(&wrapper.values, 0);
                wrapper.solve(self);
                //Wrapper::recurse_print(&wrapper.values, 0);
                let mut environment = self.environment.borrow_mut(); // sets ans
                // this looks really ugly, but it works for variable assignment.
                let result = if assignment == None{
                    if let Some(SmartValue::Cmd(_)) = wrapper.values.get(0){
                        // DO NOTHING. YOU DON'T WANT TO ADD COMMAND VALUES TO THE ENVIRONMENT.
                    } else {
                        environment.insert("ans".to_string(), wrapper.values.clone()); // sets ans
                    }

                    wrapper.to_result()
                } else {
                    if let Some(values) = wrapper.values.get(0){
                        // Check to see if the value returned is an error.
                        if let SmartValue::Error(err) = values{
                            return QueryResult::Error(err.clone())
                        }
                        let (_, id) = assignment.unwrap();

                        environment.insert(id.clone(), wrapper.values.clone()); // adds the assignment to the
                        environment.insert("ans".to_string(), wrapper.values.clone()); // adds to ans
                        return match values{
                            SmartValue::Number(number) => QueryResult::Assign(QueryAssign {
                                id,
                                data: DataType::Number(*number),
                            }),
                            SmartValue::Function(identifier) => QueryResult::Assign(QueryAssign {
                                id: identifier.clone(),
                                data: DataType::Function
                            }),
                            SmartValue::Parameters(_) => QueryResult::Assign(QueryAssign {
                                id: "function".to_string(),
                                data: DataType::Function
                            }),
                            SmartValue::Variable(identifier) => QueryResult::Assign(QueryAssign {
                                id: identifier.clone(),
                                data: DataType::Function
                            }),
                            SmartValue::Text(identifier) => QueryResult::Assign(QueryAssign {
                                id: identifier.clone(),
                                data: DataType::Function
                            }),
                            SmartValue::Range(bound1,step,bound2) => QueryResult::Assign(QueryAssign{
                                id,
                                data: DataType::Function }),
                            _ => QueryResult::Error("ASSIGNMENT NOT IMPLEMENTED".to_string()),
                        }
                    }
                    wrapper.to_result() // done in case the result is empty.
                };

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

        if let Some(_) = error.downcast_ref::<GenericError>(){
            info = format!("GENERIC ERROR detected. Please report what was done for this \
                    to appear. Thanks!");
        }

        if let Some(error) = error.downcast_ref::<UnknownIdentifierError>(){
            info = format!("UNKNOWN IDENTIFIER detected at character {}. {} is NOT A VALID \
                    variable or function name.", &error.position, &error.identifier);
        }
        QueryResult::Error(info)
    }

    ///Checks to see if rules were correctly followed. Returns Result.
    fn parser(&mut self, input:&str) -> Result<Vec<SmartValue>, Box<dyn error::Error>>{
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

        let mut environment = self.environment.borrow_mut(); // gets a mutable reference to the environment.
        let mut temp:Vec<SmartValue> = Vec::with_capacity(30); //temp value that will be returned
        let mut buf:Vec<char> = Vec::new(); //buffer for number building
        let mut mat_buf:Vec<SmartValue> = Vec::new(); //buffer for matrix building
        let mut counter = 0; //used to keep track of parentheses
        let mut number = false; //used to keep track of number building
        let mut position = 0; //used to keep track of error position
        let mut dec = false; //used to keep track of decimal usage
        let mut operator = false; //used to keep track of operator usage
        let mut paren_open = false; //used to keep track of a prior open
        let mut comma = false; //used to keep track of commas
        let mut string = false; // used to keep track of string-building
        let mut was_double = false; // used to know if a currently-building string was started with double or single quotes
        let mut beginning_index = 0; // used to keep track of the starting index, in case there is an assignment
        let mut parameters = Vec::new(); // holds identifiers of any parameters when a query is an assignment
        let mut building_matrix = false; //used to keep track of matrix-building
        let mut rows = 0;
        let mut cols = 0;
        let mut matrix_marker = 0;
        let assignment = is_assignment(&*input); // checks to see if this input is an assignment :)

        if let Some((_, index, params)) = &assignment{
            beginning_index = *index;
            if let Some(params) = params{
                parameters = params.clone();
                temp.push(SmartValue::Parameters(parameters.clone())); // pushes a marker
            }
        }

        if parameters.len() != 0 {
            println!("{:?}", &parameters)
        }

        // Converts a vector of characters into a Result<Decimal, Err>
        let to_dec = |x:&Vec<char>| {
            let buffer = (0..x.len()).map(|i| x[i]).collect::<String>(); // turns a vector of characters into a String
            Decimal::from_str(buffer.as_str())
        };

        for i in beginning_index..input.len(){
            let nth:char = input.chars().nth(i).ok_or(GenericError)?;
            let next_nth:Option<char> = input.chars().nth(i + 1);


            // if nth == '['{
            //     building_matrix = true;
            //     counter += 1;
            // }
            //
            // if nth == ' ' && building_matrix{
            //     let info = (&temp[matrix_marker..]).to_vec().clone();
            //     for _ in 0..info.len(){
            //         temp.pop();
            //     }
            //     let composed = Self::composer(info);
            //     let solved = self.recurse_solve(composed);
            //     let solved = solved[0].clone();
            //     mat_buf.push(solved);
            //     matrix_marker = temp.len();
            //     buf.clear();
            //     number = false;
            //     dec = false;
            //     operator = false;
            //     continue;
            // }
            //
            // if nth == ';' && building_matrix{
            // if cols == 0{
            //     cols = mat_buf.len();
            // }
            //     rows += 1;
            //
            // }
            //
            // if nth == ']'{
            //     counter -= 1;
            //     if counter == 0{
            //         building_matrix = false;
            //     }
            // }

            if nth == r#"""#.chars().nth(0).unwrap(){ // double quotes
                if string{
                    string = false;
                    let text = buf.iter().collect::<String>();
                    temp.push(SmartValue::Text(text));
                    buf.clear();
                    continue;
                }
                string = true;
                was_double = true;
                continue;
            }

            if nth == r#"'"#.chars().nth(0).unwrap(){ // single quotes
                if string{
                    if was_double{
                        buf.push(r#"'"#.chars().nth(0).unwrap());
                        continue;
                    }
                    string = false;
                    let text = buf.iter().collect::<String>();
                    temp.push(SmartValue::Text(text));
                    buf.clear();
                    continue;
                }
                string = true;
                continue;
            }

            if string{ // if we are building a string, then push it to the buffer.
                buf.push(nth);
                continue;
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
                // takes care of negative values in parameters
                if (paren_open | comma) && NUM.contains(next_nth.ok_or(GenericError)?){
                    buf.push('-');
                    comma = false;
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
                if !number && (!NUM.contains(next_nth.ok_or(GenericError)?) && next_nth.ok_or(GenericError)? != '(' && !SYM.contains(next_nth.ok_or(GenericError)?)) && nth != '-'{
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
                if nth == '-' && (i == 0 || i == beginning_index+1){
                    buf.push('-');
                    continue
                }

                if number{
                    temp.push(SmartValue::Number(to_dec(&buf)?))
                }

                operator = true;
                number = false;
                dec = false;
                comma = false;
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
                comma = false;
                position += 1;
                continue
            }


            //check if it contains symbols
            if SYM.contains(nth){
                if number{
                    temp.push(SmartValue::Number(to_dec(&buf)?));
                    temp.push(SmartValue::Operator('*')); // multiplies
                    buf.clear();
                }

                buf.push(nth); //push symbol onto the buffer
                let foo = &buf.iter().collect::<String>();
                if let Some(next) = next_nth {
                    let mut found = false;


                    if let Some(value) = environment.get(foo){ // custom function finding :)
                        let eee = value.iter().take_while(|s| {
                            if let SmartValue::Variable(_) = s{
                                return false;
                            }
                            if let SmartValue::Placeholder(_) = s{ // the only reason why there would be a placeholder is if there is a variable
                                return false;
                            }
                            true
                        }).count(); // this gets the length of the environment variable, not including any Variables being in it.
                        if eee != value.len(){ // if it found a Variable, then it means that this is a function.
                            temp.push(SmartValue::Function(foo.clone()));
                            paren_open = false;
                            found = true;
                            operator = false;
                            position += 1;
                            buf.clear();
                            continue
                        }
                    }


                    if next == '(' || next == ')' || next == ',' || NUM.contains(next) || OPR.contains(next) {
                        if self.function_controller.get(foo) != Function::Nil{
                            temp.push(SmartValue::Function(foo.clone()));
                            //number = true;
                            paren_open = false;
                            found = true;
                            operator = false;
                            position += 1;
                            buf.clear();
                            continue
                        } else if parameters.contains(foo){
                            temp.push(SmartValue::Variable(foo.clone())); // pushes the variable onto the temporary array
                            paren_open = false;
                            found = true;
                            operator = false;
                            position += 1;
                            buf.clear();
                            continue
                        } else if environment.contains_key(foo){
                            for value in environment.get(foo).unwrap() {
                                temp.push(value.clone());
                            }
                            //number = true;
                            paren_open = false;
                            found = true;
                            operator = false;
                            position += 1;
                            buf.clear();
                            continue
                        } else if let Some(constant) = ConstantController::get(foo){
                            temp.push(constant);
                            paren_open = false;
                            found = true;
                            operator = false;
                            position += 1;
                            buf.clear();
                        } else {
                            return Err(Box::new(UnknownIdentifierError{position, identifier:foo.clone()}))
                        }
                    }
                    if (next == '(' || NUM.contains(next)) && found{
                        temp.push(SmartValue::Operator('*'));
                        continue
                    }
                } else {
                    if self.function_controller.get(foo) != Function::Nil{
                        temp.push(SmartValue::Function(foo.clone()));
                        buf.clear();
                    } else if parameters.contains(foo){
                        temp.push(SmartValue::Variable(foo.clone()));
                        buf.clear();
                    } else if environment.contains_key(foo){
                        for value in environment.get(foo).unwrap() {
                            temp.push(value.clone());
                        }
                        buf.clear();
                    } else if let Some(constant) = ConstantController::get(foo){
                        temp.push(constant);
                        buf.clear();
                    } else {
                        return Err(Box::new(UnknownIdentifierError{position, identifier:foo.clone()}))
                    }
                }

                number = false;
                operator = false;
                paren_open = false;
                position += 1;
            }

            if nth == ',' || nth == ':'{ // Comma & RangeMarker
                if counter == 0 && nth == ','{ // if a Comma is not within parentheses (or is alone), something is wrong.
                    return Err(Box::new(FormattingError {position}));
                }
                if number {
                    temp.push(SmartValue::Number(to_dec(&buf)?));
                    buf.clear();
                    number = false;
                    operator = false;
                }
                if nth == ','{
                    comma = true;
                    temp.push(SmartValue::Comma);
                } else {
                    temp.push(SmartValue::RangeMarker);
                }
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

        if let Some((id, _, _)) = assignment{ // if there was an assignment, this is a special type of parsed information :)
            return Ok(vec![SmartValue::Assign(id.clone(), temp)]) // returns a singular assign SmartValue
        }

        Ok(temp) //sends back the parsed information.
    }

    ///Only takes slices with NO PARENTHESES.
    pub fn calculate(&mut self, input: &mut Vec<SmartValue>){
        let mut count:usize = 0;
        let mut last_comma_location:usize = 0;
        //print_sv_vec(&input);
        //Wrapper::recurse_print(&input, 0);
        //println!("---");
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
                let comma_remove = |inny:&mut Vec<SmartValue>| {
                    for i in 0..inny.len(){
                        if let Some(value) = inny.get(i){
                            if *value == SmartValue::Comma{
                                inny.remove(i);
                            }
                        }
                    }
                };
                let range = last_comma_location..count; // a range of important information
                self.calculate(&mut input[range.clone()].to_vec());
                comma_remove(input); // I tried removing it using math but I guess I couldn't figure out how to make it work consistently
                continue;
            }

            if let Some(SmartValue::RangeMarker) = input.get(count){
                let step_upper_bound = if let Some(SmartValue::RangeMarker) = input.get(count+2){ // there is another RangeMarker here, meaning that it is a range with a step given
                    Some((input[count+1].clone(), input[count+3].clone()))
                } else {
                    None
                }; // if a second RangeMarker exists, step_upper_bound will contain the step and the upper bound
                let lower_bound = input[count-1].clone();
                let range = if let Some(step_upper_bound) = &step_upper_bound{
                    let mut result = None;
                    if let SmartValue::Number(lower) = lower_bound{
                        if let SmartValue::Number(step) = step_upper_bound.0{
                            if let SmartValue::Number(upper) = step_upper_bound.1{
                                result = Some(SmartValue::Range(lower,step,upper));
                            }
                        }
                    }
                    result
                } else { // A range with a default step of 1
                    let mut result = None;
                    if let SmartValue::Number(lower) = lower_bound{
                        if let SmartValue::Number(upper) = input[count+1]{
                            result = Some(SmartValue::Range(lower,Decimal::from(1),upper));
                        }
                    }
                    result
                };

                if let Some(range) = range{ // check to see if the range was obtained
                    if let Some(_) = step_upper_bound{ // BOUND:STEP:BOUND
                        for _ in 0..5{
                            input.remove(count-1); // remove what was once there
                        }
                        input.insert(count-1,range.clone());
                    } else { // BOUND:BOUND
                        for _ in 0..3{
                            input.remove(count-1); // remove what was once there
                        }
                        input.insert(count-1, range.clone());
                    }
                } else { // There was an error :(
                    input.clear();
                    input.push(SmartValue::Error(String::from("Range Syntax Error")));
                }
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

                        fn take_while_loop(input:&Vec<SmartValue>) -> bool{
                            input.iter().take_while(|s| {
                                if let SmartValue::Variable(_) = s{
                                    return false;
                                }
                                if let SmartValue::Placeholder(holder) = s{
                                    return take_while_loop(holder)
                                }
                                true
                            }).count() == input.len()
                    }
                        let len = parameters.iter().take_while(|s| {
                            if let SmartValue::Variable(_) = s{
                                return false;
                            }
                            if let SmartValue::Placeholder(holder) = s{
                                return take_while_loop(holder)
                            }
                            true
                        }).count();

                        if len == parameters.len(){
                            let function = self.function_controller.get(name.as_str()); // gets the function from its identifier
                            let value:Result<Vec<SmartValue>, Box<dyn std::error::Error>> = match function {
                                rcas_functions::Function::Standard(func) => {
                                    func(&mut self.function_controller, parameters.clone()) // Executes the function!!
                                },
                                rcas_functions::Function::Nil => { // Function identifier does NOT exist.
                                    Err(Box::new(GenericError {}))
                                }
                            };
                            match value {
                                Ok(val) => {
                                    input.remove(count+1);
                                    input.remove(count);
                                    for i in 0..val.len(){ // INSERTS EVERY VALUE RETURNED INTO INPUT
                                        input.insert(count+i, val[i].clone());
                                    }
                                },
                                Err(err) => { // IF THERE IS AN ERROR, EVERY VALUE IS REMOVED.
                                    input.clear();
                                    input.push(SmartValue::Error(err.to_string()));
                                    return;
                                }
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

                let placeholder = SmartValue::Placeholder(Self::composer(subsection));
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
                    subsection = Self::composer(subsection); //does it all over again :)
                }
            }

            input //the magic has been done.
        }
    }

    /// Returns a safe reference to the rcas environment.
    pub fn get_environment(&mut self) -> Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>{
        self.environment.clone()
    }

    pub fn recurse_solve(&mut self, mut input:Vec<SmartValue>) -> Vec<SmartValue>{
        //print_sv_vec(&input);
        return if has_placeholder(&input) {
            for x in 0..input.len() {
                if let Some(SmartValue::Placeholder(holder)) = input.get(x) {
                    //print_sv_vec(&holder);
                    let solved = self.recurse_solve(holder.clone());
                    // Doesn't remove the placeholder if there is a function before it.

                    if let Some(SmartValue::Function(_)) = input.get(safe_sub(x)){
                        //NOTHING HERE.
                    } else if solved.len() == 0 { // if the solve returned NOTHING, and there is no function before this placeholder, then the entire answer must be nothing.
                        return Vec::new()
                    }


                    // a nice way to get the resolved value. If the previous value is a not function, then it is just a Number.
                    // Otherwise, it is probably parameters to a function, and as such should be in
                    // A placeholder.
                    let value = if let Some(SmartValue::Function(_)) = input.get(safe_sub(x)){ // if previous was a function, then put the solution in a placeholder.
                        SmartValue::Placeholder(solved)
                    } else {
                        solved[0].clone()
                    };
                    input[x] = value;
                }
                //println!("X: {}", &input[x].get_value());
            }
            let mut input = input;

            self.calculate(&mut input);
            input
        } else {
            let mut input = input;
            // Wrapper::recurse_print(&input,0);
            // println!("----");
            self.calculate(&mut input);
            input
        }
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
    RefreshEnvironment,
    SetMode(CalculationMode),
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
    Image(QueryImage), // Assigning to a query
    Function // Assigned to a function
}

pub struct SmartMatrix{
    data: Vec<Decimal>,
    rows: u64,
    columns: u64,
}

#[derive(PartialEq, Clone)]
pub struct Wrapper{
    pub values: Vec<SmartValue>,
}

impl Wrapper{
    pub fn new() -> Self{
        Wrapper {values:Vec::new()}
    }
    ///Composes a Wrapper.
    pub fn compose(input: Vec<SmartValue>) -> Self{
        //Self::recurse_print(&input, 0);
        let values = RCas::composer(input);
        //Self::recurse_print(&values, 0);
        Wrapper {values}
    }

    pub fn recurse_print(input:&Vec<SmartValue>, level:usize){
        for value in input{
            match value{
                SmartValue::Number(num) => {
                    println!("{}:{}", level, num);
                },
                SmartValue::Function(name) => {
                    println!("{}:{}", level, &name);
                },
                SmartValue::Placeholder(holder) => {
                    println!("{}:PLACEHOLDER", level);
                    Self::recurse_print(&holder, level+1);
                },
                SmartValue::Operator(opr) => {
                    println!("{}:{}", level, *opr);
                },
                SmartValue::LParen => {
                    println!("{}:(", level);
                },
                SmartValue::RParen => {
                    println!("{}:)", level);
                },
                SmartValue::Comma => {
                    println!("{}:,", level);
                },
                SmartValue::Variable(id) => println!("{}:{}", level, id),
                idk => println!("{}:{}", level, idk.get_value())
            }
        }
    }

    ///Solves a Wrapper.
    pub fn solve(&mut self, rcas:&mut RCas){
        self.values = rcas.recurse_solve(self.values.clone())
    }

    pub fn print_raw(&self){
        print_sv_vec(&self.values);
    }

    pub fn to_string(&self) -> String{ sv_vec_string(&self.values) }

    pub fn to_result(&self) -> QueryResult{
        if self.values.len() == 1 {

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
    Text(String),
    LParen,
    RParen,
    Variable(String), // Utilized in user-defined functions. Each variable as an identifier attributed to it.
    Parameters(Vec<String>), // A marker that is utilized during parsing that identifies the order in which the name of parameters are placed in a function declaration
    Placeholder(Vec<SmartValue>),
    Range(Decimal,Decimal,Decimal), // Bound1, Step, Bound2
    RangeMarker, // A colon :
    Label(String,Vec<SmartValue>), // A label can contains an identifier and a possible expression.
    Comma, // A special character used to separate parameters
    Assign(String, Vec<SmartValue>), // A special equals = operator. It is used to show that a value is being assigned to an identifier.
    Error(String), // An error returned from a function call. The Stringified version of the error is returned.
    Cmd(Command), // A command that is returned from a function call. May exist when solving
}

impl SmartValue{
    pub fn get_value(&self) -> String{
        let mut buf = String::new();
        match self{
            SmartValue::Operator(x) => buf.push(*x),
            SmartValue::Function(id) => buf.push_str(id),
            SmartValue::Number(x) => {
                let num = format!("{}", x);
                buf.push_str(&num);
            },
            SmartValue::LParen => buf.push('('),
            SmartValue::RParen => buf.push(')'),
            SmartValue::Placeholder(holder) => {
                buf.push('(');
                for x in holder{
                    buf.push_str(&*x.get_value());
                }
                buf.push(')');
            },
            SmartValue::Text(string) => buf.push_str(&**string),
            SmartValue::Variable(id) => buf.push_str(id),
            SmartValue::Parameters(_) => {},
            SmartValue::Range(bound1,step,bound2) => {
                if *step == Decimal::from(1){
                    buf.push_str(format!("{}:{}", bound1, bound2).as_str());
                } else {
                    buf.push_str(format!("{}:{}:{}", bound1, step, bound2).as_str());
                }
            }
            _ => buf.push('?')
        }
        buf
    }
}


#[derive(Debug,Clone,PartialEq)]
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

#[inline(always)]
fn safe_sub(input:usize) -> usize{
    return if input > 0{
        input -1
    } else{
        0
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
/// Checks to see of an input contains an assignment. If it does, it returns the index after the assignment operator was found.
fn is_assignment(input:&str) -> Option<(String, usize, Option<Vec<String>>)>{

    let mut identifier = input.chars().take_while(|x| *x != '=' ).collect::<String>();
    let identifier_len = identifier.len();
    let mut has_parameters = false;

    for x in identifier.chars(){
        if !SYM.contains(x  ) && x != '(' && x != ')' && x != ','{
            return None;
        }
        if x == '('{
            has_parameters = true;
        }
    }
    if identifier.chars().filter_map(|x| match x {
        '(' => Some(1),
        ')' => Some(-1),
        _ => None
    }).sum::<i32>() > 0 { // if it is within open parentheses, then this is no assignment.
        return None;
    }
    // There is an identifier at this point.

    //This will get the parameters if the assignment was that of a function.
    let parameters = if identifier.chars().last().unwrap() == ')'{ // if there is a closing word there
        let count = identifier.chars().take_while(|x| *x != '(').count();
        let parameters = identifier.chars().skip(count+1).take_while(|x| *x != ')').collect::<String>();
        let parameter_identifiers = parameters.split(",").map(|x| String::from(x)).collect::<Vec<String>>();
        identifier = (&identifier[0..count]).to_string();
        Some(parameter_identifiers)
    } else {
        None
    };


    if identifier_len == input.len(){
        return None;
    }
    Some((identifier, identifier_len, parameters))
}









