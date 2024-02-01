//! Implementation of the specific game rules.
//! 
//! This module provides a struct `Code`, which represents a 3-digit solution
//! code, as well as the criteria cards in the form of functions that take Codes
//! and return the fitting critera.
//! 
//! Criteria cards that can have multiple rulesets are not yet implemented.

/// A three-digit code
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Code {
    pub blue: u8,
    pub yellow: u8,
    pub purple: u8,
}

impl ToString for Code {

    /// A nice representation of the code to print to the console.
    fn to_string(&self) -> String {
        format!("\x1b[34m{}\x1b[33m{}\x1b[35m{}\x1b[0m", self.blue, self.yellow, self.purple)
    }
}

/// RULESET 1: Compara a single value of the code to a fixed target value.
/// 
/// Returns 0 if the value is smaller than the target, 1 if the value is
/// equal to the target and 2 if the value is greater than the target.
fn compare_values(value: u8, target: u8) -> Option<u8> {
    match value.cmp(&target) {
        std::cmp::Ordering::Less => Some(0),
        std::cmp::Ordering::Equal => Some(1),
        std::cmp::Ordering::Greater => Some(2),
    }
}

/// compare blue to 1
fn rule_1(input: &Code) -> Option<u8> { compare_values(input.blue, 1) }
/// compare blue to 3
fn rule_2(input: &Code) -> Option<u8> { compare_values(input.blue, 3) }
/// compare yellow to 3
fn rule_3(input: &Code) -> Option<u8> { compare_values(input.yellow, 3) }
/// compare yellow to 4
fn rule_4(input: &Code) -> Option<u8> { compare_values(input.yellow, 4) }

/// RULESET 2: Check a single value's parity.
/// 
/// Returns 0 if the value is even and 1 if the value is odd.
fn single_parity(value: u8) -> Option<u8> {
    match value % 2 == 0 {
        true => Some(0),
        false => Some(1),
    }
}

/// check blue's parity
fn rule_5(input: &Code) -> Option<u8> { single_parity(input.blue) }
/// check yellow's parity
fn rule_6(input: &Code) -> Option<u8> { single_parity(input.yellow) }
/// check purple's parity
fn rule_7(input: &Code) -> Option<u8> { single_parity(input.purple) }

/// RULESET 3: Check how often a digit appears within the code.
/// 
/// Returns the number of times the digit appears.
fn count_digit(code: &Code, digit: u8) -> Option<u8> {
    Some(
        (code.blue == digit) as u8
        + (code.yellow == digit) as u8
        + (code.purple == digit) as u8
    )
}

/// how many 1s?
fn rule_8(input: &Code) -> Option<u8> { count_digit(input, 1) }
/// how many 3s?
fn rule_9(input: &Code) -> Option<u8> { count_digit(input, 3) }
/// how many 4s?
fn rule_10(input: &Code) -> Option<u8> { count_digit(input, 4) }

// RULESET 4: Compare two values. Just a rehash of RULESET 1
/// compare blue to yellow
fn rule_11(input: &Code) -> Option<u8> { compare_values(input.blue, input.yellow) }
/// compare blue to purple
fn rule_12(input: &Code) -> Option<u8> { compare_values(input.blue, input.purple) }
/// compare yellow to purple
fn rule_13(input: &Code) -> Option<u8> { compare_values(input.yellow, input.purple) }

/// Look for the smallest value
/// 
/// 0 -> blue, 1 -> yellow, 2 -> purple, None if there's no single smallest value
fn rule_14(input: &Code) -> Option<u8> {
    if input.blue < input.yellow && input.blue < input.purple {
        return Some(0);
    }
    if input.yellow < input.blue && input.yellow < input.purple {
        return Some(1);
    }
    if input.purple < input.yellow && input.purple < input.blue {
        return Some(2);
    }
    None
}

/// Look for the greatest value
/// 
/// 0 => blue, 1 => yellow, 2 => purple, None if there's no single greatest value
fn rule_15(input: &Code) -> Option<u8> {
    if input.blue > input.yellow && input.blue > input.purple {
        return Some(0);
    }
    if input.yellow > input.blue && input.yellow > input.purple {
        return Some(1);
    }
    if input.purple > input.yellow && input.purple > input.blue {
        return Some(2);
    }
    None
}

/// are there more odd or even digits?
/// 
/// even => 0, odd => 1
fn rule_16(input: &Code) -> Option<u8> {
    let odd = input.blue % 2 + input.yellow % 2 + input.purple % 2;
    match odd >= 2 {
        true => Some(1),
        false => Some(0),
    }
}

/// count the number of even digits.
fn rule_17(input: &Code) -> Option<u8> {
    let odd = input.blue % 2 + input.yellow % 2 + input.purple % 2;
    let even = 3 - odd;
    Some(even)
}

/// is the digit sum odd or even?
/// 
/// even => 0, odd => 1
fn rule_18(input: &Code) -> Option<u8> {
    match (input.blue + input.yellow + input.purple) % 2 == 0 {
        true => Some(0),
        false => Some(1),
    }
}

// RULE 19 is a rehash of RULESET 1
/// compare blue + yellow to 6
fn rule_19(input: &Code) -> Option<u8> { compare_values(input.blue + input.yellow, 6) }

/// how many times does the most common digits appear?
/// 
/// returns the amount - 1.
fn rule_20(input: &Code) -> Option<u8> {
    let no_pairs = (input.blue == input.yellow) as usize
        + (input.blue == input.purple) as usize
        + (input.yellow == input.purple) as usize;
    match no_pairs {
        0 => Some(2),
        1 => Some(1),
        _ => Some(0),
    }
}

// RULE 21 is a variation of RULE 20
/// is there a single pair of same digits?
/// 
/// returns 1 if yes, 0 otherwise.
fn rule_21(input: &Code) -> Option<u8> {
    Some(rule_20(input).unwrap() % 2)
}

/// are the digits ordered?
/// 
/// 0 => ascending order, 1 => descending order, 2 => no order.
fn rule_22(input: &Code) -> Option<u8> {
    if input.blue < input.yellow && input.yellow < input.purple {
        return Some(0);
    }
    if input.blue > input.yellow && input.yellow > input.purple {
        return Some(1);
    }
    return Some(2);
}

// another rehash of RULESET 1
/// compare the digit sum to 6.
fn rule_23(input: &Code) -> Option<u8> { compare_values(input.blue + input.yellow + input.purple, 6) }

// RULES 24 and 25 are sorta similar, but I don't think there's a lot of
// abstraction possible 
/// how many ascending digits in order are there?
fn rule_24(input: &Code) -> Option<u8> {
    Some((input.blue + 1 == input.yellow) as u8
    + (input.yellow + 1 == input.purple) as u8)
}

/// how many digits that are either ascending or descending in order are there?
fn rule_25(input: &Code) -> Option<u8> {
    let r = rule_24(input).unwrap();
    if r > 0 {
        return Some(r);
    } 
    Some((input.blue == input.yellow + 1) as u8
    + (input.yellow == input.purple + 1) as u8)
}

/// The array containing all simple rules (1 - 25).
pub const RULES: [fn(input: &Code) -> Option<u8>; 25] = [
    rule_1, rule_2, rule_3, rule_4, rule_5, rule_6, rule_7, rule_8, rule_9,
    rule_10, rule_11, rule_12, rule_13, rule_14, rule_15, rule_16, rule_17,
    rule_18, rule_19, rule_20, rule_21, rule_22, rule_23, rule_24, rule_25,
];