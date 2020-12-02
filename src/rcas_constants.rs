use rust_decimal::Decimal;
use crate::rcas_lib::SmartValue;
use rust_decimal::prelude::*;

pub struct ConstantController;

impl ConstantController{
    pub fn get(identifier:&str) -> Option<SmartValue>{

         match identifier{
            "PI" => Some(SmartValue::Number(Decimal::from_f64(std::f64::consts::PI).unwrap())),
            "E" => Some(SmartValue::Number(Decimal::from_f64(std::f64::consts::E).unwrap())),
            _ => None
        }
    }
}