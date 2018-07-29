#![warn(missing_docs)]

//! # Bitcoin Amount
//!

use std::error;
use std::fmt::{self, Display, Formatter};
use std::ops::Add;
use std::str::FromStr;

/// The amount of satoshis in a BTC.
pub const SAT_PER_BTC: u64 = 100_000_000;

/// Maximum value in an `Amount`.
pub const MAX: Amount = Amount(21_000_000 * SAT_PER_BTC);
/// Minimum value in an `Amount`.
pub const MIN: Amount = Amount(0);

/// A bitcoin amount integer type.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Amount(u64);

impl Amount {
    /// Creates a new `Amount` from a satoshi amount.
    ///
    /// # Panics
    ///
    /// The satoshi amount can't be larger than [max_value][1].
    ///
    /// [1]: #method.max_value
    pub fn from_sat(sat: u64) -> Amount {
        assert!(sat <= Amount::max_value().0,
                "satoshis amount is larger than `Amount::max_value()`");

        Amount(sat)
    }

    /// Maximum value that can fit in an `Amount`.
    ///
    /// This is defined to be 21,000,000 BTC.
    pub fn max_value() -> Amount { MAX }

    /// Minimum value that can fit in an `Amount` (0 BTC).
    pub fn min_value() -> Amount { MIN }
}

impl Add for Amount {
    type Output = Amount;
    
    fn add(self, rhs: Amount) -> Self::Output {
        // TODO: overflow?
        Amount::from_sat(self.0 + rhs.0)
    }
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();

        // Make sure the amount is positive,
        let s = match s[0] {
            b'+' => &s[1..],
            b'-' => return Err(ParseAmountError::NegativeAmount),
            _ => s,
        };

        let decimal = parse_decimal(s)?;

        let integral_sat = Amount::from_sat(decimal.integral * SAT_PER_BTC);
        if integral_sat > Amount::max_value() {
            return Err(ParseAmountError::TooLarge);
        }

        if let Some((fractional, zeroes)) = decimal.fractional {
            let fractional = fractional * 10.pow(zeroes);
            let fractional_sat = Amount::from_sat(fractional * SAT_PER_BTC);
            if integral_sat + fractional_sat > Amount::max_value() {
                return Err(ParseAmountError::TooLarge);
            }

            return Ok(integral_sat + fractional_sat);
        }
        
        Ok(integral_sat)
    }
}

struct Decimal {
    integral: u64,
    fractional: Option<(u64, u64)>,
}

/// Check if the input string is a valid floating point number and if so,
/// locate the integral part, the fractional part, and the exponent in it. Does
/// not handle signs.
fn parse_decimal(s: &[u8]) -> Result<Decimal, ParseAmountError> {
    use std::str::from_utf8_unchecked;

    if s.is_empty() {
        return Err(ParseAmountError::InvalidAmount);
    }

    let (integral, s) = eat_digits(s);

    if integral.is_empty() {
        // We require at least a single digit before the point.
        return Err(ParseAmountError::InvalidAmount);
    }
    // Totally safe, integral is valid in this point.
    let integral = unsafe {
        from_utf8_unchecked(integral).parse::<u64>().unwrap()
    };

    match s.first() {
        None => {
            return Ok(Decimal {
                integral,
                fractional: None,
            });
        }
        Some(&b'.') => {
            let (fractional, s) = eat_digits(&s[1..]);
            if fractional.is_empty() {
                // We require after the point.
                return Err(ParseAmountError::InvalidAmount);
            }
            let zeroes = eat_zeroes(s) as u64;

            // Totally safe, fractional is valid in this point.
            let fractional = unsafe {
                from_utf8_unchecked(fractional).parse::<u64>().unwrap()
            };

            match s.first() {
                None => {
                    return Ok(Decimal {
                        integral,
                        fractional: Some((fractional, zeroes)),
                    });
                },
                // Trailing junk after fractional part
                _ => return Err(ParseAmountError::InvalidAmount),
            }
        }
        // Trailing junk after first digit string
        _ => return Err(ParseAmountError::InvalidAmount),
    }
}

/// Carve off decimal digits up to the first non-digit character.
fn eat_digits(s: &[u8]) -> (&[u8], &[u8]) {
    let mut i = 0;
    while i < s.len() && b'0' <= s[i] && s[i] <= b'9' {
        i += 1;
    }
    (&s[..i], &s[i..])
}

fn eat_zeroes(s: &[u8]) -> usize {
    let mut i = 0;
    while i < s.len() && s[i] == b'0' {
        i += 1;
    }

    (i, &s[i..])
}

/// An error during `Amount` parsing.
#[derive(Debug)]
pub enum ParseAmountError {
    /// The amount is negative.
    NegativeAmount,
    /// Invalid amount.
    InvalidAmount,
    /// The amount is larger than `Amount::max_value`.
    TooLarge,
}

impl Display for ParseAmountError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match *self {
            ParseAmountError::NegativeAmount => write!(fmt, "negative amount"),
            ParseAmountError::InvalidAmount => write!(fmt, "invalid amount"),
            ParseAmountError::TooLarge => write!(fmt, "amount is too large"),
        }
    }
}

impl error::Error for ParseAmountError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }

    fn description(&self) -> &'static str {
        match *self {
            ParseAmountError::NegativeAmount => "negative amount",
            ParseAmountError::InvalidAmount => "invalid amount",
            ParseAmountError::TooLarge => "amount is too large",
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn amount_from_sat() {
        assert_eq!(Amount::from_sat(253583).0, 253583);
    }

    #[test]
    #[should_panic]
    fn invalid_amount_from_sat() {
        Amount::from_sat((21_000_000 * SAT_PER_BTC) + 1);
    }

    #[test]
    fn amount_from_str() {
        let amt = Amount::from_str("0.00253583").unwrap();
        assert_eq!(amt, Amount::from_sat(253583));
        let amt = Amount::from_str("0.10000000").unwrap();
        assert_eq!(amt, Amount::from_sat(10_000_000));
    }
}
