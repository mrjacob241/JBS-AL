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
fn iterator_from_wraps_valid_iterator_like_return_protocol() {
    assert_true(
        "var wrapper = Iterator.from({});
         var result = wrapper.return();
         result.hasOwnProperty('value') &&
         result.value === undefined &&
         result.done === true",
    );
    assert_eval(
        "var called = 0;
         var base = {
           next: function () { return { value: 1, done: false }; },
           return: function () { called = called + 1; return { value: 5, done: true }; }
         };
         var result = Iterator.from(base).return();
         called + result.value;",
        Value::Number(6.0),
    );
    assert_eval(
        "var base = { next: function () { return { value: 9, done: false }; } };
         Iterator.from(base).next().value;",
        Value::Number(9.0),
    );
}

#[test]
fn iterator_prototype_symbol_dispose_calls_return_protocol() {
    assert_true("typeof Symbol.dispose === 'symbol'");
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(Iterator.prototype, Symbol.dispose);
         typeof d.value === 'function' &&
         d.value.name === '[Symbol.dispose]' &&
         d.value.length === 0 &&
         d.writable === true &&
         d.enumerable === false &&
         d.configurable === true",
    );
    assert_eval(
        "var seenThis;
         var seenArgs = 9;
         var iter = {
           return: function () { seenThis = this; seenArgs = arguments.length; return { done: true }; }
         };
         var result = Iterator.prototype[Symbol.dispose].call(iter);
         (result === undefined) + ':' + (seenThis === iter) + ':' + seenArgs;",
        Value::String("true:true:0".to_owned()),
    );
    assert_eval(
        "Iterator.prototype[Symbol.dispose].call({});",
        Value::Undefined,
    );
}

#[test]
fn iterator_concat_return_uses_helper_lifecycle_state() {
    assert_eval(
        "var returnCount = 0;
         var inner = {
           next: function () { return { value: 1, done: false }; },
           return: function () { returnCount = returnCount + 1; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return inner; };
         var iterator = Iterator.concat(source);
         iterator.next();
         iterator.return();
         iterator.return();
         returnCount;",
        Value::Number(1.0),
    );
    assert_eval(
        "var touched = 0;
         var inner = {
           next: function () { touched = touched + 10; throw 'bad'; },
           return: function () { touched = touched + 100; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { touched = touched + 1; return inner; };
         var iterator = Iterator.concat(source);
         iterator.return();
         iterator.next().done === true && touched === 0;",
        Value::Boolean(true),
    );
    assert_eval(
        "var returnCount = 0;
         var inner = {
           next: function () { return { value: undefined, done: true }; },
           return: function () { returnCount = returnCount + 1; throw 'bad'; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return inner; };
         var iterator = Iterator.concat(source);
         iterator.next();
         iterator.return();
         returnCount;",
        Value::Number(0.0),
    );
    assert_eval(
        "var argc = 9;
         var inner = {
           next: function () { return { value: 1, done: false }; },
           return: function () { argc = arguments.length; return {}; }
         };
         var source = {};
         source[Symbol.iterator] = function () { return inner; };
         var iterator = Iterator.concat(source);
         iterator.next();
         iterator.return(1, 2);
         argc;",
        Value::Number(0.0),
    );
    assert_eval(
        "var iterator;
         var saw = '';
         var inner = {
           next: function () {
             try { iterator.next(); } catch (error) { saw = error.name; }
             return { value: 1, done: false };
           }
         };
         var source = {};
         source[Symbol.iterator] = function () { return inner; };
         iterator = Iterator.concat(source);
         iterator.next();
         saw;",
        Value::String("TypeError".to_owned()),
    );
}

#[test]
fn iterator_take_drop_are_direct_lazy_helpers() {
    assert_eval(
        "var obj = { done: false, value: 7 };
         var { done, value } = obj;
         done === false && value === 7;",
        Value::Boolean(true),
    );
    assert_eval(
        "var effects = [];
         Iterator.prototype.take.call({
           get next() {
             effects.push('get next');
             return function () { return { done: true, value: undefined }; };
           }
         }, {
           valueOf: function () { effects.push('ToNumber limit'); return 0; }
         });
         effects.join(',');",
        Value::String("ToNumber limit,get next".to_owned()),
    );
    assert_eval(
        "var effects = [];
         try {
           Iterator.prototype.drop.call({
             get next() {
               effects.push('get next');
               return function () { return { done: true, value: undefined }; };
             }
           }, NaN);
         } catch (e) {}
         effects.join(',');",
        Value::String(String::new()),
    );
    assert_eval(
        "var helper = Iterator.prototype.take.call({ next: 0 }, 1);
         try { helper.next(); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var i = 0;
         var source = { next: function () { i = i + 1; return { value: i, done: i > 3 }; } };
         Iterator.prototype.drop.call(source, 1).next().value;",
        Value::Number(2.0),
    );
    assert_eval(
        "var i = 0;
         var source = { next: function () { i = i + 1; return { value: i, done: i > 2 }; } };
         var { done, value } = Iterator.prototype.take.call(source, 1).next();
         done === false && value === 1;",
        Value::Boolean(true),
    );
}

#[test]
fn iterator_callback_validation_closes_without_reading_next() {
    assert_eval(
        "var o = { __proto__: Iterator.prototype };
         typeof o.map;",
        Value::String("function".to_owned()),
    );
    assert_eval(
        "var methods = ['map', 'filter', 'find', 'flatMap', 'forEach', 'reduce', 'some', 'every'];
         var closed = 0;
         var nextReads = 0;
         for (var i = 0; i < methods.length; i++) {
           var iterator = {
             get next() { nextReads = nextReads + 1; throw 'bad'; },
             return() { closed = closed + 1; return {}; }
           };
           iterator.__proto__ = Iterator.prototype;
           try { iterator[methods[i]](); } catch (e) {}
           try { iterator[methods[i]]({}); } catch (e) {}
         }
         closed + ':' + nextReads;",
        Value::String("16:0".to_owned()),
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
