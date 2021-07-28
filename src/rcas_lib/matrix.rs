use rust_decimal::Decimal;
use crate::rcas_lib::{SmartValue, TypeMismatchError, IncorrectNumberOfArgumentsError, IndexOutOfBoundsError, GenericError, DimensionMismatch, AnyError};
use std::error;
use std::fmt::{Debug, Formatter, Display};
use rust_decimal::prelude::{ToPrimitive, Zero};
use crate::rcas_functions::FunctionController;
use core::ops::{RangeInclusive, Range};
use core::cell::Ref;
use core::any::Any;
use fltk::FrameType::DiamondDownBox;
use std::ops::{Add, AddAssign, MulAssign, DivAssign, SubAssign, Index, IndexMut, Mul};
use nalgebra::{Matrix, Dynamic, VecStorage, ComplexField};

// #[derive(Clone, PartialEq)]
// pub struct SmartMatrix{
//     id: String, // identifier that should be named the same as it is in the environment (if it is in the environment)
//     data: Vec<SmartValue>,
//     row: usize,
//     col: usize,
// }

type DecimalMatrix = Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>;
type DoubleMatrix = Matrix<f64, Dynamic, Dynamic, VecStorage<f64, Dynamic, Dynamic>>;

#[derive(Clone, PartialEq, Debug)]
pub struct DynMatrix{
    number_mat: Option<Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>>,
    normal_mat: Option<Vec<SmartValue>>,
}

impl DynMatrix{
    /// Gets a value at an index.
    pub unsafe fn get_unchecked(&self, index:usize) -> SmartValue{
        return if let Some(mat) = self.number_mat.as_ref() {
            SmartValue::Number(mat[index])
        } else {
            let mat = self.normal_mat.as_ref().unwrap();
            mat[index].clone()
        }
    }

    /// Gets a value at an index.
    pub fn get(&self, index:usize) -> Result<SmartValue, Box<dyn error::Error>>{
        return if let Some(mat) = self.number_mat.as_ref(){
            if index < mat.len(){
                Ok(SmartValue::Number(mat[index]))
            } else {
                Err(IndexOutOfBoundsError{ found_index: index as isize, max_index: mat.len() }.into())
            }
        } else {
            let mat = self.normal_mat.as_ref().unwrap();
            if index < mat.len(){
                Ok(mat[index].clone())
            } else {
                Err(IndexOutOfBoundsError{ found_index: index as isize, max_index: mat.len() }.into())
            }
        }
    }

    /// Sets a value at an index.
    ///
    /// - If the value is a Number and the matrix is a Number Matrix, the Number Matrix will remain a Number Matrix.
    /// - If the value is not a Number and the matrix is a Number Matrix, the Number Matrix will turn into a normal Matrix.
    /// - A normal Matrix can only be converted to a Number Matrix by the user.
    pub fn set(&mut self, index:usize, value:SmartValue) -> Result<(), Box<dyn error::Error>>{
        let bound_check = |index:usize, len:usize| if index > len {Some(IndexOutOfBoundsError{ found_index: index as isize, max_index: len })} else { None };

        let mut convert_to_normal = false;
        if let Some(mat) = self.number_mat.as_mut(){
            if let SmartValue::Number(num) = &value{
                match bound_check(index, mat.len()){
                    Some(err) => return Err(err.into()),
                    None => mat[index] = *num,
                }
            } else { // convert to normal_mat
                convert_to_normal = true;
            }
        } else {
            let mat = self.normal_mat.as_mut().unwrap();
            match bound_check(index, mat.len()){
                Some(err) => return Err(err.into()),
                None => mat[index] = value,
            }
            return Ok(()) // No need to possibly convert, therefore just return
        }

        if convert_to_normal{ // converts a number matrix into a normal matrix
            match bound_check(index, self.number_mat.as_ref().unwrap().len()){
                Some(err) => return Err(err.into()),
                _ => {},
            }

            self.convert_to_normal();
            self.normal_mat.as_mut().unwrap()[index] = value;
        }

        Ok(())
    }

    /// Checks if the matrix is a Number Matrix
    pub fn is_number_matrix(&self) -> bool{
       if let Some(_) = self.number_mat.as_ref(){
            return true
        }
        false
    }
    /// Safely converts a Number Matrix to a Normal Matrix
    ///
    /// - If the Matrix is already a normal Matrix, there will be no change.
    pub fn convert_to_normal(&mut self){
        if let Some(_) = self.normal_mat.as_ref(){
            return; // Don't do anything, its already a normal matrix
        }
        let data:Vec<Decimal> = self.number_mat.take().unwrap().data.into();
        let data = data.iter().map(|x| SmartValue::Number(*x)).collect::<Vec<SmartValue>>();
        self.normal_mat = Some(data);
    }

