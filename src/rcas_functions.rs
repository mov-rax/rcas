// This implements functions, which are used in rcas_lib and are known as
// SmartValue::DedicatedFunction.

use rust_decimal::Decimal;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::any::Any;
use std::fmt::Display;
use std::error::Error;
use std::fmt;
use statrs;
use crate::rcas_lib::{SmartValue, FormattingError, TypeMismatchError, IncorrectNumberOfArgumentsError, Command, NegativeNumberError, OverflowError, RCas, Wrapper, CalculationMode};
use std::ops::Div;
use crate::rcas_lib::DataType::Number;
use fxhash::FxHashMap;
use std::rc::Rc;
use std::cell::RefCell;


pub enum Function{
    Standard(Box<dyn Fn(&mut FunctionController, Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>>),
    Nil
}

impl PartialEq for Function{
    fn eq(&self, other: &Self) -> bool {
        return match self {
            Function::Standard(_) => {
                if let Function::Standard(_) = other {
                    true
                } else {
                    false
                }
            },
            Function::Nil => {
                if let Function::Nil = other {
                    return true;
                }
                false
            },
        }
    }

    fn ne(&self, other: &Self) -> bool {
        return match self {
            Function::Standard(_) => {
                if let Function::Standard(_) = other {
                    false
                } else {
                    true
                }
            },
            Function::Nil => {
                if let Function::Nil = other {
                    return false;
                }
                true
            },
        }
    }
}

pub struct FunctionController{
    environment: Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>,
    custom_function_id: String, // used just for a current custom function
    custom_function: Vec<SmartValue>, //used for a current custom function
    mode: CalculationMode,
}

impl FunctionController {

    /// Creates a new FunctionController that requires an environment to be given.
    pub fn new(environment:Rc<RefCell<FxHashMap<String, Vec<SmartValue>>>>) -> Self{
        FunctionController { environment , custom_function_id: String::new(), custom_function: Vec::new(), mode: CalculationMode::Radian}
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
            "percenterror" => Function::Standard(Box::new(Self::percenterror_f)),
            "permutations" => Function::Standard(Box::new(Self::permutations_f)),
            "combinations" => Function::Standard(Box::new(Self::combinations_f)),
            "floor" => Function::Standard(Box::new(Self::floor_f)),
            "ceil" => Function::Standard(Box::new(Self::ceil_f)),
            "round" => Function::Standard(Box::new(Self::round_f)),
            "degtorad" => Function::Standard(Box::new(Self::degtorad_f)),
            "radtodeg" => Function::Standard(Box::new(Self::radtodeg_f)),
            "variance" => Function::Standard(Box::new(Self::variance_f)),
            "fg" => Function::Standard(Box::new(Self::fg_f)),
            "fe" => Function::Standard(Box::new(Self::fe_f)),
            "clear" => Function::Standard(Box::new(Self::clear_v)),
            "drop" => Function::Standard(Box::new(Self::drop_v)),
            "setmode" => Function::Standard(Box::new(Self::setmode_f)),
            func => {
                let environment = self.environment.try_borrow();
                if let Ok(environment) = environment{
                    if let Some(value) = environment.get(func){
                        self.custom_function_id = func.to_string();
                        self.custom_function = value.clone();
                        return Function::Standard(Box::new(Self::custom_function_f))
                    }
                }
                Function::Nil
            } // Returned if function identifier does not exist.
        }
    }

    fn deg_to_rad(&self, input:&mut Vec<SmartValue>){
        let con = Decimal::from_f64(std::f64::consts::PI).unwrap();
        let con2 = Decimal::from_f64(180.0).unwrap();
        let con = con/con2;
        if self.mode == CalculationMode::Degree{
            for value in input{
                if let SmartValue::Number(dec) = value{
                    *dec = *dec * con;
                }
            }
        }
    }

    pub fn setmode_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>> {
        if input.len() == 1{
            if let SmartValue::Text(string) = &input[0]{
                match &**string {
                    "rad" => {
                        self.mode = CalculationMode::Radian;
                        return Ok(vec![SmartValue::Cmd(Command::SetMode(CalculationMode::Radian))]);
                    },
                    "deg" => {
                        self.mode = CalculationMode::Degree;
                        return Ok(vec![SmartValue::Cmd(Command::SetMode(CalculationMode::Degree))]);
                    },
                    _ => {
                        return Err(Box::new(TypeMismatchError {}))
                    }
                }
            }
        }
        Err(Box::new(IncorrectNumberOfArgumentsError{ name:"set_mode", found: input.len(), requires: 1}))
    }

