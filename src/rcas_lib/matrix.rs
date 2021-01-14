use rust_decimal::Decimal;
use crate::rcas_lib::{SmartValue, TypeMismatchError};
use std::error;
use std::fmt::{Debug, Formatter, Display};

#[derive(Clone, PartialEq)]
pub struct SmartMatrix{
    data: Vec<Decimal>,
    row: usize,
    col: usize,
}

impl SmartMatrix{
    /// Create a SmartMatrix from a slice of SmartValues with Type SmartValue::Number(_)
    ///
    /// - If any element in `input` is not a SmartValue::Number(_) an `Err` will be returned.
    pub fn magic(input: &[SmartValue]) -> Result<Self, Box<dyn error::Error>>{
        let col = input.iter()
            .take_while(|x| **x != SmartValue::SemiColon)
            .count();
        let row = if col < input.len(){
            input.len()/col
        } else {
            1
        };

        let data = input.iter()
            .filter_map(|x| if let SmartValue::Number(number) = *x{
                return Some(number);
            } else {
                None
            }).collect::<Vec<Decimal>>();

        if data.len() != row*col{
            return Err(Box::new(TypeMismatchError{}))
        }
        Ok(Self {
            data,
            col,
            row
        })
    }

    /// Return a value in a matrix
    pub fn get(&self, col:usize, row:usize) -> Option<Decimal>{
        if col > 0 && row > 0{
            let base = self.col * (row - 1);
            let index = base + col - 1;
            return Some(self.data[index])
        }
        None
    }

    /// Set a value in a matrix
    ///
    /// If it was properly set it will return a Some(())
    pub fn set(&mut self, col:usize, row:usize, value:Decimal) -> Option<()>{
        if col > 0 && row > 0{
            let base = self.col * (row - 1);
            let index = base + col - 1;
            self.data[index] = value;
            return Some(())
        }
        None
    }

    /// Get a mutable reference to a value in the matrix
    pub fn get_mut(&mut self, col:usize, row:usize) -> Option<&mut Decimal>{
        if col > 0 && row > 0 {
            let base = self.col * (row - 1);
            let index = base + col - 1;
            return Some(&mut self.data[index])
        }
        None
    }

    /// Gets the number of elements in the matrix
    pub fn len(&self) -> usize{
        self.data.len()
    }

    /// Gets the number of rows in the matrix
    pub fn rows(&self) -> usize{
        self.row
    }

    // Gets the number of columns in the matrix
    pub fn cols(&self) -> usize{
        self.col
    }
}

impl Debug for SmartMatrix{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.data.iter())
            .finish()
    }
}

impl Display for SmartMatrix{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut temp = String::new();
        for i in 1..=self.row{
            for j in 1..=self.col{
                temp.push_str(self.get(j,i).unwrap().to_string().as_str());
                temp.push('\t');
            }
            temp.push('\n');
        }
        temp.pop(); // removes the last newline
        write!(f, "{}", temp)
    }
}