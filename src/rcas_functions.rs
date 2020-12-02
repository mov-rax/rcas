// This implements functions, which are used in rcas_lib and are known as
// SmartValue::DedicatedFunction.

use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::any::Any;
use std::fmt::Display;
use std::error::Error;
use std::fmt;
use statrs;
use crate::rcas_lib::{SmartValue, FormattingError, TypeMismatchError, IncorrectNumberOfArgumentsError, Command, NegativeNumberError, OverflowError, RCas, Wrapper};
use std::ops::Div;
use crate::rcas_lib::DataType::Number;
use fxhash::FxHashMap;
use std::rc::Rc;
use std::cell::RefCell;

///Shows to the world all of the standard functions given by default.
pub static STANDARD_FUNCTIONS:[&str;30] = ["cos", "sin", "tan", "sec", "csc", "cot", "mod", "plot", "sum", "exp", "factorial", "sqrt", "clear", "^", "!", "cosh",
                                            "sinh", "tanh", "acos", "asin", "atan", "log", "ln", "mul", "max", "min", "avg", "stdev", "mag", "angle"];


pub enum Function{
    Standard(Box<dyn Fn(&mut FunctionController, Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>>),
    Nil
}

pub struct FunctionController{
    environment: Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>,
    custom_function_id: String, // used just for a current custom function
    custom_function: Vec<SmartValue>, //used for a current custom function
}

impl FunctionController {

    /// Creates a new FunctionController that requires an environment to be given.
    pub fn new(environment:Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>) -> Self{
        FunctionController { environment , custom_function_id: String::new(), custom_function: Vec::new()}
    }

    pub fn get(&mut self, identifier: &str) -> Function {
        match identifier {
            "cos" => Function::Standard(Box::new(Self::cos_f)),
            "sin" => Function::Standard(Box::new(Self::sin_f)),
            "tan" => Function::Standard(Box::new(Self::tan_f)),
            "sec" => Function::Standard(Box::new(Self::sec_f)),
            "csc" => Function::Standard(Box::new(Self::csc_f)),
            "cot" => Function::Standard(Box::new(Self::cot_f)),
            "mod" => Function::Standard(Box::new(Self::mod_f)),
            "sum" => Function::Standard(Box::new(Self::sum_f)),
            "mul" => Function::Standard(Box::new(Self::mul_f)),
            "exp" => Function::Standard(Box::new(Self::exp_f)),
            "^" => Function::Standard(Box::new(Self::exp_f)),
            "factorial" => Function::Standard(Box::new(Self::factorial_f)),
            "!" => Function::Standard(Box::new(Self::factorial_f)),
            "sqrt" => Function::Standard(Box::new(Self::sqrt_f)),
            "cosh" => Function::Standard(Box::new(Self::cosh_f)),
            "sinh" => Function::Standard(Box::new(Self::sinh_f)),
            "tanh" => Function::Standard(Box::new(Self::tanh_f)),
            "acos" => Function::Standard(Box::new(Self::acos_f)),
            "asin" => Function::Standard(Box::new(Self::asin_f)),
            "atan" => Function::Standard(Box::new(Self::atan_f)),
            "log" => Function::Standard(Box::new(Self::log_f)),
            "ln" => Function::Standard(Box::new(Self::ln_f)),
            "max" => Function::Standard(Box::new(Self::max_f)),
            "min" => Function::Standard(Box::new(Self::min_f)),
            "avg" => Function::Standard(Box::new(Self::avg_f)),
            "stdev" => Function::Standard(Box::new(Self::stdev_f)),
            "mag" => Function::Standard(Box::new(Self::mag_f)),
            "angle" => Function::Standard(Box::new(Self::angle_f)),
            "clear" => Function::Standard(Box::new(Self::clear_v)),
            //"drop" => Function::Standard(Box::new(Self::drop_v)),
            func => {
                let environment = self.environment.borrow();
                if let Some(value) = environment.get(func){
                    self.custom_function_id = func.to_string();
                    self.custom_function = value.clone();
                    return Function::Standard(Box::new(Self::custom_function_f))
                }
                Function::Nil
            } // Returned if function identifier does not exist.
        }
    }

