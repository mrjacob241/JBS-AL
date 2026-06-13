use jbs::{Runtime, Value};

fn eval(source: &str) -> Value {
    let mut runtime = Runtime::new();
    runtime
        .eval_script(source)
        .unwrap_or_else(|error| panic!("{source} failed: {error}"))
}

fn assert_eval(source: &str, expected: Value) {
    assert_eq!(eval(source), expected, "{source}");
}

fn assert_true(source: &str) {
    assert_eval(source, Value::Boolean(true));
}

#[test]
fn number_static_constants_are_installed_with_intrinsic_descriptors() {
    assert_true("Number.MAX_VALUE > 1");
    assert_true("Number.MIN_VALUE > 0 && Number.MIN_VALUE < 1");
    assert_eval(
        "Number.MAX_SAFE_INTEGER",
        Value::Number(9_007_199_254_740_991.0),
    );
    assert_eval(
        "Number.MIN_SAFE_INTEGER",
        Value::Number(-9_007_199_254_740_991.0),
    );
    assert_true("Number.EPSILON > 0 && Number.EPSILON < 1");
    assert_true("Number.NaN !== Number.NaN");
    assert_eval("Number.NEGATIVE_INFINITY", Value::Number(f64::NEG_INFINITY));
    assert_eval("Number.POSITIVE_INFINITY", Value::Number(f64::INFINITY));
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(Number, 'MAX_VALUE'); d.writable === false && d.enumerable === false && d.configurable === false",
    );
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(Number, 'NaN'); d.writable === false && d.enumerable === false && d.configurable === false",
    );
}

#[test]
fn string_function_accepts_symbols_and_reflect_has_tostringtag() {
    assert_eval(
        "String(Symbol.toStringTag)",
        Value::String("Symbol(toStringTag)".to_owned()),
    );
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(Reflect, Symbol.toStringTag);
         d.value === 'Reflect' && d.writable === false && d.enumerable === false && d.configurable === true",
    );
}

#[test]
fn array_from_mapper_defaults_to_global_receiver_when_no_this_arg_is_supplied() {
    assert_true("Array.from([1], function () { return this === globalThis; })[0]");
    assert_eval(
        "Array.from([1], function () { return this.marker; }, { marker: 37 })[0]",
        Value::Number(37.0),
    );
}

#[test]
fn array_of_constructs_through_custom_constructor_and_sets_length() {
    assert_eval(
        "function Bag(len) { this.constructedLength = len; } var result = Array.of.call(Bag, 'a', 'b'); result.constructedLength;",
        Value::Number(2.0),
    );
    assert_true(
        "function Bag(len) { this.constructedLength = len; } var result = Array.of.call(Bag, 'a', 'b'); result instanceof Bag && result[0] === 'a' && result[1] === 'b'",
    );
    assert_eval(
        "function Bag() { Object.defineProperty(this, 'length', { set: function (v) { this.seen = v; }, configurable: true }); } Array.of.call(Bag, 1, 2, 3).seen;",
        Value::Number(3.0),
    );
    assert_true(
        "function Broken() { throw new Test262Error(); } var caught = false; try { Array.of.call(Broken); } catch (error) { caught = error instanceof Test262Error; } caught",
    );
}

#[test]
fn aggregate_error_consumes_iterables_before_array_like_fallback() {
    assert_eval(
        "var obj = {0: 'array-like', length: 1}; obj[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: 'iter-' + i, done: i > 2 }; } }; }; var e = new AggregateError(obj, 'many'); e.errors.length + ':' + e.errors[0] + ':' + e.errors[1];",
        Value::String("2:iter-1:iter-2".to_owned()),
    );
    assert_eval(
        "var e = new AggregateError({0: 'x', length: 1}, 'many'); e.errors[0];",
        Value::String("x".to_owned()),
    );
}

#[test]
fn indexed_iterator_objects_are_iterable() {
    assert_eval(
        "var total = 0; for (var pair of [3, 4].entries()) { total = total + pair[1]; } total;",
        Value::Number(7.0),
    );
    assert_true("var it = [1].values(); it[Symbol.iterator]() === it");
}
