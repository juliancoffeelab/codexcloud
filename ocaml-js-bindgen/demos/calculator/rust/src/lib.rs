use ocamljs_bindgen::ocamljs_bindgen;

/// A calculator key. The binding passes this enum as a scalar.
#[ocamljsify::ocamljsify]
#[derive(Clone, Copy)]
pub enum Key {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Decimal,
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    Clear,
    Backspace,
    Negate,
    Percent,
}

#[derive(Clone, Copy)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Copy)]
enum Fault {
    DivisionByZero,
    Overflow,
}

/// Stateful, immediate-execution calculator. Its contents stay in Rust.
#[ocamljs_bindgen(opaque)]
pub struct Calculator {
    entry: String,
    accumulator: Option<f64>,
    operation: Option<Operation>,
    repeat: Option<(Operation, f64)>,
    fresh: bool,
    waiting: bool,
    error: Option<Fault>,
}

#[ocamljs_bindgen]
impl Calculator {
    /// Start a calculator displaying zero.
    pub fn create() -> Calculator {
        Calculator {
            entry: "0".into(),
            accumulator: None,
            operation: None,
            repeat: None,
            fresh: true,
            waiting: false,
            error: None,
        }
    }

    /// Read the display. Errors remain visible until clear or a new number.
    pub fn display(&self) -> String {
        match self.error {
            Some(Fault::DivisionByZero) => "Cannot divide by zero".into(),
            Some(Fault::Overflow) => "Overflow".into(),
            None => self.entry.clone(),
        }
    }

    /// Apply one key. Operations execute from left to right, like a desk calculator.
    pub fn press(&mut self, key: Key) {
        use Key::*;
        if matches!(key, Clear) {
            *self = Self::create();
            return;
        }
        let digit = match key {
            Zero => Some('0'),
            One => Some('1'),
            Two => Some('2'),
            Three => Some('3'),
            Four => Some('4'),
            Five => Some('5'),
            Six => Some('6'),
            Seven => Some('7'),
            Eight => Some('8'),
            Nine => Some('9'),
            _ => None,
        };
        if self.error.is_some() {
            if digit.is_some() || matches!(key, Decimal) {
                *self = Self::create();
            } else {
                return;
            }
        }
        if let Some(digit) = digit {
            self.waiting = false;
            if self.fresh {
                self.entry = "0".into();
                self.fresh = false;
                self.repeat = None;
            }
            if self.entry == "0" {
                self.entry.clear();
            } else if self.entry == "-0" {
                self.entry = "-".into();
            }
            if self.entry.len() < 16 {
                self.entry.push(digit);
            }
            return;
        }
        match key {
            Decimal => {
                self.waiting = false;
                if self.fresh {
                    self.entry = "0".into();
                    self.fresh = false;
                    self.repeat = None;
                }
                if !self.entry.contains('.') {
                    self.entry.push('.');
                }
            }
            Backspace if !self.fresh => {
                self.entry.pop();
                if self.entry.is_empty() || self.entry == "-" {
                    self.entry = "0".into();
                }
            }
            Negate => {
                if self.waiting || (self.fresh && self.entry == "0" && self.repeat.is_none()) {
                    self.entry = "0".into();
                    self.fresh = false;
                }
                self.waiting = false;
                self.repeat = None;
                if self.entry.starts_with('-') {
                    self.entry.remove(0);
                } else {
                    self.entry.insert(0, '-');
                }
            }
            Percent => {
                self.show(self.number() / 100.0);
                self.fresh = true;
                self.waiting = false;
                self.repeat = None;
            }
            Add | Subtract | Multiply | Divide => {
                let value = self.number();
                if !self.waiting
                    && let (Some(left), Some(operation)) = (self.accumulator, self.operation)
                {
                    self.evaluate(left, operation, value);
                    if self.error.is_some() {
                        return;
                    }
                }
                self.accumulator = Some(self.number());
                self.operation = Some(match key {
                    Add => Operation::Add,
                    Subtract => Operation::Subtract,
                    Multiply => Operation::Multiply,
                    _ => Operation::Divide,
                });
                self.repeat = None;
                self.fresh = true;
                self.waiting = true;
            }
            Equals => {
                if let (Some(left), Some(operation)) = (self.accumulator, self.operation) {
                    let right = self.number();
                    self.evaluate(left, operation, right);
                    if self.error.is_none() {
                        self.repeat = Some((operation, right));
                    }
                } else if let Some((operation, right)) = self.repeat {
                    self.evaluate(self.number(), operation, right);
                }
                self.operation = None;
                self.accumulator = None;
                self.fresh = true;
                self.waiting = false;
            }
            _ => {}
        }
    }
}

impl Calculator {
    fn number(&self) -> f64 {
        self.entry.parse().unwrap_or(0.0)
    }

    fn show(&mut self, value: f64) {
        if !value.is_finite() {
            self.error = Some(Fault::Overflow);
        } else {
            // Round binary floating-point noise to a useful calculator precision.
            self.entry = if value != 0.0 && !(1e-9..1e12).contains(&value.abs()) {
                format!("{value:.11e}")
            } else {
                let decimals = if value == 0.0 {
                    0
                } else {
                    (11.0 - value.abs().log10().floor()).max(0.0) as usize
                };
                let text = format!("{value:.decimals$}");
                if text.contains('.') {
                    text.trim_end_matches('0').trim_end_matches('.').into()
                } else {
                    text
                }
            };
            if self.entry == "-0" {
                self.entry = "0".into();
            }
        }
    }

    fn evaluate(&mut self, left: f64, operation: Operation, right: f64) {
        if matches!(operation, Operation::Divide) && right == 0.0 {
            self.error = Some(Fault::DivisionByZero);
            self.operation = None;
            self.accumulator = None;
            self.repeat = None;
            return;
        }
        self.show(match operation {
            Operation::Add => left + right,
            Operation::Subtract => left - right,
            Operation::Multiply => left * right,
            Operation::Divide => left / right,
        });
    }
}

#[cfg(test)]
#[path = "../../tests/rust/state.rs"]
mod state_tests;

#[cfg(feature = "conformance")]
include!("../../tests/rust/conformance.rs");