    /// Attempts to convert a Normal Matrix to a Number Matrix
    ///
    /// - Requires `row` and `col` (unlike converting to a normal).
    pub fn try_convert_to_number(&mut self, row:usize, col:usize) -> bool{
        if let Some(_) = self.number_mat.as_ref(){
            return true // All is good, don't do anything.
        }
        let is_all_number = self.normal_mat.as_ref().unwrap().iter().all(|x| if let SmartValue::Number(_) = x { true } else { false });
        if is_all_number{
            let data = self.normal_mat.take().unwrap().iter().map(|x| if let SmartValue::Number(num) = x {*num} else {unreachable!()}).collect::<Vec<Decimal>>();
            let data = VecStorage::new(Dynamic::new(row), Dynamic::new(col), data);
            let data = Matrix::from_data(data);
            self.number_mat = Some(data);
            return true
        }
        false
    }


}

/// A type that supports both a Decimal matrix and a SmartValue matrix.
#[derive(Clone, PartialEq, Debug)]
pub struct SmartMatrix {
    id: String,
    mat: DynMatrix,
    row: usize,
    col: usize,
}

impl SmartMatrix {

    pub fn from_number_data(data:Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>) -> Self{
        let row = data.nrows();
        let col = data.ncols();
        Self {
            id: "Matrix".to_string(),
            mat: DynMatrix { number_mat: Some(data), normal_mat: None },
            row,
            col
        }
    }

    pub fn new_from_matrices(input: &[Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>]) -> Self{
        let col = input.len();
        let mat = input.iter().map(|m| SmartValue::Matrix(SmartMatrix::from_number_data(m.clone()))).collect::<Vec<SmartValue>>();
        Self {
            id: "Matrix".to_string(),
            mat: DynMatrix { number_mat: None, normal_mat: Some(mat) },
            row: 1,
            col
        }
    }

    pub fn new_from_decimals(input: &[Decimal]) -> Self{
        let data = VecStorage::new(Dynamic::new(1), Dynamic::new(input.len()), input.to_vec());
        let mat = Matrix::from_data(data);
        Self {
            id: "Matrix".to_string(),
            mat: DynMatrix { number_mat: Some(mat), normal_mat: None },
            row: 1,
            col: input.len()
        }
    }

    pub fn new_from(input: &[SmartValue]) -> Result<Self, Box<dyn error::Error>>{

        let data = input.iter()
            .filter_map(|x| if let SmartValue::SemiColon = *x{
                return None
            } else {
                Some(x.clone())
            }).collect::<Vec<SmartValue>>();

        let col = input.iter()
            .take_while(|x| **x != SmartValue::SemiColon)
            .count();

        let row = data.len()/col;

        let num_data = data.iter()
            .filter_map(|x| if let SmartValue::Number(num) = *x{
                return Some(num)
            } else {
                None
            }).collect::<Vec<Decimal>>();


        if num_data.len() == data.len(){ // All are numbers, therefore, it will use the number_mat.
            let num_data = VecStorage::new(Dynamic::new(row),Dynamic::new(col), num_data);
            let mat = DynMatrix{ number_mat: Some(Matrix::from_data(num_data)), normal_mat: None };
            return Ok(Self{
                id: "Matrix".to_string(),
                mat,
                row,
                col
            })
        } else{ // Not all are numbers
            return Ok(Self{
                id: "Matrix".to_string(),
                mat: DynMatrix { number_mat: None, normal_mat: Some(data) },
                row,
                col
            })
        }

        Err(IncorrectNumberOfArgumentsError{
            name: "Matrix",
            found: data.len(),
            requires: row*col,
        }.into())

    }

    pub fn new_from_1d_range(mat:&Self, range: Range<usize>) -> Self{
        return if mat.mat.is_number_matrix(){
            let data = mat.mat.number_mat.as_ref().unwrap().data.as_vec()[range].iter().cloned().collect::<Vec<Decimal>>();
            let col = data.len();
            let data = VecStorage::new(Dynamic::new(1), Dynamic::new(col), data);
            Self {
                id: "Matrix".to_string(),
                mat: DynMatrix { number_mat: Some(Matrix::from_data(data)), normal_mat: None },
                row: 1,
                col
            }
        } else {
            let data = mat.mat.normal_mat.as_ref().unwrap().iter().cloned().collect::<Vec<SmartValue>>();
            let col = data.len();
            Self {
                id: "Matrix".to_string(),
                mat: DynMatrix { number_mat: None, normal_mat: Some(data) },
                row: 1,
                col
            }
        }

    }

