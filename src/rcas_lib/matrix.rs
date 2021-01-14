use rust_decimal::Decimal;
use crate::rcas_lib::{SmartValue, TypeMismatchError};
use std::error;
use std::fmt::{Debug, Formatter, Display};
use rust_decimal::prelude::ToPrimitive;

#[derive(Clone, PartialEq)]
pub struct SmartMatrix{
    data: Vec<SmartValue>,
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
            .filter_map(|x| if let SmartValue::SemiColon = *x{
                return None
            } else {
                Some(x.clone())
            }).collect::<Vec<SmartValue>>();

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
    pub fn get(&self, col:usize, row:usize) -> Option<&SmartValue>{
        if col > 0 && row > 0{
            let base = self.col * (row - 1);
            let index = base + col - 1;
            return Some(&self.data[index])
        }
        None
    }

    /// Set a value in a matrix
    ///
    /// If it was properly set it will return a Some(())
    pub fn set(&mut self, col:usize, row:usize, value:SmartValue) -> Option<()>{
        if col > 0 && row > 0{
            let base = self.col * (row - 1);
            let index = base + col - 1;
            self.data[index] = value;
            return Some(())
        }
        None
    }

    pub fn set_from(&mut self, index_mat:&SmartMatrix, value:SmartValue) -> Option<()>{
        if index_mat.len() == 1{
            if let SmartValue::Number(index) = &index_mat.data[0]{
                if let Some(index) = index.to_usize(){
                    if index > 0{
                        self.data[(index) - 1 as usize] = value;
                        return Some(())
                    }
                }
            }
        } else if index_mat.len() == 2{
            if let SmartValue::Number(col) = &index_mat.data[0]{
                if let SmartValue::Number(row) = &index_mat.data[1]{
                    if let Some(col) = col.to_usize(){
                        if let Some(row) = row.to_usize(){
                            self.set(col, row, value);
                            return Some(())
                        }
                    }
                }
            }
        }
        None
    }

    /// Get a mutable reference to a value in the matrix
    pub fn get_mut(&mut self, col:usize, row:usize) -> Option<&mut SmartValue>{
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
                temp.push_str(self.get(j,i).unwrap().get_value(false).as_str());
                temp.push('\t');
            }
            temp.push('\n');
        }
        temp.pop(); // removes the last newline
        write!(f, "{}", temp)
    }
}