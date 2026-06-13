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
fn arrays_and_strings_are_iterable_by_for_of_and_array_from() {
    assert_eval(
        "var sum = 0; for (var value of [1, 2, 3]) { sum = sum + value; } sum;",
        Value::Number(6.0),
    );
    assert_eval(
        "var text = ''; for (var ch of 'abc') { text = text + ch; } text;",
        Value::String("abc".to_owned()),
    );
    assert_eval("Array.from([4, 5])[1];", Value::Number(5.0));
    assert_eval("Array.from('xy')[0];", Value::String("x".to_owned()));
}

#[test]
fn built_in_iterator_objects_are_iterable_themselves() {
    assert_true("var it = [1, 2].values(); it[Symbol.iterator]() === it");
    assert_true("var it = [1, 2].keys(); it[Symbol.iterator]() === it");
    assert_true("var it = [1, 2].entries(); it[Symbol.iterator]() === it");
    assert_true("var it = 'ab'[Symbol.iterator](); it[Symbol.iterator]() === it");
    assert_eval(
        "var it = [3, 4].values(); var sum = 0; for (var value of it) { sum = sum + value; } sum;",
        Value::Number(7.0),
    );
}

#[test]
fn iterator_result_objects_have_value_then_done_properties() {
    assert_true(
        "var result = [9].values().next();
         result.value === 9 &&
         result.done === false &&
         Object.keys(result).length === 2 &&
         Object.keys(result)[0] === 'value' &&
         Object.keys(result)[1] === 'done'",
    );
    assert_true(
        "var it = [].values();
         var result = it.next();
         result.value === undefined &&
         result.done === true &&
         Object.keys(result).length === 2 &&
         Object.keys(result)[0] === 'value' &&
         Object.keys(result)[1] === 'done'",
    );
}

#[test]
fn for_of_closes_iterators_on_break() {
    assert_eval(
        "var closed = 0;
         var iter = {
           next: function () { return { value: 1, done: false }; },
           return: function () { closed = closed + 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return iter; };
         for (var value of source) { break; }
         closed;",
        Value::Number(1.0),
    );
}

#[test]
fn for_of_closes_iterators_on_throw() {
    assert_eval(
        "var closed = 0;
         var iter = {
           next: function () { return { value: 1, done: false }; },
           return: function () { closed = closed + 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return iter; };
         try {
           for (var value of source) { throw 'stop'; }
         } catch (error) {}
         closed;",
        Value::Number(1.0),
    );
}

#[test]
fn array_from_closes_iterators_when_mapping_abruptly_completes() {
    assert_eval(
        "var closed = 0;
         var iter = {
           next: function () { return { value: 1, done: false }; },
           return: function () { closed = closed + 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return iter; };
         try {
           Array.from(source, function () { throw 'stop'; });
         } catch (error) {}
         closed;",
        Value::Number(1.0),
    );
}

#[test]
fn object_from_entries_closes_iterators_when_entry_processing_abruptly_completes() {
    assert_eval(
        "var closed = 0;
         var iter = {
           next: function () { return { value: 1, done: false }; },
           return: function () { closed = closed + 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return iter; };
         try {
           Object.fromEntries(source);
         } catch (error) {}
         closed;",
        Value::Number(1.0),
    );
    assert_eval(
        "Object.fromEntries([['a', 1], ['b', 2]]).b;",
        Value::Number(2.0),
    );
}

#[test]
fn iterator_helpers_cover_common_adapter_and_terminal_paths() {
    assert_eval(
        "Iterator.from([1, 2, 3]).map(function (v) { return v * 2; }).toArray().join(',');",
        Value::String("2,4,6".to_owned()),
    );
    assert_eval(
        "Iterator.from([1, 2, 3, 4]).filter(function (v) { return v % 2 === 0; }).toArray().join(',');",
        Value::String("2,4".to_owned()),
    );
    assert_eval(
        "Iterator.from([1, 2, 3, 4]).drop(1).take(2).toArray().join(',');",
        Value::String("2,3".to_owned()),
    );
    assert_eval(
        "Iterator.from([1, 2]).flatMap(function (v) { return [v, v + 10]; }).toArray().join(',');",
        Value::String("1,11,2,12".to_owned()),
    );
    assert_eval(
        "Iterator.from([2, 4]).every(function (v) { return v % 2 === 0; });",
        Value::Boolean(true),
    );
    assert_eval(
        "Iterator.from([1, 3, 4]).some(function (v) { return v % 2 === 0; });",
        Value::Boolean(true),
    );
    assert_eval(
        "Iterator.from([1, 3, 4]).find(function (v) { return v > 2; });",
        Value::Number(3.0),
    );
    assert_eval(
        "var total = 0; Iterator.from([1, 2, 3]).forEach(function (v) { total = total + v; }); total;",
        Value::Number(6.0),
    );
    assert_eval(
        "Iterator.from([1, 2, 3]).reduce(function (a, v) { return a + v; }, 0);",
        Value::Number(6.0),
    );
    assert_eval(
        "var i = 0; var it = { next: function () { i = i + 1; return { value: i, done: i > 3 }; } }; Iterator.prototype.take.call(it, 2).toArray().join(',');",
        Value::String("1,2".to_owned()),
    );
    assert_eval(
        "Iterator.concat([1, 2], ['a']).toArray().join(',');",
        Value::String("1,2,a".to_owned()),
    );
    assert_eval(
        "var touched = 0; var iterable = {}; iterable[Symbol.iterator] = function () { touched = touched + 1; throw 'bad'; }; try { Iterator.concat(iterable, null); } catch (e) {} touched;",
        Value::Number(0.0),
    );
}