    pub fn cols(&self) -> usize{
        self.col
    }

    pub fn rows(&self) -> usize{
        self.row
    }

    /// Tries to convert a Normal Matrix into a Number Matrix.
    ///
    /// - Returns `true` if it was successfully converted.
    /// - Returns `false` if it could not convert.
    pub fn try_convert_to_number(&mut self) -> bool{
        self.mat.try_convert_to_number(self.row,self.col)
    }

    pub fn new_from_2d_range(mat:&Self, row_range: Range<usize>, col_range:Range<usize>) -> Self{
        use nalgebra::storage::Storage;

        let row = row_range.end - row_range.start;
        let col = col_range.end - row_range.start;
        return if mat.mat.is_number_matrix(){
            let slice = mat.mat.number_mat.as_ref().unwrap().slice_range(col_range, row_range);
            let matrix:Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>> = Matrix::from(slice);
            Self {
                id: "Matrix".to_string(),
                mat: DynMatrix { number_mat: Some(matrix), normal_mat: None },
                row,
                col
            }
        } else {
            let mut data = Vec::new();
            for row in row_range.clone(){
                for col in col_range.clone(){
                    data.push(unsafe {mat.get_unchecked(row,col)})
                }
            }
            Self {
                id: "Matrix".to_string(),
                mat: DynMatrix { number_mat: None, normal_mat: Some(data) },
                row,
                col
            }
        }

    }

    /// Used to get a value at a location for a non-number matrix.
    ///
    /// - Will panic if beyond the indices of the matrix.
    /// - `row` & `col` start at 0 instead of 1.
    unsafe fn get_unchecked(&self, row:usize, col:usize) -> SmartValue{
        let base = self.col * row;
        let index = base + col;
        self.mat.get_unchecked(index)
    }

