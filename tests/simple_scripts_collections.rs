use std::fs;
use std::path::Path;

use jbs::{Runtime, Value};

#[test]
fn parser_stress_scripts_pass() {
    run_manifest("SimpleScripts/ParserStress", 50);
}

#[test]
fn parser_jbs2_stress_scripts_pass() {
    run_manifest("SimpleScripts/ParserJBS2", 50);
}

#[test]
fn parser_map_iterator_scripts_parse() {
    run_parse_manifest("SimpleScripts/ParserMapIterators", 50);
}

#[test]
fn parser_jbs9_hardened_scripts_pass() {
    run_manifest("SimpleScripts/ParserJBS9Hardened", 100);
}

#[test]
fn regexp_focused_simple_scripts_pass() {
    run_manifest("SimpleScripts/RegExpFocused", 100);
}

#[test]
fn jbs1_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS1", 100);
}

#[test]
fn jbs2_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS2", 50);
}

#[test]
fn jbs2_extra_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS2Extra", 50);
}

#[test]
fn jbs3_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS3", 50);
}

#[test]
fn jbs3_focused_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS3Focused", 50);
}

#[test]
fn jbs4_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS4", 50);
}

#[test]
fn jbs5_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS5", 100);
}

#[test]
fn jbs6_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS6", 100);
}

#[test]
fn jbs7_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS7", 50);
}

#[test]
fn jbs7_regression_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS7Regressions", 7);
}

#[test]
fn jbs8_environment_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS8Env", 5);
}

#[test]
fn jbs8_iterators_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS8Iterators", 13);
}

#[test]
fn jbs8_errors_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS8Errors", 10);
}

#[test]
fn jbs8_focused_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS8Focused", 100);
}

#[test]
fn jbs9_arrays_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS9Arrays", 100);
}

#[test]
fn jbs10_iterators_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS10", 100);
}

#[test]
fn jbs11_collections_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBS11", 100);
}

#[test]
fn jbs_focused_simple_scripts_pass() {
    run_manifest("SimpleScripts/JBSFocused", 100);
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
        let mut runtime = Runtime::new();
        let actual = runtime
            .eval_script(&source)
            .unwrap_or_else(|error| panic!("{}/{} failed: {}", root.display(), parts[0], error));
        let expected = expected_value(parts[1], parts[2]);
        assert_eq!(actual, expected, "{} returned wrong value", parts[0]);
        count += 1;
    }

    assert_eq!(count, expected_count);
}

fn run_parse_manifest(root: &str, expected_count: usize) {
    let root = Path::new(root);
    let manifest = fs::read_to_string(root.join("manifest.txt")).unwrap();
    let mut count = 0;

    for raw in manifest.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let source = fs::read_to_string(root.join(line)).unwrap();
        jbs::syntax::parse_only(&source).unwrap_or_else(|error| {
            panic!("{}/{} failed to parse: {}", root.display(), line, error)
        });
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
