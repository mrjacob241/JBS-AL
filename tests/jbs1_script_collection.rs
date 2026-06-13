use std::fs;
use std::path::Path;

use jbs::{Runtime, Value};

#[test]
fn jbs1_plain_js_scripts_pass() {
    let root = Path::new("SimpleScripts/JBS1Scripts");
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
        let mut runtime = Runtime::new();
        let actual = runtime
            .eval_script(&source)
            .unwrap_or_else(|error| panic!("{} failed: {}", parts[0], error));
        let expected = expected_value(parts[1], parts[2]);
        assert_eq!(actual, expected, "{} returned wrong value", parts[0]);
        count += 1;
    }

    assert_eq!(count, 50);
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