    pub fn custom_function_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>> {

        let mut cas = RCas::new();

        fn variable_replacement_loop(input: &SmartValue, value:&String, replacer:&SmartValue) -> SmartValue{
            if let SmartValue::Placeholder(holder) = input.clone(){
                let answer = holder.iter().map(|s| {
                    if let SmartValue::Variable(id) = s{
                        return replacer.clone()
                    }
                    if let SmartValue::Placeholder(_) = s {
                        return variable_replacement_loop(s, value, replacer)
                    }
                    s.clone()
                }).collect::<Vec<SmartValue>>();
                return SmartValue::Placeholder(answer);
            }
            SmartValue::Error("idk fam".to_string())
        }

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
                    if let SmartValue::Placeholder(_) = s{
                        return variable_replacement_loop(s, value, &input[index])
                    }
                    s.clone()
                }).collect();
            }

            let mut answer = RCas::composer(answer);
            answer = cas.recurse_solve(answer);
            return Ok(answer)
        }
        Err(Box::new(TypeMismatchError{}))
    }

    pub fn drop_v(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        let mut environment = self.environment.borrow_mut();
        for value in &input{
            if let SmartValue::Text(id) = value{
                environment.remove(id);
            } else {
                return Err(Box::new(TypeMismatchError {}))
            }
        }
        Ok(vec![SmartValue::Cmd(Command::RefreshEnvironment)])
    }

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

    pub fn sin_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0]{
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().sin()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sin", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn cos_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
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

    pub fn sec_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().cos()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sec", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn csc_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(1.0 / number.to_f64().unwrap().sin()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "csc", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn cot_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cos() / number.to_f64().unwrap().sin()).unwrap() );
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "cot", found:input.len(), requires:1})) // any more than 1 input = error
    }

    pub fn cosh_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().cosh()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "cosh", found:input.len(), requires:1}))
    }

    pub fn sinh_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().sinh()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "sinh", found:input.len(), requires:1}))
    }

    pub fn tanh_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().tanh()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "tanh", found:input.len(), requires:1}))
    }

    pub fn acos_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().acos()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "acos", found:input.len(), requires:1}))
    }

    pub fn asin_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
        if input.len() == 1{
            if let SmartValue::Number(number) = input[0] {
                let value = SmartValue::Number(Decimal::from_f64(number.to_f64().unwrap().asin()).unwrap());
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "asin", found:input.len(), requires:1}))
    }

    pub fn atan_f(&mut self, mut input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        self.deg_to_rad(&mut input);
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

    pub fn percenterror_f (&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 2{
            if let SmartValue::Number(means) = input[0]{
                if let SmartValue::Number(theo) = input[1]{
                    let percenterror = ((means.to_f64().unwrap() - theo.to_f64().unwrap()).abs() / (theo.to_f64().unwrap()).abs()) * 100.00;
                    let percenterror = Decimal::from_f64(percenterror).unwrap();
                    let value = SmartValue::Number(percenterror);
                    return Ok(vec![value]);
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "percenterror", found:input.len(), requires:2}))
    }

    pub fn permutations_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 2{
            if let SmartValue::Number(n) = input[0]{
                if n.is_sign_negative(){ // ensures that the number cannot be negative
                    return Err(Box::new(NegativeNumberError{}))
                }
                if let SmartValue::Number(r) = input[1]{
                    if (n-r).is_sign_negative(){ // ensures that the number cannot be negative
                        return Err(Box::new(NegativeNumberError{}))
                    }
                    let permutation = statrs::function::gamma::gamma(n.to_f64().unwrap()+1.0) / statrs::function::gamma::gamma((n-r).to_f64().unwrap()+1.0);
                    let permutation = Decimal::from_f64(permutation.round()).unwrap();
                    let value = SmartValue::Number(permutation);
                    return Ok(vec![value]);
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "permutations", found:input.len(), requires:2}))
    }

    pub fn combinations_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 2{
            if let SmartValue::Number(n) = input[0]{
                if n.is_sign_negative(){ // ensures that the number cannot be negative
                    return Err(Box::new(NegativeNumberError{}))
                }
                if let SmartValue::Number(r) = input[1]{
                    if (n-r).is_sign_negative(){ // ensures that the number cannot be negative
                        return Err(Box::new(NegativeNumberError{}))
                    }
                    if r.is_sign_negative(){
                        return Err(Box::new(NegativeNumberError{}))
                    }
                    let combinations = statrs::function::gamma::gamma(n.to_f64().unwrap()+1.0) / (statrs::function::gamma::gamma((r).to_f64().unwrap()+1.0) * statrs::function::gamma::gamma((n-r).to_f64().unwrap()+1.0));
                    let combinations = Decimal::from_f64(combinations.round()).unwrap();
                    let value = SmartValue::Number(combinations);
                    return Ok(vec![value]);
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "combinations", found:input.len(), requires:2}))
    }

    pub fn floor_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(n) = input[0]{
                let floor = n.to_f64().unwrap().floor();
                let floor = Decimal::from_f64(floor).unwrap();
                let value = SmartValue::Number(floor);
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "floor", found:input.len(), requires:1}))
    }

    pub fn ceil_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(n) = input[0]{
                let ceil = n.to_f64().unwrap().ceil();
                let ceil = Decimal::from_f64(ceil).unwrap();
                let value = SmartValue::Number(ceil);
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "ceil", found:input.len(), requires:1}))
    }

    pub fn round_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(n) = input[0]{
                let round = n.to_f64().unwrap().round();
                let round = Decimal::from_f64(round).unwrap();
                let value = SmartValue::Number(round);
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "round", found:input.len(), requires:1}))
    }

    pub fn degtorad_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
        if input.len() == 1{
            if let SmartValue::Number(n) = input[0]{
                let deg2rad = n.to_f64().unwrap() * ((103993.0/33102.0)/180.0);
                let deg2rad = Decimal::from_f64(deg2rad).unwrap();
                let value = SmartValue::Number(deg2rad);
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "deg2rad", found:input.len(), requires:1}))
    }

    pub fn radtodeg_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>> {
        if input.len() == 1 {
            if let SmartValue::Number(n) = input[0] {
                let rad2deg = n.to_f64().unwrap() * (180.0 / (103993.0 / 33102.0));
                let rad2deg = Decimal::from_f64(rad2deg).unwrap();
                let value = SmartValue::Number(rad2deg);
                return Ok(vec![value])
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError { name: "rad2deg", found: input.len(), requires: 1 }))
    }
    pub fn variance_f(&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{
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
            let var = SmartValue::Number(Decimal::from_f64((val/size).to_f64().unwrap()).unwrap());
            return Ok(vec![var]);
        }
        return Err(Box::new(TypeMismatchError{}))
    }

    pub fn fg_f (&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{ // the force of gravity
        if input.len() == 3{
            if let SmartValue::Number(m1) = input[0]{
                if m1.is_sign_negative(){ // ensures that the number cannot be negative
                    return Err(Box::new(NegativeNumberError{}))
                }
                if let SmartValue::Number(m2) = input[1]{
                    if m2.is_sign_negative(){ // ensures that the number cannot be negative
                        return Err(Box::new(NegativeNumberError{}))
                    }
                    if let SmartValue::Number(r) = input[2]{
                        if r.is_sign_negative(){ // ensures that the number cannot be negative
                            return Err(Box::new(NegativeNumberError{}))
                        }
                        let G = 6.67 * (10.0_f64.powf(-11.00));
                        let fg = (G * m1.to_f64().unwrap()*m2.to_f64().unwrap()) / (r.to_f64().unwrap().powf(2.00));
                        let fg = Decimal::from_f64(fg).unwrap();
                        let value = SmartValue::Number(fg);
                        return Ok(vec![value])
                    }
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "fg", found:input.len(), requires:3}))
    }

    pub fn fe_f (&mut self, input:Vec<SmartValue>) -> Result<Vec<SmartValue>, Box<dyn std::error::Error>>{ // electrostatic force
        if input.len() == 3{
            if let SmartValue::Number(q1) = input[0]{
                if let SmartValue::Number(q2) = input[1]{
                    if let SmartValue::Number(d) = input[2]{
                        if d.is_sign_negative(){ // ensures that the number cannot be negative
                            return Err(Box::new(NegativeNumberError{}))
                        }
                        let k = 8.987 * (10.0_f64.powf(9.0));
                        let fe = (k * q1.to_f64().unwrap().abs()*q2.to_f64().unwrap().abs()) / (d.to_f64().unwrap().powf(2.00));
                        let fe = Decimal::from_f64(fe).unwrap();
                        let value = SmartValue::Number(fe);
                        return Ok(vec![value])
                    }
                }
            }
        }
        return Err(Box::new(IncorrectNumberOfArgumentsError{name: "fe", found:input.len(), requires:3}))
    }
}













