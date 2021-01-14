use rust_decimal::Decimal;
use crate::rcas_lib::{SmartValue, TypeMismatchError, IncorrectNumberOfArgumentsError, IndexOutOfBoundsError};
use std::error;
use std::fmt::{Debug, Formatter, Display};
use rust_decimal::prelude::ToPrimitive;
use crate::rcas_functions::FunctionController;
use core::ops::RangeInclusive;

#[derive(Clone, PartialEq)]
pub struct SmartMatrix{
    id: String, // identifier that should be named the same as it is in the environment (if it is in the environment)
    data: Vec<SmartValue>,
    row: usize,
    col: usize,
}

impl SmartMatrix{
    /// Create a SmartMatrix from a slice of SmartValues with Type SmartValue::Number(_)
    ///
    /// - If any element in `input` is not a SmartValue::Number(_) an `Err` will be returned.
    /// - Internal ID defaults to Name 'Matrix'
    pub fn new_from(input: &[SmartValue]) -> Result<Self, Box<dyn error::Error>>{
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
            return Err(Box::new(IncorrectNumberOfArgumentsError{
                name: "Matrix",
                found: data.len(),
                requires: row*col
            }))
        }
        Ok(Self {
            id: "Matrix".to_string(),
            data,
            col,
            row
        })
    }


    pub fn new_from_1d_range(mat:&SmartMatrix, range: RangeInclusive<usize>) -> Self{
        let data = (&mat.data[range.clone()]).iter().cloned().collect::<Vec<SmartValue>>();
        Self {
            id: "Matrix".to_string(),
            data,
            col: range.count(),
            row: 1
        }
    }

    pub fn new_from_2d_range(mat:&SmartMatrix, row_range: RangeInclusive<usize>, col_range:RangeInclusive<usize>) -> Self{
        let mut data = Vec::new();
        for row in row_range.clone(){
            for col in col_range.clone(){
                data.push(unsafe {mat.get_unchecked(row, col).clone()});
            }
        }
        Self {
            id: "Matrix".to_string(),
            data,
            col: col_range.count(),
            row: row_range.count()
        }
    }

    /// Return a value in a matrix
    pub fn get(&self, row:usize, col:usize) -> Option<&SmartValue>{
        if col > 0 && row > 0{
            let base = self.col * (row - 1);
            let index = base + col - 1;
            return Some(&self.data[index])
        }
        None
    }

    unsafe fn get_unchecked(&self, row:usize, col:usize) -> &SmartValue{
        let base = self.col * (row - 1);
        let index = base + col - 1;
        &self.data[index]
    }
    // x[1:10,1]

    pub fn get_from(&self, index_mat:&SmartMatrix) -> Result<SmartValue, Box<dyn error::Error>>{
        let wrong_type_error = || Box::new(TypeMismatchError{
            found_in: self.id.clone(),
            found_type: "Number".to_string(),
            required_type: "Natural Number"
        });
        // If any of the bounds inserted are beyond this matrix's matrix, it will
        // return the Err.
        // This is used for 1-dimensional indexing
        let check_len = |a:usize, b:usize| {
            if a > self.data.len(){
                return Err(IndexOutOfBoundsError{ found_index: (a as isize), max_index: self.data.len() })
            } else if b > self.data.len(){
                return Err(IndexOutOfBoundsError{ found_index: (b as isize), max_index: self.data.len() })
            }
            Ok(())
        };
        // If any of the bounds inserted are beyond this matrix's limits, it will
        // return the Err(Box<IndexOutOfBoundsError>)
        // This is used for 2-dimensional indexing
        let check_bounds = |a1:usize, b1:usize, a2:usize, b2:usize| {
            let mut result = None;
            if a1 > self.row{
                result = Some((a1, self.row))
            } else if a2 > self.col{
                result = Some((a2, self.col))
            } else if b1 > self.row{
                result = Some((b1, self.row))
            } else if b2 > self.col{
                result = Some((b2, self.col))
            }
            if let Some((bad_index, max_index)) = result{
                return Err(IndexOutOfBoundsError{ found_index: (bad_index as isize), max_index })
            }
            Ok(())
        };

        if index_mat.data.len() == 1{
            if let SmartValue::Range(bound1, step, bound2) = index_mat.data[0]{
                if step == Decimal::from(1){ // discrete
                    let bound1 = bound1.to_usize().ok_or_else(|| wrong_type_error())?;
                    let bound2 = bound2.to_usize().ok_or_else(|| wrong_type_error())?;
                    let (a, b) = if bound1 < bound2 {(bound1, bound2)} else {(bound2,bound1)};
                    let _ = check_len(a,b)?;
                    return Ok(SmartValue::Matrix(Self::new_from_1d_range(&self, (a-1)..=(b-1))))
                } else {
                    return Err(wrong_type_error())
                }
            } else if let SmartValue::Number(val) = index_mat.data[0]{
                let index = val.to_usize().ok_or_else(|| wrong_type_error())?;
                let _ = check_len(index,index)?;
                return if index > 0 {
                    Ok(self.data[index - 1].clone())
                } else {
                    Err(wrong_type_error())
                }

            } else { // Value in index is not a Range or a Natural Number
                return Err(Box::new(TypeMismatchError{
                    found_in: self.id.clone(),
                    found_type: FunctionController::internal_type_of(&index_mat.data[0]),
                    required_type: "Natural Number"
                }));
            }
        } else if index_mat.data.len() == 2{
            // Range,Range & Range,Natural Number
            if let SmartValue::Range(bound_01, step_0, bound_02) = index_mat.data[0]{
                if let SmartValue::Range(bound_11, step_1, bound_12) = index_mat.data[1]{
                    return if step_0 == Decimal::from(1) && step_1 == Decimal::from(1) { // discrete
                        let bound_01 = bound_01.to_usize().ok_or_else(|| wrong_type_error())?;
                        let bound_02 = bound_02.to_usize().ok_or_else(|| wrong_type_error())?;
                        let bound_11 = bound_11.to_usize().ok_or_else(|| wrong_type_error())?;
                        let bound_12 = bound_12.to_usize().ok_or_else(|| wrong_type_error())?;
                        let (a1, b1) = if bound_01 < bound_02 { (bound_01, bound_02) } else { (bound_02, bound_01) };
                        let (a2, b2) = if bound_11 < bound_12 { (bound_11, bound_12) } else { (bound_12, bound_11) };
                        let _ = check_bounds(a1, b1, a2, b2)?;
                        Ok(SmartValue::Matrix(Self::new_from_2d_range(&self, a1..=b1, a2..=b2)))
                    } else {
                        Err(wrong_type_error())
                    }
                } else if let SmartValue::Number(col) = index_mat.data[1]{
                    return if step_0 == Decimal::from(1) {
                        let bound_01 = bound_01.to_usize().ok_or_else(|| wrong_type_error())?;
                        let bound_02 = bound_02.to_usize().ok_or_else(|| wrong_type_error())?;
                        let col = col.to_usize().ok_or_else(|| wrong_type_error())?;
                        let (a, b) = if bound_01 < bound_02 { (bound_01, bound_02) } else { (bound_02, bound_01) };
                        let _ = check_bounds(a,b,col,col)?;
                        Ok(SmartValue::Matrix(Self::new_from_2d_range(&self, a..=b, col..=col)))
                    } else {
                        Err(wrong_type_error())
                    }
                } else {
                    return Err(Box::new(TypeMismatchError{
                        found_in: self.id.clone(),
                        found_type: FunctionController::internal_type_of(&index_mat.data[1]),
                        required_type: "Natural Number"
                    }))
                }
            }
            // Natural Number,Range & Natural Number, Natural Number
            if let SmartValue::Number(row) = index_mat.data[0]{
                return if let SmartValue::Range(bound1, step, bound2) = index_mat.data[1] {
                    let row = row.to_usize().ok_or_else(|| wrong_type_error())?;
                    if step == Decimal::from(1) {
                        let bound1 = bound1.to_usize().ok_or_else(|| wrong_type_error())?;
                        let bound2 = bound2.to_usize().ok_or_else(|| wrong_type_error())?;
                        let (a, b) = if bound1 < bound2 { (bound1, bound2) } else { (bound2, bound1) };
                        let _ = check_bounds(row, row, a, b)?;
                        Ok(SmartValue::Matrix(Self::new_from_2d_range(&self, row..=row, a..=b)))
                    } else {
                        Err(wrong_type_error())
                    }
                } else if let SmartValue::Number(col) = index_mat.data[1] {
                    let row = row.to_usize().ok_or_else(|| wrong_type_error())?;
                    let col = col.to_usize().ok_or_else(|| wrong_type_error())?;
                    let _ = check_bounds(row, row, col, col)?;
                    Ok(unsafe { self.get_unchecked(row, col).clone() })
                } else { // col index is not a Natural Number or a Range
                    Err(Box::new(TypeMismatchError {
                        found_in: self.id.clone(),
                        found_type: FunctionController::internal_type_of(&index_mat.data[1]),
                        required_type: "Natural Number"
                    }))
                }
            }
            // If the row index is not a Natural Number or a Range
            return Err(Box::new(TypeMismatchError{
                found_in: self.id.clone(),
                found_type: FunctionController::internal_type_of(&index_mat.data[0]),
                required_type: "Natural Number"
            }))

        }

        Err(Box::new(IncorrectNumberOfArgumentsError{
            name: "Matrix",
            found: index_mat.len(),
            requires: 1
        }))
    }

    /// Set a value in a matrix
    ///
    /// If it was properly set it will return a Some(())
    pub fn set(&mut self, row:usize, col:usize, value:SmartValue) -> Option<()>{
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
            if let SmartValue::Number(row) = &index_mat.data[0]{
                if let SmartValue::Number(col) = &index_mat.data[1]{
                    if let Some(col) = col.to_usize(){
                        if let Some(row) = row.to_usize(){
                            self.set(row, col, value);
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
                temp.push_str(self.get(i,j).unwrap().get_value(false).as_str());
                temp.push('\t');
            }
            temp.push('\n');
        }
        temp.pop(); // removes the last newline
        write!(f, "{}", temp)
    }
}