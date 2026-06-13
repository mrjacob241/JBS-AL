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
fn object_from_entries_closes_iterator_when_entry_processing_fails() {
    assert_eval(
        "var closed = 0;
         var iter = {
           next: function () { return { value: 'xy', done: false }; },
           return: function () { closed = 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return iter; };
         try { Object.fromEntries(source); } catch (e) {}
         closed;",
        Value::Number(1.0),
    );
}

#[test]
fn array_from_closes_iterator_when_mapper_fails() {
    assert_eval(
        "var closed = 0;
         var iter = {
           next: function () { return { value: 1, done: false }; },
           return: function () { closed = 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return iter; };
         try { Array.from(source, function () { throw 'stop'; }); } catch (e) {}
         closed;",
        Value::Number(1.0),
    );
}

#[test]
fn for_of_closes_iterator_on_abrupt_loop_exit() {
    assert_eval(
        "var closed = 0;
         var iter = {
           next: function () { return { value: 1, done: false }; },
           return: function () { closed = 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return iter; };
         for (var value of source) { break; }
         closed;",
        Value::Number(1.0),
    );
}

#[test]
fn array_constructor_results_share_create_data_property_path() {
    assert_eval(
        "function C(length) { this.length = length; }
         var result = Array.from.call(C, [4, 5]);
         result instanceof C && result[0] === 4 && result[1] === 5 && result.length === 2;",
        Value::Boolean(true),
    );
}
