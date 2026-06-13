use std::{fs, path::Path};

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
fn symbol_iterator_is_well_known_symbol() {
    assert_eval("typeof Symbol;", Value::String("function".to_owned()));
    assert_eval(
        "typeof Symbol.iterator;",
        Value::String("symbol".to_owned()),
    );
    assert_eval("Symbol.iterator === Symbol.iterator;", Value::Boolean(true));
    assert_eval("Symbol() === Symbol();", Value::Boolean(false));
}

#[test]
fn array_from_and_of_descriptors() {
    assert_eval("Array.from.length;", Value::Number(1.0));
    assert_eval("Array.from.name;", Value::String("from".to_owned()));
    assert_eval("Array.of.length;", Value::Number(0.0));
    assert_eval("Array.of.name;", Value::String("of".to_owned()));
    assert_eval(
        "Object.getOwnPropertyDescriptor(Array, 'from').enumerable;",
        Value::Boolean(false),
    );
    assert_eval(
        "Object.getOwnPropertyDescriptor(Array, 'of').configurable;",
        Value::Boolean(true),
    );
}

#[test]
fn arrays_and_strings_are_iterable() {
    assert_eval(
        "var it = [2, 4][Symbol.iterator](); it.next().value;",
        Value::Number(2.0),
    );
    assert_eval(
        "var it = [2, 4].keys(); it.next().value;",
        Value::Number(0.0),
    );
    assert_eval(
        "var it = [2, 4].entries(); it.next().value[1];",
        Value::Number(2.0),
    );
    assert_eval(
        "var it = 'ab'[Symbol.iterator](); it.next().value + it.next().value;",
        Value::String("ab".to_owned()),
    );
    assert_eval(
        "var it = [1][Symbol.iterator](); it.next(); it.next().done;",
        Value::Boolean(true),
    );
}

#[test]
fn array_from_uses_iterables_and_array_like_fallback() {
    assert_eval("Array.from([3, 5])[1];", Value::Number(5.0));
    assert_eval("Array.from('abc').length;", Value::Number(3.0));
    assert_eval("Array.from('abc')[2];", Value::String("c".to_owned()));
    assert_eval(
        "Array.from({0: 'x', 1: 'y', length: 2})[1];",
        Value::String("y".to_owned()),
    );
    assert_eval(
        "Array.from([1, 2], function (v, i) { return v + i; })[1];",
        Value::Number(3.0),
    );
    assert_eval("Array.of(7).length;", Value::Number(1.0));
    assert_eval("Array.of(7, 8)[1];", Value::Number(8.0));
}

#[test]
fn array_from_constructs_custom_this_for_array_like_inputs() {
    assert_eval(
        "function MyCollection() { this.args = arguments; } var result = Array.from.call(MyCollection, { length: 42 }); result.args.length;",
        Value::Number(1.0),
    );
    assert_eval(
        "function MyCollection() { this.args = arguments; } var result = Array.from.call(MyCollection, { length: 42 }); result.args[0];",
        Value::Number(42.0),
    );
    assert_eval(
        "function MyCollection() { this.args = arguments; } var result = Array.from.call(MyCollection, { length: 1, 0: 'x' }); result instanceof MyCollection;",
        Value::Boolean(true),
    );
}

#[test]
fn array_from_uses_iterable_constructor_path_without_length_argument() {
    assert_eval(
        "var seenThis, seenArgs, calls = 0;
         function C() { seenThis = this; seenArgs = arguments; calls = calls + 1; }
         var items = {};
         items[Symbol.iterator] = function () { return { next: function () { return { done: true }; } }; };
         var result = Array.from.call(C, items);
         result instanceof C && result.constructor === C && seenThis === result && seenArgs.length === 0 && calls === 1;",
        Value::Boolean(true),
    );
}

#[test]
fn array_from_uses_undefined_mapper_receiver_when_this_arg_absent() {
    assert_eval(
        "var seen;
         Array.from([1, 2], function () { 'use strict'; seen = this; });
         seen === undefined;",
        Value::Boolean(true),
    );
}

#[test]
fn array_from_closes_iterable_when_result_element_define_fails() {
    assert_eval(
        "function C() {
           Object.defineProperty(this, '0', { writable: true, configurable: false });
         }
         var closeCount = 0;
         var nextResult = { done: false };
         var items = {};
         items[Symbol.iterator] = function () {
           return {
             return: function () { closeCount = closeCount + 1; },
             next: function () {
               var result = nextResult;
               nextResult = { done: true };
               return result;
             }
           };
         };
         var threw = false;
         try { Array.from.call(C, items); } catch (error) { threw = error instanceof TypeError; }
         threw && closeCount === 1;",
        Value::Boolean(true),
    );
}

#[test]
fn array_from_object_constructor_result_uses_object_constructor() {
    assert_eval(
        "Array.from.call(Object, []).constructor === Object;",
        Value::Boolean(true),
    );
}

#[test]
fn array_from_and_for_of_use_custom_iterator_protocol() {
    let source = "var obj = {}; obj[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: i * 2, done: i > 3 }; } }; }; Array.from(obj)[2];";
    assert_eval(source, Value::Number(6.0));
    let source = "var obj = {}; obj[Symbol.iterator] = function () { var i = 0; return { next: function () { i = i + 1; return { value: i, done: i > 3 }; } }; }; var sum = 0; for (var x of obj) { sum = sum + x; } sum;";
    assert_eval(source, Value::Number(6.0));
}

#[test]
fn for_of_over_arrays_and_strings() {
    assert_eval(
        "var sum = 0; for (var x of [1, 2, 3]) { sum = sum + x; } sum;",
        Value::Number(6.0),
    );
    assert_eval(
        "var out = ''; for (var ch of 'abc') { out = out + ch; } out;",
        Value::String("abc".to_owned()),
    );
    assert_eval(
        "var hit = 0; for (var x of [1, 2, 3]) { if (x === 2) { break; } hit = hit + x; } hit;",
        Value::Number(1.0),
    );
}

#[test]
fn simple_scripts_pass() {
    let root = Path::new("SimpleScripts/JBS8Iterators");
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

    assert_eq!(count, 13);
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
