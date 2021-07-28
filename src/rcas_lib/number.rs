use std::ops::{Deref, DerefMut, Neg, RemAssign, Rem};
use rust_decimal::Decimal;
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign};
use rust_decimal::prelude::ToPrimitive;
use nalgebra::{ComplexField, Field, Complex};
use std::fmt::{self, Debug, Formatter, Display};
use simba::scalar::ClosedNeg;
use num_traits::Num;
use simba::simd::SimdValue;

/// A wrapper over the rust-decimal.
///
/// - Adds support for imaginary numbers!
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Number{
    real: Decimal,
    im: Decimal,
}

impl Number{
    pub fn from_imaginary(real: Decimal, imaginary: Decimal) -> Self{
        Self {
            real,
            im: imaginary
        }
    }

    pub fn from_real(real: Decimal) -> Self{
        Self {
            real,
            im: Decimal::from(0),
        }
    }

    pub fn is_real(&self) -> bool{
        if self.im == Decimal::from(0){
            return true
        } else {
            false
        }
    }

    /// Returns the real part of a complex number
    pub fn re(&self) -> Decimal{
        self.real
    }

    pub fn im(&self) -> Decimal{
        self.im
    }

}
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Phasor{
    magnitude: Decimal,
    angle: Decimal,
}

impl Phasor{
    pub fn deconstruct(self) -> (Decimal, Decimal){
        (self.magnitude, self.angle)
    }

    pub fn magnitude(&self) -> Decimal{
        self.magnitude
    }

    pub fn angle(&self) -> Decimal{
        self.angle
    }

    pub fn from_number(number:Number) -> Self{
        use rust_decimal::prelude::FromPrimitive;
        // a^ + b^2 = c^2
        let magnitude = (number.real * number.real) * (number.im * number.im);
        // θ = atan(opposite/hypotenuse)
        let angle = Decimal::from_f64((number.im/number.real).to_f64().unwrap().atan()).unwrap();
        Self {magnitude, angle}
    }

    pub fn from_decimal(number:Decimal) -> Self {
        Self {
            magnitude: number,
            angle: Decimal::from(0)
        }
    }
}



impl Mul for Phasor{
    type Output = Phasor;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            magnitude: self.magnitude * rhs.magnitude,
            angle: self.angle + rhs.angle
        }
    }
}

impl MulAssign for Phasor{
    fn mul_assign(&mut self, rhs: Self) {
        self.magnitude *= rhs.magnitude;
        self.angle += rhs.angle;
    }
}

impl Div for Phasor{
    type Output = Phasor;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            magnitude: self.magnitude/rhs.magnitude,
            angle: self.angle - rhs.angle,
        }
    }
}

impl DivAssign for Phasor{
    fn div_assign(&mut self, rhs: Self) {
        self.magnitude /= rhs.magnitude;
        self.angle -= rhs.angle;
    }
}

impl From<Phasor> for Number{
    fn from(phasor: Phasor) -> Self {
        use rust_decimal::prelude::FromPrimitive;
        // sin(θ)*hypotenuse = opposite (imaginary)
        // cos(θ)*hypotenuse = adjacent (real)
        let real = Decimal::from_f64((phasor.angle.to_f64().unwrap()).sin()).unwrap() * phasor.magnitude;
        let im = Decimal::from_f64((phasor.angle.to_f64().unwrap()).cos()).unwrap() * phasor.magnitude;
        Self {
            real,
            im
        }
    }
}

impl Add for Number{
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real + rhs.real,
            im: self.im + rhs.im,
        }
    }
}

impl Add<Decimal> for Number{
    type Output = Number;

    fn add(self, rhs: Decimal) -> Self::Output {
        Self {
            real: self.real + rhs,
            im: self.im
        }
    }
}

impl AddAssign for Number{
    fn add_assign(&mut self, rhs: Self) {
        self.real += rhs.real;
        self.im += rhs.im;
    }
}

impl AddAssign<Decimal> for Number{
    fn add_assign(&mut self, rhs: Decimal) {
        self.real += rhs;
    }
}

impl Sub for Number{
    type Output = Number;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real - rhs.real,
            im: self.im - rhs.im,
        }
    }
}

impl Sub<Decimal> for Number{
    type Output = Number;

    fn sub(self, rhs: Decimal) -> Self::Output {
        Self {
            real: self.real - rhs,
            im: self.im
        }
    }
}

impl SubAssign for Number{
    fn sub_assign(&mut self, rhs: Self) {
        self.real -= rhs.real;
        self.im -= rhs.im;
    }
}

