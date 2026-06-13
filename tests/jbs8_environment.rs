use std::fs;
use std::path::Path;

use jbs::{Runtime, Value};

fn eval(source: &str) -> Value {
    let mut runtime = Runtime::new();
    runtime
        .eval_script(source)
        .unwrap_or_else(|error| panic!("{source} failed: {error}"))
}

#[test]
fn top_level_this_is_global_object() {
    assert_eq!(eval("this === globalThis;"), Value::Boolean(true));
}

#[test]
fn top_level_this_property_write_updates_global_lookup() {
    assert_eq!(
        eval("this.jbs8Value = 41; jbs8Value + globalThis.jbs8Value;"),
        Value::Number(82.0)
    );
}

#[test]
fn bare_function_call_this_uses_global_object_in_non_strict_code() {
    assert_eq!(
        eval("function f() { this.jbsCallValue = 12; } f(); globalThis.jbsCallValue;"),
        Value::Number(12.0)
    );
}

#[test]
fn method_call_this_stays_receiver() {
    assert_eq!(
        eval("var o = { x: 7 }; function f() { return this.x; } o.f = f; o.f();"),
        Value::Number(7.0)
    );
}

#[test]
fn jbs8_environment_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS8Env", 5);
}

fn run_manifest(root: &str, expected_count: usize) {
    let root = Path::new(root);
    let manifest = fs::read_to_string(root.join("manifest.txt")).unwrap();
    let mut count = 0;

    for raw in manifest.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<_> = line.split('|').collect();
        assert_eq!(parts.len(), 3, "bad manifest line: {line}");
        let source = fs::read_to_string(root.join(parts[0])).unwrap();
        let actual = eval(&source);
        let expected = expected_value(parts[1], parts[2]);
        assert_eq!(actual, expected, "{} returned wrong value", parts[0]);
        count += 1;
    }

    assert_eq!(count, expected_count);
}

fn expected_value(kind: &str, raw: &str) -> Value {
    match kind {
        "undefined" => Value::Undefined,
        "null" => Value::Null,
        "boolean" => Value::Boolean(raw.parse().unwrap()),
        "number" => Value::Number(raw.parse().unwrap()),
        "string" => Value::String(raw.to_owned()),
        other => panic!("unknown expected value kind: {other}"),
    }
}
