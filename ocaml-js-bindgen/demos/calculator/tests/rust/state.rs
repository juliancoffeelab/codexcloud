use super::Key;
use super::{Calculator, Key::*};

fn enter(calculator: &mut Calculator, keys: &[Key]) {
    for &key in keys {
        calculator.press(key);
    }
}

#[test]
fn chaining_operator_replacement_and_repeat() {
    let mut c = Calculator::create();
    enter(&mut c, &[One, Two, Add, Three, Equals]);
    assert_eq!(c.display(), "15");
    c.press(Equals);
    assert_eq!(c.display(), "18");
    enter(
        &mut c,
        &[Clear, Eight, Add, Multiply, Two, Add, One, Equals],
    );
    assert_eq!(c.display(), "17");
    enter(&mut c, &[Clear, Five, Add, Equals, Equals]);
    assert_eq!(c.display(), "15");
}

#[test]
fn entry_editing_sign_and_percent() {
    let mut c = Calculator::create();
    enter(
        &mut c,
        &[
            Decimal, One, Decimal, Two, Backspace, Add, Decimal, Two, Equals,
        ],
    );
    assert_eq!(c.display(), "0.3");
    enter(&mut c, &[Clear, Negate, Five, Add, Negate, Two, Equals]);
    assert_eq!(c.display(), "-7");
    enter(&mut c, &[Clear, Five, Zero, Percent]);
    assert_eq!(c.display(), "0.5");
    enter(
        &mut c,
        &[Clear, Two, Add, Five, Zero, Percent, Add, One, Equals],
    );
    assert_eq!(c.display(), "3.5");
    // Computed displays cannot be edited into invalid exponential strings.
    c.show(1e-15);
    c.fresh = true;
    enter(&mut c, &[Negate, Backspace, Two]);
    assert_eq!(c.display(), "2");
}

#[test]
fn errors_recover_with_new_number_or_clear() {
    let mut c = Calculator::create();
    enter(&mut c, &[One, Divide, Zero, Equals, Add, Equals]);
    assert_eq!(c.display(), "Cannot divide by zero");
    c.press(Two);
    assert_eq!(c.display(), "2");
    enter(&mut c, &[Divide, Zero, Equals, Clear]);
    assert_eq!(c.display(), "0");
    c.show(f64::MAX);
    enter(&mut c, &[Multiply, Nine, Equals]);
    assert_eq!(c.display(), "Overflow");
    c.press(Two);
    assert_eq!(c.display(), "2");
}

#[test]
fn display_keeps_integer_zeros_and_small_values() {
    let mut c = Calculator::create();
    enter(&mut c, &[One, Zero, Zero, Add, Zero, Equals]);
    assert_eq!(c.display(), "100");
    c.show(1e-15);
    assert!(c.number() > 0.0);
    c.show(f64::INFINITY);
    assert_eq!(c.display(), "Overflow");
    c.press(Decimal);
    assert_eq!(c.display(), "0.");
}
