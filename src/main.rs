use std::ops::Deref;
use crate::rcas_lib::{composer, calculate, Wrapper};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromStr, ToPrimitive, FromPrimitive};

mod rcas_lib;
mod rcas_functions;

fn main() {
    let expression = "sin(2)";
    rcas_lib::RCas::query(expression);
    // let magic = rcas_lib::parser("4*sin(2)");
    // if let Ok(magic) = magic{
    //     rcas_lib::print_sv_vec(&magic);
    // }
    // let test = rcas_functions::SmartFunction::get("cos");
    // if let rcas_functions::SmartFunction::Mono(func) = test{
    //     let value:Decimal = func(Decimal::from(10));
    //     println!("{}", value);
    // }

    // let parser = rcas_lib::parser;
    // let printer = rcas_lib::print_sv_vec;
    // println!("Here is some testing!");
    // println!("Expression: {}", &expression);
    // let mut conv = parser(expression);
    // match conv{
    //     Ok(val) => {printer(&val);
    //         let mut val = val;
    //         println!("solution:");
    //         calculate(&mut val);
    //         printer(&val);
    //         //wrap time
    //         println!("Wrapping time!");
    //         let wrap = rcas_lib::Wrapper::compose(val);
    //         wrap.print_raw();
    //     },
    //     Err(e) => { println!("oof");
    //         println!("{}", &e);
    //     },
    // }

}
