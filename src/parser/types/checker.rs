use super::parse::{Primitive, TypeError};
use super::super::Expander;
use super::super::expand_string;
use types::{Array, Value};

pub enum ReturnValue {
    Str(Value),
    Vector(Array),
}

pub fn is_array(value: &str) -> bool { value.starts_with('[') && value.ends_with(']') }

pub fn is_boolean(value: &str) -> Result<&str, ()> {
    if ["true", "1", "y"].contains(&value) {
        Ok("true")
    } else if ["false", "0", "n"].contains(&value) {
        Ok("false")
    } else {
        Err(())
    }
}

fn is_boolean_string(value: &ReturnValue) -> Result<&str, ()> {
    if let ReturnValue::Str(ref value) = *value {
        is_boolean(&value.as_str())
    } else {
        unreachable!()
    }
}

fn is_integer_string(value: ReturnValue) -> Result<ReturnValue, ()> {
    let is_ok = if let ReturnValue::Str(ref num) = value {
        num.parse::<i64>().is_ok()
    } else {
        unreachable!()
    };

    if is_ok { Ok(value) } else { Err(()) }
}

fn is_float_string(value: ReturnValue) -> Result<ReturnValue, ()> {
    let is_ok = if let ReturnValue::Str(ref num) = value {
        num.parse::<f64>().is_ok()
    } else {
        unreachable!()
    };

    if is_ok { Ok(value) } else { Err(()) }
}

fn is_boolean_array(values: &mut ReturnValue) -> bool {
    if let ReturnValue::Vector(ref mut values) = *values {
        for element in values.iter_mut() {
            let boolean = {
                match is_boolean(&element) {
                    Ok(boolean) => boolean.into(),
                    Err(()) => return false,
                }
            };
            *element = boolean;
        }
        true
    } else {
        unreachable!()
    }
}

fn is_integer_array(value: ReturnValue) -> Result<ReturnValue, ()> {
    let is_ok = if let ReturnValue::Vector(ref nums) = value {
        nums.iter().all(|num| num.parse::<i64>().is_ok())
    } else {
        unreachable!()
    };

    if is_ok { Ok(value) } else { Err(()) }
}

fn is_float_array(value: ReturnValue) -> Result<ReturnValue, ()> {
    let is_ok = if let ReturnValue::Vector(ref nums) = value {
        nums.iter().all(|num| num.parse::<f64>().is_ok())
    } else {
        unreachable!()
    };

    if is_ok { Ok(value) } else { Err(()) }
}

fn get_string<E: Expander>(shell: &E, value: &str) -> ReturnValue {
    ReturnValue::Str(expand_string(value, shell, false).join(" "))
}

fn get_array<E: Expander>(shell: &E, value: &str) -> ReturnValue {
    ReturnValue::Vector(expand_string(value, shell, false))
}

pub fn value_check<'a, E: Expander>(
    shell: &E,
    value: &'a str,
    expected: Primitive,
) -> Result<ReturnValue, TypeError<'a>> {
    macro_rules! string { () => { get_string(shell, value) } }
    macro_rules! array { () => { get_array(shell, value) } }
    let is_array = is_array(value);
    match expected {
        Primitive::Any if is_array => Ok(array!()),
        Primitive::Any => Ok(string!()),
        Primitive::AnyArray if is_array => Ok(array!()),
        Primitive::Str if !is_array => Ok(string!()),
        Primitive::StrArray if is_array => Ok(array!()),
        Primitive::Boolean if !is_array => {
            let value = string!();
            let value = is_boolean_string(&value).map_err(
                |_| TypeError::BadValue(expected),
            )?;
            Ok(ReturnValue::Str(value.to_owned()))
        }
        Primitive::BooleanArray if is_array => {
            let mut values = array!();
            if is_boolean_array(&mut values) {
                Ok(values)
            } else {
                Err(TypeError::BadValue(expected))
            }
        }
        Primitive::Integer if !is_array => is_integer_string(string!()).map_err(|_| TypeError::BadValue(expected)),
        Primitive::IntegerArray if is_array => is_integer_array(array!()).map_err(|_| TypeError::BadValue(expected)),
        Primitive::Float if !is_array => is_float_string(string!()).map_err(|_| TypeError::BadValue(expected)),
        Primitive::FloatArray if is_array => is_float_array(array!()).map_err(|_| TypeError::BadValue(expected)),
        _ => Err(TypeError::BadValue(expected)),
    }
}
