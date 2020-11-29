// This implements functions, which are used in rcas_lib and are known as
// SmartValue::DedicatedFunction.

use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::any::Any;
use std::fmt::Display;
use std::error::Error;
use std::fmt;
use crate::rcas_lib::{SmartValue, FormattingError};

///Shows to the world all of the standard functions given by default.
pub static STANDARD_FUNCTIONS:[&str;7] = ["cos", "sin", "tan", "sec", "csc", "cot", "mod"];
pub static VOID_FUNCTIONS:[&str;1] = ["plot"];

/// An object that returns an enum that contains the function
/// attributed to an identifier.
pub enum SmartFunction{
    ///INPUT => OUTPUT
    Mono(Box<dyn Fn(Decimal) -> Decimal>),
    ///INPUT,INPUT => OUTPUT
    Binary(Box<dyn Fn(Decimal, Decimal) -> Decimal>),
    ///INPUT,INPUT,INPUT,... => OUTPUT
    Poly(Box<dyn Fn(Vec<Decimal>) -> Decimal>),
    ///INPUT,INPUT,INPUT,... => OUTPUT,OUTPUT,OUTPUT,...
    PolyPoly(Box<dyn Fn(Vec<Decimal>) -> Vec<Decimal>>),
    ///INPUT => OUTPUT?
    MonoOpt(Box<dyn Fn(Decimal) -> Option<Decimal>>),
    ///INPUT,INPUT => OUTPUT?
    BinaryOpt(Box<dyn Fn(Decimal, Decimal) -> Option<Decimal>>),
    ///INPUT,INPUT,INPUT,... => OUTPUT?
    PolyOpt(Box<dyn Fn(Vec<Decimal>) -> Option<Decimal>>),
    ///INPUT,INPUT,INPUT,... => OUTPUT?,OUTPUT?,OUTPUT?,...
    PolyPolyOpt(Box<dyn Fn(Vec<Decimal>) -> Option<Vec<Decimal>>>),
    Nil
}

pub enum Function{
    Standard(Box<dyn Fn(Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>>),
    //Void(Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>>>),
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
            _ => Self::Nil // Returned if function identifier does not exist.
        }
    }
}

impl SmartFunction{
    pub fn get(identifier:&str) -> Self{
        match identifier{
            "cos" => Self::Mono(Box::new(cos)),
            "sin" => Self::Mono(Box::new(sin)),
            "tan" => Self::Mono(Box::new(tan)),
            "sec" => Self::Mono(Box::new(sec)),
            "csc" => Self::Mono(Box::new(csc)),
            "cot" => Self::Mono(Box::new(cot)),
            "mod" => Self::BinaryOpt(Box::new(modulo)),
            _ => Self::Nil
        }
    }
}

pub fn sin_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0]{
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().sin()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(FormattingError{ position: 2})) // any more than 1 input = error
}

pub fn cos_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cos()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(FormattingError{ position: 2})) // any more than 1 input = error
}

pub fn tan_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().tan()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(FormattingError{ position: 2})) // any more than 1 input = error
}

pub fn mod_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 2{
        if let SmartValue::Number(value) = input[0]{
            if let SmartValue::Number(modder) = input[1]{
                let (value, modder) = (value.to_i128().unwrap(), modder.to_i128().unwrap());
                return Ok(vec![SmartValue::Number(Decimal::from_i128(value % modder).unwrap())]); //this looks ugly, but it works...
            }
        }
    }
    return Err(Box::new(FormattingError{ position: 2})) // any more than 1 input = error
}

pub fn sec_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().cos()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(FormattingError{ position: 2})) // any more than 1 input = error
}

pub fn csc_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().sin()).unwrap());
            return Ok(vec![value])
        }
    }
    return Err(Box::new(FormattingError{ position: 2})) // any more than 1 input = error
}

pub fn cot_f(input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
    if input.len() == 1{
        if let SmartValue::Number(number) = input[0] {
            let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cos() / number.to_f64().unwrap().sin()).unwrap() );
            return Ok(vec![value])
        }
    }
    return Err(Box::new(FormattingError{ position: 2})) // any more than 1 input = error
}


pub fn sin(input:Decimal) -> Decimal{
    Decimal::from_f64(input.to_f64().unwrap().sin()).unwrap()
}

pub fn cos(input:Decimal) -> Decimal{
    Decimal::from_f64(input.to_f64().unwrap().cos()).unwrap()
}

pub fn tan(input:Decimal) -> Decimal{
    Decimal::from_f64(input.to_f64().unwrap().tan()).unwrap()
}
///Optional, because the DECIMALS are required to be whole numbers
pub fn modulo(input_to_mod:Decimal, modder:Decimal) -> Option<Decimal>{
    if !input_to_mod.to_string().contains(".") && !modder.to_string().contains("."){
        let temp1 = input_to_mod.to_i128().unwrap();
        let temp2 = modder.to_i128().unwrap();
        return Decimal::from_i128(temp1 % temp2)
    }
    None
}

pub fn sec(input:Decimal) -> Decimal{ Decimal::from_f64(1.0/input.to_f64().unwrap().cos()).unwrap() }

pub fn csc(input:Decimal) -> Decimal{ Decimal::from_f64(1.0/input.to_f64().unwrap().sin()).unwrap() }

pub fn cot(input:Decimal) -> Decimal{ Decimal::from_f64(input.to_f64().unwrap().cos()/input.to_f64().unwrap().sin()).unwrap() }