impl SubAssign<Decimal> for Number{
    fn sub_assign(&mut self, rhs: Decimal) {
        self.real -= rhs;
    }
}

impl Mul for Number{
    type Output = Number;

    fn mul(self, rhs: Self) -> Self::Output {
        (Phasor::from_number(self) * Phasor::from_number(rhs)).into()
    }
}

impl Mul<Decimal> for Number{
    type Output = Number;

    fn mul(self, rhs: Decimal) -> Self::Output {
        (Phasor::from_number(self) * Phasor::from_decimal(rhs)).into()
    }
}

impl MulAssign for Number{
    fn mul_assign(&mut self, rhs: Self) {
        let result:Number = (Phasor::from_number(*self) * Phasor::from_number(rhs)).into();
        self.real = result.real;
        self.im = result.im;
    }
}

impl MulAssign<Decimal> for Number{
    fn mul_assign(&mut self, rhs: Decimal) {
        let result:Number = (Phasor::from_number(*self) * Phasor::from_decimal(rhs)).into();
        self.real = result.real;
        self.im = result.im;
    }
}

impl Div for Number{
    type Output = Number;

    fn div(self, rhs: Self) -> Self::Output {
        (Phasor::from_number(self) / Phasor::from_number(rhs)).into()
    }
}

impl Div<Decimal> for Number{
    type Output = Number;

    fn div(self, rhs: Decimal) -> Self::Output {
        (Phasor::from_number(self) / Phasor::from_decimal(rhs)).into()
    }
}

impl DivAssign for Number{
    fn div_assign(&mut self, rhs: Self) {
        let result:Number = (Phasor::from_number(*self) / Phasor::from_number(rhs)).into();
        self.real = result.real;
        self.im = result.im;
    }
}

impl DivAssign<Decimal> for Number{
    fn div_assign(&mut self, rhs: Decimal) {
        let result:Number = (Phasor::from_number(*self) / Phasor::from_decimal(rhs)).into();
        self.real = result.real;
        self.im = result.im;
    }
}

/// Easily Implements types for conversion to Number
macro_rules! impl_from{
    ($($T:ty),*) => {
    $(
        impl From<$T> for Number {
            fn from(num:$T) -> Self{
                Self {
                    real: Decimal::from(num),
                    im: Decimal::from(0),
                }
            }
        }
    )*

    }
}

impl_from!(usize,isize,i64,i32,u32,i16,u16,i8,u8);


impl Debug for Number{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Number")
            .field("real", &self.real)
            .field("im", &self.im)
            .finish()
    }
}

impl Display for Number{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        return if self.real == Decimal::from(0) {
            write!(f, "{}", self.real)
        } else {
            if self.im.is_sign_positive() {
                write!(f, "{} + j{}", self.real, self.im)
            } else {
                write!(f, "{} - j{}", self.real, self.im.abs())
            }
        }
    }
}

impl From<Decimal> for Number{
    fn from(num: Decimal) -> Self {
        Self {
            real: num,
            im: Decimal::from(0)
        }
    }
}

impl Neg for Number{
    type Output = Number;

    fn neg(mut self) -> Self::Output {
        self.real = -self.real;
        self.im = -self.im;
        self
    }
}

impl Rem for Number{
    type Output = Number;

    fn rem(self, rhs: Self) -> Self::Output {
        Self{
            real: self.real % rhs.real,
            im: self.im % rhs.im
        }
    }
}

impl RemAssign for Number{
    fn rem_assign(&mut self, rhs: Self) {
        self.real.rem_assign(rhs.real);
        self.im.rem_assign(rhs.im);
    }
}

impl num_traits::One for Number{
    fn one() -> Self {
        Self {
            real: Decimal::from(1),
            im: Decimal::from(0),
        }
    }

    fn set_one(&mut self) {
        self.real = Decimal::from(1);
        self.im = Decimal::from(0);
    }

    fn is_one(&self) -> bool where
        Self: PartialEq, {
        self == &Self::one()
    }
}

impl num_traits::Zero for Number{
    fn zero() -> Self {
        Self {
            real: Decimal::from(0),
            im: Decimal::from(0)
        }
    }

    fn set_zero(&mut self) {
        self.real = Decimal::from(0);
        self.im = Decimal::from(0);
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl num_traits::Num for Number{
    type FromStrRadixErr = rust_decimal::Error;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        let real = Decimal::from_str_radix(str, radix)?;
        let im = Decimal::from_str_radix(str, radix)?;
        Ok(Self {
            real,
            im
        })
    }
}