    pub fn custom_function_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>> {

        let mut answer:Vec<SmartValue> = (&self.custom_function[1..]).to_vec();
        if let SmartValue::Parameters(params) = &self.custom_function[0]{
            if params.len() != input.len(){
                return Err(Box::new(IncorrectNumberOfArgumentsError { name: "custom_function", found: input.len(), requires: params.len() }))
            }

            for (index, value) in params.iter().enumerate(){
                answer = answer.iter().map(|s| {
                    if let SmartValue::Variable(id) = s{
                        if id == value{
                            return input[index].clone();
                        }
                    }
                    s.clone()
                }).collect();
            }
            let answer = Wrapper::compose(answer);
            return Ok(answer.values.clone())
        }
        Err(Box::new(TypeMismatchError{}))
    }
    // pub fn drop_v(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    //     let mut environment = self.environment.borrow_mut();
    //     for value in &input{
    //         if let SmartValue::Function(id) = value{
    //             if environment.
    //         }
    //     }
    // }

    pub fn clear_v(&mut self,input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>> {
        if input.is_empty(){
            return Ok(vec![SmartValue::Cmd(Command::ClearScreen)])
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name:"clear", found:input.len(), requires:0}))
    }

    pub fn sqrt_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0]{
                let number = number.to_f64().unwrap();
                let number = number.sqrt();
                let number = Decimal::from_f64(number).unwrap();
                return Ok(vec![SmartValue::Number(number)])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sqrt", found: input.len(), requires: 1}))
    }

    pub fn factorial_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0]{

                if number.is_sign_negative(){ // ensures that the number cannot be negative
                    return Err(Box::new(NegativeNumberError{}))
                }

                if number.round() == number{ // if it is an integer, then a more exact version of factorial will be used.
                    let mut temp = number;
                    let mut value = Decimal::from(1);
                    let subtractor = Decimal::from(1);
                    for _ in 0..temp.to_u128().unwrap(){
                        let checked = value.checked_mul(temp);
                        if let Some(checked) = checked{
                            value = checked;
                        } else {
                            return Err(Box::new(OverflowError {}))
                        }
                        temp -= subtractor;
                    }
                    return Ok(vec![SmartValue::Number(value)])
                }
                let result = statrs::function::gamma::gamma(number.to_f64().unwrap()+1.0); // magical math stuff happens that is equivalent to any n!
                let result = Decimal::from_f64(result).unwrap();
                let result = SmartValue::Number(result);
                return Ok(vec![result])
            } else{
                return Err(Box::new(TypeMismatchError {}))
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError {name: "factorial", found: input.len(), requires: 1}))
    }

    pub fn exp_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 2{
            if let SmartValue::Number(base) = input[0]{
                if let SmartValue::Number(exponent) = input[1]{
                    let base = base.to_f64().unwrap();
                    let exponent = exponent.to_f64().unwrap();
                    let number = base.powf(exponent);
                    let number = Decimal::from_f64(number);
                    if let Some(number) = number{
                        return Ok(vec![SmartValue::Number(number)]);
                    }
                    return Err(Box::new(OverflowError{}))
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError {name: "exp", found:input.len(), requires:2}))
    }

    pub fn sum_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() != 0{
            let mut sum = Decimal::from(0);
            for i in input{
                if let SmartValue::Number(number) = i{
                    sum += number;
                }
            }
            return Ok(vec![SmartValue::Number(sum)]);
        }
        return Err(Box::new(TypeMismatchError{}))

    }

    pub fn sin_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0]{
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().sin()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sin", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn cos_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cos()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "cos", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn tan_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().tan()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "tan", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn mod_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 2{
            if let SmartValue::Number(value) = input[0]{
                if let SmartValue::Number(modder) = input[1]{
                    if value.round() != value || modder.round() != modder{
                        return Err(Box::new(TypeMismatchError{})) // Should be whole numbers.
                    }
                    let (value, modder) = (value.to_i128().unwrap(), modder.to_i128().unwrap());
                    return Ok(vec![SmartValue::Number(Decimal::from_i128(value % modder).unwrap())]); //this looks ugly, but it works...
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "mod", found:input.len(), requires:2})) // any more than 1 input = error
    }

    pub fn sec_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().cos()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sec", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn csc_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().sin()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "csc", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn cot_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cos() / number.to_f64().unwrap().sin()).unwrap() );
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "cot", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn cosh_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cosh()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "cosh", found:input.len(), requires:1}))
    }

    pub fn sinh_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().sinh()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sinh", found:input.len(), requires:1}))
    }

    pub fn tanh_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().tanh()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "tanh", found:input.len(), requires:1}))
    }

    pub fn acos_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().acos()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "acos", found:input.len(), requires:1}))
    }

    pub fn asin_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().asin()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "asin", found:input.len(), requires:1}))
    }

    pub fn atan_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().atan()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "atan", found:input.len(), requires:1}))
    }

    pub fn log_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().log10()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "log", found:input.len(), requires:1}))
    }

    pub fn ln_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().ln()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "ln", found:input.len(), requires:1}))
    }

    pub fn mul_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>> {
        if input.len() != 0 {
            let mut mul = Decimal::from(1);
            for i in input {
                if let SmartValue::Number(number) = i {
                    mul *= number;
                }
            }
            return Ok(vec![SmartValue::Number(mul)]);
        }
        return Err(Box::new(TypeMismatchError {}))
    }

    pub fn max_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() != 0{
            let mut max = Decimal::from(i64::min_value());
            for i in input{
                if let SmartValue::Number(number) = i{
                    if number > max{max = number;}
                }
            }
            return Ok(vec![SmartValue::Number(max)]);
        }
        return Err(Box::new(TypeMismatchError{}))

    }

    pub fn min_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() != 0{
            let mut min = Decimal::from(i64::max_value());
            for i in input{
                if let SmartValue::Number(number) = i{
                    if number < min{min = number;}
                }
            }
            return Ok(vec![SmartValue::Number(min)]);
        }
        return Err(Box::new(TypeMismatchError{}))

    }

    pub fn avg_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        let size = Decimal::from_usize(input.len()).unwrap();
        if input.len() != 0{
            let mut avg = Decimal::from(0);
            for i in input{
                if let SmartValue::Number(number) = i{
                    avg += number.div(size);
                }
            }
            return Ok(vec![SmartValue::Number(avg)]);
        }
        return Err(Box::new(TypeMismatchError{}))

    }

    pub fn stdev_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        let size = Decimal::from_usize(input.len()).unwrap();
        let copy = input.clone();
        if input.len() != 0{
            let mut avg = Decimal::from(0);
            let mut val = Decimal::from(0);
            for i in input{
                if let SmartValue::Number(number) = i{
                    avg += number.div(size);
                }
            }
            for  i in copy{
                if let SmartValue::Number(number) = i{
                    val += (number*number) - (Decimal::from(2)*(number*avg)) + (avg * avg);
                }
            }
            let stdev = SmartValue::Number(Decimal::from_f64((val/size).to_f64().unwrap().sqrt()).unwrap());
            return Ok(vec![stdev]);
        }
        return Err(Box::new(TypeMismatchError{}))
    }

    pub fn mag_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() != 0{
            let mut value = Decimal::from(0);
            for i in input{
                if let SmartValue::Number(number) = i{
                    value += (number*number);
                }
            }
            return Ok(vec![SmartValue::Number(Decimal::from_f64(value.to_f64().unwrap().sqrt()).unwrap())])
        }
        return Err(Box::new(TypeMismatchError{}))
    }

    pub fn angle_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 2{
            if let SmartValue::Number(num1) = input[0]{
                if let SmartValue::Number(num2) = input[1]{
                    let angle = (num2.to_f64().unwrap()/num1.to_f64().unwrap()).atan();
                    let angle = Decimal::from_f64(angle).unwrap();
                    let number = SmartValue::Number(angle);
                    return Ok(vec![number]);
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError {name: "angle", found:input.len(), requires:2}))
    }
}










