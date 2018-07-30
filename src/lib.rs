#![warn(missing_docs)]

//! # Bitcoin Amount
//!

#[cfg(feature = "serde")]
extern crate serde;
#[cfg(feature = "serde_json_number")]
extern crate serde_json;

use std::error;
use std::fmt::{self, Display, Formatter};

use std::ops::{Add, Div, Mul, Sub};

use std::num::ParseFloatError;
use std::str::FromStr;

/// The primitive type that holds the satoshis.
type Inner = i64;

/// The amount of satoshis in a BTC.
pub const SAT_PER_BTC: i64 = 100_000_000;

/// The amount of satoshis in a BTC.
pub const SAT_PER_BTC_FP: f64 = 100_000_000.0;

/// Maximum value in an `Amount`.
pub const MAX: Amount = Amount(Inner::max_value());
/// Minimum value in an `Amount`.
pub const MIN: Amount = Amount(Inner::min_value());

/// A bitcoin amount integer type.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Amount(Inner);

impl Amount {
    /// Creates a new `Amount` from a satoshi amount.
    ///
    /// # Panics
    ///
    /// The satoshi amount can't be larger than [max_value][1].
    ///
    /// [1]: #method.max_value
    pub fn from_btc(btc: f64) -> Amount {
        let sat = round_and_to_sat(btc);
        Amount::from_sat(sat)
    }

    /// Creates a new `Amount` from a satoshi amount.
    pub fn from_sat(sat: Inner) -> Amount {
        Amount(sat)
    }

    /// Creates an `Amount` from a JSON number, the JSON number unit
    /// SHOULD be in BTC not satoshis.
    #[cfg(feature = "serde_json_number")]
    pub fn from_json_number(num: &serde_json::value::Number) -> Amount {
        let num = format!("{}", num);
        Amount::from_str(&*num).unwrap()
    }

    /// Returns the additive identity of `Amount`.
    pub fn zero() -> Amount {
        Amount(0)
    }

    /// Returns the multiplicative identity of `Amount`.
    pub fn one() -> Amount {
        Amount(1)
    }

    /// Maximum value that can fit in an `Amount`.
    pub fn max_value() -> Amount { MAX }

    /// Minimum value that can fit in an `Amount`.
    pub fn min_value() -> Amount { MIN }

    /// Converts this `Amount` to the inner satoshis.
    pub fn into_inner(self) -> Inner {
        self.0
    }
}

impl Add for Amount {
    type Output = Amount;
    
    fn add(self, rhs: Amount) -> Self::Output {
        Amount::from_sat(self.0 + rhs.0)
    }
}

impl Div for Amount {
    type Output = Amount;
    
    fn div(self, rhs: Amount) -> Self::Output {
        Amount::from_sat(self.0 / rhs.0)
    }
}

impl Mul for Amount {
    type Output = Amount;
    
    fn mul(self, rhs: Amount) -> Self::Output {
        Amount::from_sat(self.0 * rhs.0)
    }
}

impl Sub for Amount {
    type Output = Amount;
    
    fn sub(self, rhs: Amount) -> Self::Output {
        Amount::from_sat(self.0 - rhs.0)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>
    {
        Inner::deserialize(deserializer).map(Amount)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer
    {
        Inner::serialize(&self.0, serializer)
    }
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let btc = f64::from_str(s).map_err(ParseAmountError)?;

        Ok(Amount::from_btc(btc))
    }
}

/// An error during `Amount` parsing.
#[derive(Debug)]
pub struct ParseAmountError(ParseFloatError);

impl Display for ParseAmountError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "invalid floating point integer: {}", self.0)
    }
}

impl error::Error for ParseAmountError {
    fn cause(&self) -> Option<&error::Error> {
        Some(&self.0)
    }

    fn description(&self) -> &'static str {
        "floating point error"
    }
}

fn round_and_to_sat(v: f64) -> Inner {
    if v < 0.0 {
        ((v * SAT_PER_BTC_FP) - 0.5) as Inner
    } else {
        ((v * SAT_PER_BTC_FP) + 0.5) as Inner
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn amount_from_btc() {
        assert_eq!(Amount::from_btc(0.00253583).0, 253583);
    }

    #[test]
    fn amount_from_sat() {
        assert_eq!(Amount::from_sat(253583).0, 253583);
    }

    #[test]
    fn amount_from_str() {
        let amt = Amount::from_str("0.00253583").unwrap();
        assert_eq!(amt, Amount::from_sat(253583));
        let amt = Amount::from_str("0.10000000").unwrap();
        assert_eq!(amt, Amount::from_sat(10_000_000));
    }

    #[test]
    fn amount_add_div_mul_sub() {
        let res = ((Amount::from_btc(0.0025) +
                    Amount::from_btc(0.0005)) * (Amount::from_btc(2.0))) /
                    Amount::from_btc(2.0);

        assert_eq!(res, Amount::from_btc(0.003));
    }
}
