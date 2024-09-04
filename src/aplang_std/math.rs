use crate::{std_function};
use crate::interpreter::{Env, Value};

pub(super) fn std_math(env: &mut Env) {
    // Trigonometric Functions
    std_function!(env.functions => fn SIN(angle: Value::Number) {
        let result = f64::sin(angle);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn COS(angle: Value::Number) {
        let result = f64::cos(angle);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn TAN(angle: Value::Number) {
        let result = f64::tan(angle);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn ASIN(value: Value::Number) {
        let result = f64::asin(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn ACOS(value: Value::Number) {
        let result = f64::acos(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn ATAN(value: Value::Number) {
        let result = f64::atan(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn ATAN2(y: Value::Number, x: Value::Number) {
        let result = f64::atan2(y, x);
        return Ok(Value::Number(result));
    });

    // Hyperbolic Functions
    std_function!(env.functions => fn SINH(value: Value::Number) {
        let result = f64::sinh(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn COSH(value: Value::Number) {
        let result = f64::cosh(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn TANH(value: Value::Number) {
        let result = f64::tanh(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn ASINH(value: Value::Number) {
        let result = f64::asinh(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn ACOSH(value: Value::Number) {
        let result = f64::acosh(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn ATANH(value: Value::Number) {
        let result = f64::atanh(value);
        return Ok(Value::Number(result));
    });

    // Exponential and Logarithmic Functions
    std_function!(env.functions => fn EXP(value: Value::Number) {
        let result = f64::exp(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn LOG(value: Value::Number, base: Value::Number) {
        let result = f64::log(value, base);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn LOG10(value: Value::Number) {
        let result = f64::log10(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn LOG2(value: Value::Number) {
        let result = f64::log2(value);
        return Ok(Value::Number(result));
    });

    // Rounding and Clamping Functions
    std_function!(env.functions => fn ROUND(value: Value::Number) {
        let result = f64::round(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn FLOOR(value: Value::Number) {
        let result = f64::floor(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn CEIL(value: Value::Number) {
        let result = f64::ceil(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn INT(value: Value::Number) {
        let result = f64::trunc(value);
        return Ok(Value::Number(result));
    });

    std_function!(env.functions => fn CLAMP(value: Value::Number, min: Value::Number, max: Value::Number) {
        let result = value.max(min).min(max);
        return Ok(Value::Number(result));
    });

    // Constants
    std_function!(env.functions => fn PI() {
        return Ok(Value::Number(std::f64::consts::PI));
    });

    std_function!(env.functions => fn E() {
        return Ok(Value::Number(std::f64::consts::E));
    });

    std_function!(env.functions => fn TAU() {
        return Ok(Value::Number(std::f64::consts::TAU));
    });
}