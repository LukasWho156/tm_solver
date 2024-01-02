#[derive(Debug, Clone)]
pub struct Code {
    pub blue: u8,
    pub yellow: u8,
    pub purple: u8,
}

impl ToString for Code {
    fn to_string(&self) -> String {
        format!("\x1b[34m{}\x1b[33m{}\x1b[35m{}\x1b[0m", self.blue, self.yellow, self.purple)
    }
}

// RULESET 1: Compara a single value of the code to a fixed target value
fn compare_values(value: u8, target: u8) -> Option<u8> {
    match value.cmp(&target) {
        std::cmp::Ordering::Less => Some(0),
        std::cmp::Ordering::Equal => Some(1),
        std::cmp::Ordering::Greater => Some(2),
    }
}

fn rule_1(input: &Code) -> Option<u8> { compare_values(input.blue, 1) }
fn rule_2(input: &Code) -> Option<u8> { compare_values(input.blue, 3) }
fn rule_3(input: &Code) -> Option<u8> { compare_values(input.yellow, 3) }
fn rule_4(input: &Code) -> Option<u8> { compare_values(input.yellow, 4) }

// RULESET 2: Check a single value's parity
fn single_parity(value: u8) -> Option<u8> {
    match value % 2 == 0 {
        true => Some(0),
        false => Some(1),
    }
}

fn rule_5(input: &Code) -> Option<u8> { single_parity(input.blue) }
fn rule_6(input: &Code) -> Option<u8> { single_parity(input.yellow) }
fn rule_7(input: &Code) -> Option<u8> { single_parity(input.purple) }

// RULESET 3: Check how often a digit appears within the code
fn count_digit(code: &Code, digit: u8) -> Option<u8> {
    Some(
        (code.blue == digit) as u8
        + (code.yellow == digit) as u8
        + (code.purple == digit) as u8
    )
}

fn rule_8(input: &Code) -> Option<u8> { count_digit(input, 1) }
fn rule_9(input: &Code) -> Option<u8> { count_digit(input, 3) }
fn rule_10(input: &Code) -> Option<u8> { count_digit(input, 4) }

// RULESET 4: Compare two values. Just a rehash of RULESET 1
fn rule_11(input: &Code) -> Option<u8> { compare_values(input.blue, input.yellow) }
fn rule_12(input: &Code) -> Option<u8> { compare_values(input.blue, input.purple) }
fn rule_13(input: &Code) -> Option<u8> { compare_values(input.yellow, input.purple) }

// RULES 14 and 15 are pretty unique
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

// so are RULES 16 - 18
fn rule_16(input: &Code) -> Option<u8> {
    let odd = input.blue % 2 + input.yellow % 2 + input.purple % 2;
    match odd >= 2 {
        true => Some(1),
        false => Some(0),
    }
}

fn rule_17(input: &Code) -> Option<u8> {
    let odd = input.blue % 2 + input.yellow % 2 + input.purple % 2;
    let even = 3 - odd;
    Some(even)
}

fn rule_18(input: &Code) -> Option<u8> {
    match (input.blue + input.yellow + input.purple) % 2 == 0 {
        true => Some(0),
        false => Some(1),
    }
}

// RULE 19 is a rehash of RULESET 1
fn rule_19(input: &Code) -> Option<u8> { compare_values(input.blue + input.yellow, 6) }

// RULE 21 is a variation of RULE 20
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

fn rule_21(input: &Code) -> Option<u8> {
    Some(rule_20(input).unwrap() % 2)
}

pub const RULES: [fn(input: &Code) -> Option<u8>; 21] = [
    rule_1, rule_2, rule_3, rule_4, rule_5, rule_6, rule_7, rule_8, rule_9,
    rule_10, rule_11, rule_12, rule_13, rule_14, rule_15, rule_16, rule_17,
    rule_18, rule_19, rule_20, rule_21,
];