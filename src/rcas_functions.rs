// This implements functions, which are used in rcas_lib and are known as
// SmartValue::DedicatedFunction.

use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::any::Any;
use std::fmt::Display;
use std::error::Error;
use std::fmt;
use statrs;
use crate::rcas_lib::{SmartValue, FormattingError, TypeMismatchError, IncorrectNumberOfArgumentsError, Command, NegativeNumberError, OverflowError};

///Shows to the world all of the standard functions given by default.
pub static STANDARD_FUNCTIONS:[&str;15] = ["cos", "sin", "tan", "sec", "csc", "cot", "mod", "plot", "sum", "exp", "factorial", "sqrt", "clear", "^", "!"];

pub enum Function{
    Standard(Box<dyn Fn(Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>>),
    Nil
}

impl Function {
    pub fn get(identifier:&str) -> Self {
        match identifier{
            "cos" => Self::Standard(Box::new(cos_f)),
            "sin" => Self::Standard(Box::new(sin_f)),
            "tan" => Self::Standard(Box::new(tan_f)),
            "sec" => Self::Standard(Box::new(sec_f)),
            "csc" => Self::Standard(Box::new(csc_f)),
            "cot" => Self::Standard(Box::new(cot_f)),
            "mod" => Self::Standard(Box::new(mod_f)),
            "sum" => Self::Standard(Box::new(sum_f)),
            "exp" => Self::Standard(Box::new(exp_f)),
            "^" => Self::Standard(Box::new(exp_f)),
            "factorial" => Self::Standard(Box::new(factorial_f)),
            "!" => Self::Standard(Box::new(factorial_f)),
            "sqrt" => Self::Standard(Box::new(sqrt_f)),
            "clear" => Self::Standard(Box::new(clear_v)),
            _ => Self::Nil // Returned if function identifier does not exist.
        }
    }
}

pub fn clear_v(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>> {
    if input.is_empty(){
        return Ok(vec![SmartValue::Cmd(Command::ClearScreen)])
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError{name:"clear", found:input.len(), requires:0}))
}

pub fn sqrt_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
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

pub fn factorial_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
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

pub fn exp_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 2{
        if let SmartValue::Number(base) = input[0]{
            if let SmartValue::Number(exponent) = input[1]{
                let number = base.to_f64().unwrap().powf(exponent.to_f64().unwrap());
                let number = Decimal::from_f64(number).unwrap();
                let number = SmartValue::Number(number);
                return Ok(vec![number]);
            }
        }
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError {name: "exp", found:input.len(), requires:2}))
}

pub fn sum_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
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

pub fn sin_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0]{
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().sin()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sin", found:input.len(), requires:1})) // any more than 1 input = error
}

pub fn cos_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cos()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError{name: "cos", found:input.len(), requires:1})) // any more than 1 input = error
}

pub fn tan_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().tan()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError{name: "tan", found:input.len(), requires:1})) // any more than 1 input = error
}

pub fn mod_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
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

pub fn sec_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().cos()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sec", found:input.len(), requires:1})) // any more than 1 input = error
}

pub fn csc_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().sin()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError{name: "csc", found:input.len(), requires:1})) // any more than 1 input = error
}

pub fn cot_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cos() / number.to_f64().unwrap().sin()).unwrap() );
            return Ok(vec![value])
        }
    }
    return Err(Box::new(IncorrectNumberOfArgumentsError{name: "cot", found:input.len(), requires:1})) // any more than 1 input = error
}
