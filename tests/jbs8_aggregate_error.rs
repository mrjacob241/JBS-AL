use jbs::{Runtime, Value};

fn eval(source: &str) -> Value {
    let mut runtime = Runtime::new();
    runtime
        .eval_script(source)
        .unwrap_or_else(|error| panic!("{source} failed: {error}"))
}

fn assert_true(source: &str) {
    assert_eq!(eval(source), Value::Boolean(true), "{source}");
}

fn assert_eval(source: &str, expected: Value) {
    assert_eq!(eval(source), expected, "{source}");
}

#[test]
fn aggregate_error_copies_array_like_errors() {
    assert_eval(
        "var input = {0: 'a', 1: 'b', length: 2}; var e = new AggregateError(input, 'many'); e.errors.length + ':' + e.errors[0] + e.errors[1];",
        Value::String("2:ab".to_owned()),
    );
    assert_true(
        "var input = [1, 2]; var e = new AggregateError(input, 'many'); e.errors !== input",
    );
    assert_true("Array.isArray(new AggregateError([1], 'many').errors)");
}

#[test]
fn aggregate_error_supports_cause_option() {
    assert_eval(
        "new AggregateError([1], 'many', { cause: 42 }).cause",
        Value::Number(42.0),
    );
    assert_true(
        "var e = new AggregateError([], undefined, { cause: 'why' }); e.cause === 'why' && e.message === ''",
    );
    assert_true("!Object.hasOwn(new AggregateError([], 'many'), 'cause')");
    assert_true("Object.hasOwn(new AggregateError([], 'many', { cause: undefined }), 'cause')");
}

#[test]
fn aggregate_error_uses_custom_new_target_prototype() {
    assert_true(
        "var custom = { x: 42 };
         var newt = new Proxy(function () {}, { get: function (target, key) { return key === 'prototype' ? custom : target[key]; } });
         var error = Reflect.construct(AggregateError, [[]], newt);
         Object.getPrototypeOf(error) === custom && error.x === 42",
    );
}

#[test]
fn native_errors_share_message_coercion_and_cause_options() {
    assert_eval(
        "Error({ toString: function() { return 'coerced'; } }).message",
        Value::String("coerced".to_owned()),
    );
    assert_eval(
        "TypeError('bad', Object.create({ cause: 7 })).cause",
        Value::Number(7.0),
    );
    assert_true("var e = RangeError('bad', 12); !Object.hasOwn(e, 'cause')");
}

#[test]
fn error_own_property_descriptors_match_native_error_fields() {
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(Error('x', { cause: 1 }), 'cause'); d.value === 1 && d.writable === true && d.enumerable === false && d.configurable === true",
    );
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(new AggregateError([1], 'many'), 'errors'); Array.isArray(d.value) && d.writable === true && d.enumerable === false && d.configurable === true",
    );
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(new AggregateError([], 'many'), 'message'); d.value === 'many' && d.writable === true && d.enumerable === false && d.configurable === true",
    );
    assert_true("Object.getOwnPropertyDescriptor(AggregateError, 'prototype').writable === false");
    assert_true(
        "Object.getOwnPropertyDescriptor(globalThis, 'AggregateError').configurable === true",
    );
}