    pub fn get_from(&self, index_mat:&Self) -> Result<SmartValue, Box<dyn error::Error>>{

        let wrong_type_error = || Box::new(TypeMismatchError{
            found_in: self.id.clone(),
            found_type: "Number".to_string(),
            required_type: "Natural Number"
        });
        // If any of the bounds inserted are beyond this matrix's matrix, it will
        // return the Err.
        // This is used for 1-dimensional indexing
        let check_len = |a:usize, b:usize| {
            if a > self.len(){
                return Err(IndexOutOfBoundsError{ found_index: (a as isize), max_index: self.len() })
            } else if b > self.len(){
                return Err(IndexOutOfBoundsError{ found_index: (b as isize), max_index: self.len() })
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

        let num_check = |num:Decimal| num.floor() == num && num > Decimal::from(0);

        if index_mat.len() == 1{

            let first = index_mat.mat.get(0)?;

            return match first {
                SmartValue::Range(bound1, step, bound2) => {
                   if num_check(bound1) && num_check(bound2) && step == Decimal::from(1) {
                        let bound1 = bound1.to_usize().unwrap();
                        let bound2 = bound2.to_usize().unwrap();
                        let (a, b) = if bound1 < bound2 { (bound1, bound2) } else { (bound2, bound1) };
                        let _ = check_len(a, b)?;
                        Ok(SmartValue::Matrix(Self::new_from_1d_range(&self, (a - 1)..b)))
                    } else {
                        Err(wrong_type_error())
                    }
                },
                SmartValue::Number(val) => {
                    if num_check(val){
                        let index = val.to_usize().unwrap();
                        let _ = check_len(index,index)?;
                        Ok(self.mat.get(index-1)?)
                    } else {
                        Err(wrong_type_error())
                    }
                },
                _ => Err( TypeMismatchError {
                        found_in: index_mat.id.clone(),
                        found_type: FunctionController::internal_type_of(&index_mat.mat.get(0)?),
                        required_type: "Natural Number"
                    }.into())

            };
        } else if index_mat.len() == 2{
            let first = index_mat.mat.get(0)?;
            let second = index_mat.mat.get(1)?;

            return match first {
                SmartValue::Range(bound_01, step_0, bound_02) => match second {
                    SmartValue::Range(bound_11, step_1, bound_12) => {
                        if step_0 == Decimal::from(1) && step_1 == Decimal::from(1) && num_check(bound_01) && num_check(bound_02) && num_check(bound_11) && num_check(bound_12) {
                            let bound_01 = bound_01.to_usize().unwrap();
                            let bound_02 = bound_02.to_usize().unwrap();
                            let bound_11 = bound_11.to_usize().unwrap();
                            let bound_12 = bound_12.to_usize().unwrap();
                            let (a1, b1) = if bound_01 < bound_02 { (bound_01, bound_02) } else { (bound_02, bound_01) };
                            let (a2, b2) = if bound_11 < bound_12 { (bound_11, bound_12) } else { (bound_12, bound_11) };
                            let _ = check_bounds(a1, b1, a2, b2)?;
                            Ok(SmartValue::Matrix(Self::new_from_2d_range(&self, (a1 - 1)..b1, (a2 - 1)..b2)))
                        } else {
                            Err(wrong_type_error())
                        }
                    },
                    SmartValue::Number(col) => {
                        if step_0 == Decimal::from(1) && num_check(bound_01) && num_check(bound_02) && num_check(col) {
                            let bound_01 = bound_01.to_usize().unwrap();
                            let bound_02 = bound_02.to_usize().unwrap();
                            let col = col.to_usize().unwrap();
                            let (a, b) = if bound_01 < bound_02 { (bound_01, bound_02) } else { (bound_02, bound_01) };
                            let _ = check_bounds(a, col, b, col)?;
                            Ok(SmartValue::Matrix(Self::new_from_2d_range(&self, (a - 1)..b, (col - 1)..col)))
                        } else {
                            Err(wrong_type_error())
                        }
                    },
                    _ => Err(Box::new(TypeMismatchError {
                        found_in: index_mat.id.clone(),
                        found_type: FunctionController::internal_type_of(&index_mat.mat.get(1)?),
                        required_type: "Natural Number"
                    }))
                },
                SmartValue::Number(row) => match second {
                    SmartValue::Range(bound1, step, bound2) => {
                        if step == Decimal::from(1) && num_check(row) && num_check(bound1) && num_check(bound2) {
                            let row = row.to_usize().unwrap();
                            let bound1 = bound1.to_usize().unwrap();
                            let bound2 = bound2.to_usize().unwrap();
                            let (a, b) = if bound1 < bound2 { (bound1, bound2) } else { (bound2, bound1) };
                            let _ = check_bounds(row, a, row, b)?;
                            Ok(SmartValue::Matrix(Self::new_from_2d_range(&self, (row - 1)..row, (a - 1)..b)))
                        } else {
                            Err(wrong_type_error())
                        }
                    },
                    SmartValue::Number(col) => {
                        if num_check(row) && num_check(col) {
                            let row = row.to_usize().unwrap();
                            let col = col.to_usize().unwrap();
                            Ok(self.get(row - 1, col - 1).ok_or_else(|| wrong_type_error())?)
                        } else {
                            Err(wrong_type_error())
                        }
                    },
                    _ => Err(Box::new(TypeMismatchError {
                        found_in: index_mat.id.clone(),
                        found_type: FunctionController::internal_type_of(&index_mat.mat.get(1)?),
                        required_type: "Natural Number"
                    })),
                },
                _ => Err(Box::new(TypeMismatchError {
                    found_in: index_mat.id.clone(),
                    found_type: FunctionController::internal_type_of(&index_mat.mat.get(0)?),
                    required_type: "Natural Number"
                }))
            };

            // If the row index is not a Natural Number or a Range
            return Err(Box::new(TypeMismatchError{
                found_in: self.id.clone(),
                found_type: FunctionController::internal_type_of(&index_mat.mat.get(0)?),
                required_type: "Natural Number"
            }))

        }

        Err(Box::new(IncorrectNumberOfArgumentsError{
            name: "Matrix",
            found: index_mat.len(),
            requires: 1
        }))
    }

    pub fn get(&self, row:usize, col:usize) -> Option<SmartValue>{
        let base = self.col * row;
        let index = base + col;
        self.mat.get(index).ok()
    }

    pub fn len(&self) -> usize{
        return if self.mat.is_number_matrix() {
            let mat = self.mat.number_mat.as_ref().unwrap();
            mat.len()
        } else {
            let mat = self.mat.normal_mat.as_ref().unwrap();
            mat.len()
        }
    }

    /// Set data in matrix
    ///
    /// - row and col starts at index 0
    pub fn set(&mut self, row:usize, col:usize, value: SmartValue) -> Option<()>{
        let base = self.col * row;
        let index = base + col;
        self.mat.set(index, value).ok()
    }

    pub fn set_id(&mut self, id:String){
        self.id = id;
    }

    pub fn set_from(&mut self, index_mat:&Self, value:SmartValue) -> Result<(), Box<dyn error::Error>>{
        let wrong_type_error = || Box::new(TypeMismatchError{
            found_in: self.id.clone(),
            found_type: "Number".to_string(),
            required_type: "Natural Number"
        });

        let check_len = |a:usize, b:usize, len:usize| {
            if a > len{
                return Err(IndexOutOfBoundsError{ found_index: (a as isize), max_index: len })
            } else if b > len{
                return Err(IndexOutOfBoundsError{ found_index: (b as isize), max_index: len })
            }
            Ok(())
        };

        let num_check = |num:Decimal| num.floor() == num && num > Decimal::from(0);


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


        if index_mat.len() == 1 {
            let first= index_mat.mat.get(0)?;
            return match first {
                SmartValue::Range(bound1, step, bound2) => {
                    if step == Decimal::from(1) && num_check(bound1) && num_check(bound2){ // discrete
                        let bound1 = bound1.to_usize().ok_or_else(|| wrong_type_error())?;
                        let bound2 = bound2.to_usize().ok_or_else(|| wrong_type_error())?;
                        let (a, b) = if bound1 < bound2 { (bound1, bound2) } else { (bound2, bound1) };
                        let _ = check_len(a, b, self.row * self.col)?; // make sure its within the boundaries of the matrix
                        if let SmartValue::Matrix(values) = value { // we are setting this range to multiple values
                            if values.row == 1 && values.col == (b - a + 1) { // check dimensions
                                for i in 0..=(b - a) {
                                    self.mat.set(i + a - 1, unsafe { values.mat.get_unchecked(i) })?;
                                }
                            } else {
                                return Err(Box::new(DimensionMismatch {
                                    name: self.id.clone(),
                                    found: (values.row, values.col),
                                    requires: (1, b - a),
                                    extra_info: None
                                }))
                            }
                        } else { // Any other type (not a matrix)
                            for i in (a - 1)..b {
                                self.mat.set(i, value.clone())?;
                            }
                        }
                        Ok(())
                    } else {
                        Err(wrong_type_error())
                    }
                },
                SmartValue::Number(num) => {
                    if num_check(num) {
                        let index = num.to_usize().unwrap();
                        self.mat.set(index-1, value);
                        Ok(())
                    } else {
                        Err(wrong_type_error())
                    }
                },
                _ => {
                    Err(Box::new(TypeMismatchError {
                        found_in: self.id.clone(),
                        found_type: unsafe {FunctionController::internal_type_of(&index_mat.mat.get_unchecked(0))},
                        required_type: "Natural Number"
                    }))
                }
            }
        } else if index_mat.len() == 2{
            // Range,Range & Range,Natural Number
            let first = index_mat.mat.get(0)?;
            let second = index_mat.mat.get(1)?;
            return match first {
                SmartValue::Range(bound_01, step_0, bound_02) => match second {
                    SmartValue::Range(bound_11, step_1, bound_12) => {
                        if step_0 == Decimal::from(1) && step_1 == Decimal::from(1) && num_check(bound_01) && num_check(bound_02) && num_check(bound_11) && num_check(bound_12) { // discrete
                            let bound_01 = bound_01.to_usize().ok_or_else(|| wrong_type_error())?;
                            let bound_02 = bound_02.to_usize().ok_or_else(|| wrong_type_error())?;
                            let bound_11 = bound_11.to_usize().ok_or_else(|| wrong_type_error())?;
                            let bound_12 = bound_12.to_usize().ok_or_else(|| wrong_type_error())?;
                            let (a1, b1) = if bound_01 < bound_02 { (bound_01, bound_02) } else { (bound_02, bound_01) };
                            let (a2, b2) = if bound_11 < bound_12 { (bound_11, bound_12) } else { (bound_12, bound_11) };
                            let _ = check_bounds(a1, b1, a2, b2)?;
                            if let SmartValue::Matrix(values) = value {
                                if values.row == (b1 - a1 + 1) && values.col == (b2 - a2 + 1) { // check dimensions
                                    for row in (a1 - 1)..b1 {
                                        for col in (a2 - 1)..b2 {
                                            unsafe { self.set(row, col, values.get_unchecked(row - a1, col - a2)) };
                                        }
                                    }
                                } else {
                                    return Err(Box::new(DimensionMismatch {
                                        name: values.id.clone(),
                                        found: (values.row, values.col),
                                        requires: (b1 - a1 + 1, b2 - a2 + 1),
                                        extra_info: None
                                    }))
                                }
                            } else { // any other type
                                for row in (a1 - 1)..b1 {
                                    for col in (a2 - 1)..b2 {
                                        self.set(row, col, value.clone());
                                    }
                                }
                            }
                            Ok(())
                        } else {
                            Err(wrong_type_error())
                        }
                    },
                    SmartValue::Number(col) => {
                        if step_0 == Decimal::from(1) && num_check(bound_01) && num_check(bound_02) && num_check(col) {
                            let col = col.to_usize().unwrap();
                            let bound1 = bound_01.to_usize().unwrap();
                            let bound2 = bound_02.to_usize().unwrap();
                            let (a, b) = if bound1 < bound2 { (bound1, bound2) } else { (bound2, bound1) };
                            let _ = check_bounds(a, col, b, col)?;
                            if let SmartValue::Matrix(values) = value {
                                if values.row == b - a + 1 && values.col == 1 { // Dimensions of slice are equivalent to the dimensions of the matrix
                                    for row in (a - 1)..b {
                                        // the values.data[ ... ] is able to be done due to the data being a column.
                                        // the data required is contiguous in this case.
                                        unsafe { self.set(row, col - 1, values.get_unchecked(row - a, col - 1)) };
                                    }
                                } else {
                                    return Err(DimensionMismatch {
                                        name: values.id.clone(),
                                        found: (values.row, values.col),
                                        requires: (b - a + 1, 1),
                                        extra_info: None
                                    }.into())
                                }
                            } else { // Anything other than a matrix
                                for row in (a - 1)..b {
                                    unsafe { self.set(row, col, value.clone()) };
                                }
                            }
                            Ok(())
                        } else {
                            Err(wrong_type_error())
                        }
                    }

                    _ => {
                        Err(TypeMismatchError {
                            found_in: self.id.clone(),
                            found_type: FunctionController::internal_type_of(&second),
                            required_type: "Natural Number"
                        }.into())
                    }
                },
                SmartValue::Number(row) => match second {
                    SmartValue::Range(bound1, step, bound2) => {
                        if num_check(row) && num_check(bound1) && num_check(bound2) && step == Decimal::from(1) {
                            let row = row.to_usize().unwrap();
                            let bound1 = bound1.to_usize().unwrap();
                            let bound2 = bound2.to_usize().unwrap();
                            let (a, b) = if bound1 < bound2 { (bound1, bound2) } else { (bound2, bound1) };
                            let _ = check_bounds(row, a, row, b)?;
                            if let SmartValue::Matrix(values) = value {
                                if values.row == 1 && values.col == (b - a + 1) { // a nice horizontal matrix
                                    for col in (a - 1)..b {
                                        // values.data[...] can be used due to the necessary data being contiguous
                                        unsafe { self.set(row - 1, col, values.mat.get_unchecked(col - a)) };
                                    }
                                } else {
                                    return Err(Box::new(DimensionMismatch {
                                        name: values.id.clone(),
                                        found: (values.row, values.col),
                                        requires: (1, b - a + 1),
                                        extra_info: None
                                    }))
                                }
                            } else { // anything other than a matrix
                                for col in (a - 1)..b {
                                    self.set(row, col, value.clone());
                                }
                            }
                            Ok(())
                        } else {
                            Err(wrong_type_error())
                        }
                    },
                    SmartValue::Number(col) => {
                        if num_check(row) && num_check(col) {
                            let row = row.to_usize().unwrap();
                            let col = col.to_usize().unwrap();
                            self.set(row - 1, col - 1, value);
                            Ok(())
                        } else {
                            Err(wrong_type_error())
                        }
                    }
                    _ => {
                        Err(TypeMismatchError {
                            found_in: self.id.clone(),
                            found_type: FunctionController::internal_type_of(&second),
                            required_type: "Natural Number"
                        }.into())
                    }
                },
                _ => {
                    Err(TypeMismatchError {
                        found_in: self.id.clone(),
                        found_type: FunctionController::internal_type_of(&first),
                        required_type: "Natural Number"
                    }.into())
                }
            };
            }

        Err(Box::new(GenericError{})) // I don't know what to replace this with :(
    }


    /// Done for Non-Number matrices, it checks if all elements in the Matrix are a Number.
    /// - returns `true` if all elements in the matrix are a Number
    /// - returns `false` if there is an element in the matrix that is not a Number
    fn check_number(&self) -> Result<(), Box<dyn error::Error>>{
        return if self.mat.is_number_matrix(){
            Ok(())
        } else {
            Err(TypeMismatchError{
                found_in: self.id.clone(),
                found_type: "Matrix".to_string(),
                required_type: "Number Matrix"
            }.into())
        }

    }
    /// Checks to see if the matrix is a Number Matrix or a Normal Matrix (Matrix)
    pub fn is_number_matrix(&self) -> bool{
        self.mat.is_number_matrix()
    }

    pub fn add_scalar(&mut self, input:Decimal) -> Result<(), Box<dyn error::Error>>{
        let _ = self.check_number()?;
        let mat = self.mat.number_mat.as_mut().unwrap();
        mat.add_scalar_mut(input);
        Ok(())
    }

    pub fn mul_scalar(&mut self, input:Decimal) -> Result<(), Box<dyn error::Error>>{
        let _ = self.check_number()?;
        let mat= self.mat.number_mat.as_mut().unwrap();
        mat.mul_assign(input);
        Ok(())
    }

    pub fn div_scalar(&mut self, input:Decimal) -> Result<(), Box<dyn error::Error>>{
        let _ = self.check_number()?;
        let mat= self.mat.number_mat.as_mut().unwrap();
        mat.mul_assign(Decimal::from(1)/input);
        Ok(())
    }

    /// Safely performs a Matrix operation on a Number Matrix.
    ///
    /// - `cond` is the condition for executing the operation. (self.row, self.col, input.row, input.col)
    /// - `f` is the function that will execute the matrix operation.
    /// - `err` is the error that will be returned if the condition is not met.
    fn matrix_operation<C,T>(&mut self, input:&SmartMatrix, cond: C, f: T, err:Box<dyn error::Error>) -> Result<(), Box<dyn error::Error>>
    where T: Fn(&mut Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>, &Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>),
        C: Fn(usize,usize,usize,usize) -> bool
    {
        let _ = self.check_number()?;
        let _ = input.check_number()?;
        let mat= self.mat.number_mat.as_mut().unwrap();
        if cond(self.row,self.col,input.row,input.col){
            let input_mat = input.mat.number_mat.as_ref().unwrap();
            f(mat, input_mat);
        } else {
            return Err(err)
        }
        Ok(())
    }

    pub fn add(&mut self, input:&SmartMatrix) -> Result<(), Box<dyn error::Error>>{
        self.matrix_operation(input,
                              |s_row,s_col,i_row,i_col| s_row == i_row && s_col == i_col,
                              |mat,input_mat| mat.add_assign(input_mat),
                              DimensionMismatch {
                                  name: input.id.clone(),
                                  found: (input.row, input.col),
                                  requires: (self.row, self.col),
                                  extra_info: None
                              }.into())
    }

    pub fn sub(&mut self, input:&SmartMatrix) -> Result<(), Box<dyn error::Error>>{
        self.matrix_operation(input,
                              |s_row,s_col,i_row,i_col| s_row == i_row && s_col == i_col,
                              |mat,input_mat| mat.sub_assign(input_mat),
                              DimensionMismatch {
                                  name: input.id.clone(),
                                  found: (input.row, input.col),
                                  requires: (self.row, self.col),
                                  extra_info: None
                              }.into())
    }

    pub fn mul(&mut self, input:&SmartMatrix) -> Result<(), Box<dyn error::Error>>{
        self.matrix_operation(input,
                              |s_row,s_col,i_row,i_col| s_row == i_col && s_col == i_row,
                              |mat,input_mat| mat.mul_assign(input_mat),
                              DimensionMismatch {
                                  name: input.id.clone(),
                                  found: (input.row, input.col),
                                  requires: (self.col, self.row),
                                  extra_info: None
                              }.into())
    }

    pub fn identity_mat(side:usize) -> Self{
        let data = Matrix::<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>::identity(side, side);
        Self {
            id: "Matrix".to_string(),
            mat: DynMatrix { number_mat: Some(data), normal_mat: None },
            row: side,
            col: side
        }
    }

    pub fn zero_mat(row:usize,col:usize) -> Self{
        let data = Matrix::<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>::zeros(row,col);
        Self {
            id: "Matrix".to_string(),
            mat: DynMatrix { number_mat: Some(data), normal_mat: None },
            row,
            col
        }
    }

    pub fn ones_mat(row:usize,col:usize) -> Self{
        let mut data:Matrix<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>> = Matrix::<Decimal, Dynamic, Dynamic, VecStorage<Decimal, Dynamic, Dynamic>>::zeros(row,col);
        let data = data.map(|f| Decimal::from(1)).into();
        Self {
            id: "Matrix".to_string(),
            mat: DynMatrix { number_mat: Some(data), normal_mat: None },
            row,
            col
        }
    }

    /// LU Decomposition of the matrix.
    ///
    /// - Returns a SmartValue::Matrix( [ L ] [ R ] )
    /// - If matrix is NOT a Number Matrix, an Error will be returned
    pub fn lu_decomposition(&self) -> Result<SmartValue, Box<dyn error::Error>>{
        use rust_decimal::prelude::FromPrimitive;
        if self.mat.is_number_matrix(){
            let mat = self.mat.number_mat.as_ref().unwrap();
            let mat_f64 = mat.map(|f| f.to_f64().unwrap()); // temporarily converts decimal to f64
            let lu = mat_f64.lu();
            let data = lu.unpack();
            let l:DecimalMatrix = data.1.map(|f| Decimal::from_f64(f).unwrap()).into();
            let u:DecimalMatrix = data.2.map(|f| Decimal::from_f64(f).unwrap()).into();
            return Ok(SmartValue::Matrix(SmartMatrix::new_from_matrices(&[l,u])))
        }
        return Err(TypeMismatchError{
            found_in: self.id.clone(),
            found_type: "Matrix".to_string(),
            required_type: "Number Matrix"
        }.into())
    }

    /// Inverts the matrix in-place
    pub fn inverse(&mut self) -> Result<(), Box<dyn error::Error>>{
        use rust_decimal::prelude::FromPrimitive;
        if self.mat.is_number_matrix() {
            let mat = self.mat.number_mat.as_mut().unwrap();
            let mut mat:DoubleMatrix = mat.map(|f| f.to_f64().unwrap()).into();

            return if mat.try_inverse_mut() {
                self.row = mat.nrows();
                self.col = mat.ncols();
                self.mat.number_mat = Some(mat.map(|f| Decimal::from_f64(f).unwrap()));
                Ok(())
            } else {
                Err(AnyError { info: Some("Matrix is not invertible".to_string()) }.into())
            }

        }

        Err(TypeMismatchError {
            found_in: self.id.clone(),
            found_type: "Matrix".to_string(),
            required_type: "Number Matrix"
        }.into())
    }

    /// Gets the sum of a matrix
    ///
    /// - If Matrix is 2-dimensional, it will sum the Columns
    /// - If Matrix is 1-dimensional, it will sum the entire Matrix
    pub fn sum(&self) -> Result<SmartValue, Box<dyn error::Error>>{
        if self.mat.is_number_matrix(){
            return if self.row == 1 {
                let mat = self.mat.number_mat.as_ref().unwrap();
                let sum = mat.sum();
                Ok(SmartValue::Number(sum))
            } else { // sum all columns
                let mat = self.mat.number_mat.as_ref().unwrap();
                let sum_mat = mat.column_sum();
                let sum_mat = sum_mat.as_slice();
                let sum_mat = VecStorage::new(Dynamic::new(1), Dynamic::new(self.col), sum_mat.to_vec());
                let sum_mat = Matrix::from_data(sum_mat);
                Ok(SmartValue::Matrix(SmartMatrix::from_number_data(sum_mat)))
            }
        }

        Err(TypeMismatchError{
            found_in: self.id.clone(),
            found_type: "Matrix".to_string(),
            required_type: "Number Matrix"
        }.into())
    }

    /// Gets the dimensions of the Matrix
    pub fn size(&self) -> SmartValue{
        return if self.row == 1 {
            SmartValue::Number(self.col.into())
        } else {
            SmartValue::Matrix(SmartMatrix::new_from_decimals(&[self.row.into(), self.col.into()]))
        }
    }
}

impl Display for SmartMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut temp = String::new();
        for i in 0..self.row{
            for j in 0..self.col{
                temp.push_str(self.get(i,j).unwrap().get_value(false).as_str());
                temp.push('\t');
            }
            temp.push('\n');
        }
        let _ = temp.pop();
        write!(f, "{}", temp)
    }
}