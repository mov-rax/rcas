use rust_decimal::Decimal;
use crate::rcas_lib::SmartValue;
use rust_decimal::prelude::*;

pub struct ConstantController;

impl ConstantController{
    pub fn get(identifier:&str) -> Option<SmartValue>{

         match identifier{
            "PI" => Some(SmartValue::Number(Decimal::from_f64(std::f64::consts::PI).unwrap())),
            "E" => Some(SmartValue::Number(Decimal::from_f64(std::f64::consts::E).unwrap())),
             "TAU" => Some(SmartValue::Number(Decimal::from_f64(std::f64::consts::TAU).unwrap())), //2 times pi
             "PHI" => Some(SmartValue::Number(Decimal::from_f64(1.61803398874989484820).unwrap())), //the golden ratio
             "W" => Some(SmartValue::Number(Decimal::from_f64(1.61803398874989484820).unwrap())), //Wallis constant
             "H" => Some(SmartValue::Number(Decimal::from_f64(6.626068 * 10.0_f64.powf(-34.00)).unwrap())), //PLanck's
             "MOL" => Some(SmartValue::Number(Decimal::from_f64(6.0221515 * 10.0_f64.powf(23.00)).unwrap())), //Avogrado's Number
             "C" => Some(SmartValue::Number(Decimal::from_f64(299792458.00_f64).unwrap())), //speed of light
             "G" => Some(SmartValue::Number(Decimal::from_f64(6.67300 * 10.0_f64.powf(-11.0)).unwrap())), //gravitational constant
             "KE" => Some(SmartValue::Number(Decimal::from_f64(8.9875517923*10.0_f64.powf(9.00)).unwrap())), //Coulomb's constant
            _ => None
        }
    }
}