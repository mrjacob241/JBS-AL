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

#[test]
fn json_global_is_ordinary_singleton_with_methods() {
    assert_eval("typeof JSON;", Value::String("object".to_owned()));
    assert_eval("Array.isArray(JSON);", Value::Boolean(false));
    assert_eval("JSON.stringify.length;", Value::Number(3.0));
    assert_eval("JSON.parse.length;", Value::Number(2.0));
    assert_eval(
        "Object.prototype.toString.call(JSON);",
        Value::String("[object JSON]".to_owned()),
    );
}

#[test]
fn json_global_can_participate_in_descriptor_builtins() {
    assert_eval(
        "var obj = {}; JSON.prop = { value: 15, enumerable: true }; Object.defineProperties(obj, JSON); obj.prop;",
        Value::Number(15.0),
    );
    assert_eval(
        "var obj = {}; var result = false; Object.defineProperty(JSON, 'prop2', { get: function() { result = this === JSON; return {}; }, enumerable: true, configurable: true }); Object.defineProperties(obj, JSON); result;",
        Value::Boolean(true),
    );
}

#[test]
fn json_stringify_serializes_basic_values_arrays_and_objects() {
    assert_eval("JSON.stringify(null);", Value::String("null".to_owned()));
    assert_eval("JSON.stringify(true);", Value::String("true".to_owned()));
    assert_eval(
        "JSON.stringify('a\\nb');",
        Value::String("\"a\\nb\"".to_owned()),
    );
    assert_eval(
        "JSON.stringify([1, undefined, 'x']);",
        Value::String("[1,null,\"x\"]".to_owned()),
    );
    assert_eval(
        "JSON.stringify({ a: 1, b: undefined, c: 'x' });",
        Value::String("{\"a\":1,\"c\":\"x\"}".to_owned()),
    );
}

#[test]
fn json_parse_builds_plain_objects_and_arrays() {
    assert_eval("JSON.parse('null');", Value::Null);
    assert_eval(
        "JSON.parse('{\"a\":1,\"b\":[true,null,\"x\"]}').b[2];",
        Value::String("x".to_owned()),
    );
    assert_eval(
        "Array.isArray(JSON.parse('[1,2,3]'));",
        Value::Boolean(true),
    );
}
