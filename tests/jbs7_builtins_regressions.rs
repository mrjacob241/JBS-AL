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
fn array_prototype_has_array_brand() {
    assert_true("Array.isArray(Array.prototype)");
}

#[test]
fn array_prototype_has_zero_length() {
    assert_true("Array.prototype.length === 0");
}

#[test]
fn object_assign_boxes_primitive_target_and_string_sources() {
    assert_eval(
        "var target = 12; var result = Object.assign(target, 'aaa', 'bb2b', '1c'); result[0] + result[1] + result[2] + result[3];",
        Value::String("1c2b".to_owned()),
    );
    assert_true(
        "var target = 12; var result = Object.assign(target, 'aaa', 'bb2b', '1c'); Object.getOwnPropertyNames(result).length === 4;",
    );
}

#[test]
fn primitive_prototype_methods_unwrap_primitive_and_wrapper_receivers() {
    assert_eval(
        "Boolean.prototype.toString();",
        Value::String("false".to_owned()),
    );
    assert_eval(
        "(new Boolean(true)).toString();",
        Value::String("true".to_owned()),
    );
    assert_eval(
        "(new Number(12)).toString();",
        Value::String("12".to_owned()),
    );
}

#[test]
fn date_global_constructor_and_prototype_shape_are_present() {
    assert_true("typeof Date === 'function'");
    assert_true("Date.length === 7");
    assert_true("Date.name === 'Date'");
    assert_true("Date.prototype.constructor === Date");
    assert_true("Object.getOwnPropertyDescriptor(Date, 'prototype').writable === false");
    assert_true(
        "Object.getOwnPropertyDescriptor(Date.prototype, 'constructor').enumerable === false",
    );
}

#[test]
fn date_call_construct_and_object_tag_have_minimal_ecmascript_shape() {
    assert_true("typeof Date() === 'string'");
    assert_true("Object.getPrototypeOf(new Date()) === Date.prototype");
    assert_true("Object.prototype.toString.call(new Date()) === '[object Date]'");
    assert_true("Object.prototype.toString.call(Date.prototype) === '[object Object]'");
    assert_true("new Date().toString() === 'Invalid Date'");
    assert_true("new Date().valueOf() !== new Date().valueOf()");
    assert_true("new Date().toJSON() === null");
    assert_eval("new Date(0).getTime()", Value::Number(0.0));
    assert_true("1 / new Date(-0).getTime() === Infinity");
    assert_eval(
        "new Date('1970').toISOString()",
        Value::String("1970-01-01T00:00:00.000Z".to_owned()),
    );
    assert_eval("Date.UTC(1970, 0, 1)", Value::Number(0.0));
    assert_eval("Date.UTC(70, 0, 1)", Value::Number(0.0));
    assert_true("Date.UTC() !== Date.UTC()");
    assert_true("typeof Date.now() === 'number'");
    assert_eval("Date.parse('1970-01-01T00:00:00.000Z')", Value::Number(0.0));
    assert_true(
        "Date.UTC.length === 7 && Date.UTC.name === 'UTC' &&
         Date.parse.length === 1 && Date.now.length === 0",
    );
    assert_true(
        "var log = '';
         var year = { valueOf: function () { log += 'year'; return 1970; } };
         var month = { valueOf: function () { log += 'month'; return 0; } };
         var date = { valueOf: function () { log += 'date'; return 1; } };
         Date.UTC(year, month, date);
         log === 'yearmonthdate'",
    );
    assert_true(
        "var d = new Date(Date.UTC(2016, 6, 5, 15, 34, 45, 876));
         d.getUTCFullYear() === 2016 &&
         d.getUTCMonth() === 6 &&
         d.getUTCDate() === 5 &&
         d.getUTCDay() === 2 &&
         d.getUTCHours() === 15 &&
         d.getUTCMinutes() === 34 &&
         d.getUTCSeconds() === 45 &&
         d.getUTCMilliseconds() === 876",
    );
    assert_true(
        "var d = new Date(Date.UTC(2016, 6, 5, 15, 34, 45, 876));
         d.getFullYear() === d.getUTCFullYear() &&
         d.getMonth() === d.getUTCMonth() &&
         d.getDate() === d.getUTCDate() &&
         d.getDay() === d.getUTCDay() &&
         d.getHours() === d.getUTCHours() &&
         d.getMinutes() === d.getUTCMinutes() &&
         d.getSeconds() === d.getUTCSeconds() &&
         d.getMilliseconds() === d.getUTCMilliseconds() &&
         d.getTimezoneOffset() === 0",
    );
    assert_true("new Date().getUTCFullYear() !== new Date().getUTCFullYear()");
    assert_eval(
        "Date.prototype.toISOString.call(new Date(0))",
        Value::String("1970-01-01T00:00:00.000Z".to_owned()),
    );
    assert_true(
        "var threw = false;
         try { new Date().toISOString(); } catch (error) { threw = error instanceof RangeError; }
         threw",
    );
    assert_eval(
        "Date.prototype.toJSON.call({ valueOf: function () { return 0; }, toISOString: function () { return 'ok'; } })",
        Value::String("ok".to_owned()),
    );
    assert_true("typeof Date.prototype[Symbol.toPrimitive] === 'function'");
    assert_eval(
        "Date.prototype[Symbol.toPrimitive].call({ toString: function () { return 's'; }, valueOf: function () { return 1; } }, 'default')",
        Value::String("s".to_owned()),
    );
    assert_eval(
        "Date.prototype[Symbol.toPrimitive].call({ toString: function () { return 's'; }, valueOf: function () { return 1; } }, 'number')",
        Value::Number(1.0),
    );
    assert_eval(
        "var d = new Date(0); d.setTime(1000); d.getTime();",
        Value::Number(1000.0),
    );
    assert_eval(
        "var d = new Date(Date.UTC(2016, 6, 1)); d.setUTCDate(2); d.getUTCDate();",
        Value::Number(2.0),
    );
    assert_eval(
        "var d = new Date(Date.UTC(2016, 6, 1)); d.setUTCFullYear(2017, 0, 3); d.getUTCFullYear() * 10000 + d.getUTCMonth() * 100 + d.getUTCDate();",
        Value::Number(20170003.0),
    );
    assert_eval(
        "var old = new Date(1438560000000);
         old.toString = function () { throw 'bad'; };
         old.valueOf = function () { throw 'bad'; };
         new Date(old).getTime();",
        Value::Number(1438560000000.0),
    );
    assert_true(
        "var d = new Date(0);
         var arg = { valueOf: function () { d.setTime(NaN); return 1; } };
         var result = d.setUTCMilliseconds(arg);
         result === d.getTime() && d.getUTCMilliseconds() === 1",
    );
    assert_true(
        "var d = new Date(NaN);
         var arg = { valueOf: function () { d.setTime(0); return 1; } };
         var result = d.setUTCMilliseconds(arg);
         result !== result && d.getTime() === 0",
    );
    assert_eval(
        "new Date(8640000000000000).toISOString()",
        Value::String("+275760-09-13T00:00:00.000Z".to_owned()),
    );
    assert_eval(
        "Date.parse('+275760-09-13T00:00:00.000Z')",
        Value::Number(8640000000000000.0),
    );
    assert_true(
        "Date.parse('+275760-09-13T00:00:00.001Z') !== Date.parse('+275760-09-13T00:00:00.001Z')",
    );
    assert_eval(
        "new Date('2014-03-23T00:00:00Z').toUTCString()",
        Value::String("Sun, 23 Mar 2014 00:00:00 GMT".to_owned()),
    );
    assert_eval(
        "new Date(0).toDateString()",
        Value::String("Thu Jan 01 1970".to_owned()),
    );
    assert_eval(
        "new Date(0).toTimeString()",
        Value::String("00:00:00 GMT+0000".to_owned()),
    );
    assert_true("/^(Sun|Mon|Tue)$/.test('Tue')");
    assert_true("/^[0-9]{2}$/.test('07')");
    assert_true("!(/^[0-9]{2}$/.test('7'))");
    assert_true("/^(Sun|Mon|Tue), [0-9]{2} (Jan|Feb|Mar) [0-9]{4}$/.test('Sun, 23 Mar 2014')");
    assert_true(
        "/^(Sun|Mon|Tue|Wed|Thu|Fri|Sat), [0-9]{2} (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT$/.test(new Date(0).toUTCString())",
    );
    assert_true(
        "/^[0-9]{2}:[0-9]{2}:[0-9]{2} GMT[+-][0-9]{4}( \\(.+\\))?$/.test(new Date(0).toTimeString())",
    );
    assert_eval(
        "var d = new Date(0); Date.parse(d.toString()) + Date.parse(d.toUTCString()) + Date.parse(d.toISOString());",
        Value::Number(0.0),
    );
    assert_eval(
        "Date.UTC(1970, 0, 1, 80063993375, 29, 1, -288230376151711740)",
        Value::Number(29312.0),
    );
    assert_eval(
        "Date.UTC(1970, 0, 213503982336, 0, 0, 0, -18446744073709552000)",
        Value::Number(34447360.0),
    );
    assert_true(
        "var date = new Date(2016, 6);
         date.setDate(2);
         date.setDate(null) === new Date(2016, 5, 30).getTime()",
    );
    assert_true(
        "var date = new Date(2016, 6, 7, 11, 36, 23, 2);
         date.setMonth(2) === new Date(2016, 2, 7, 11, 36, 23, 2).getTime() &&
         date.setMonth(null) === new Date(2016, 0, 7, 11, 36, 23, 2).getTime() &&
         date.setMonth(true) === new Date(2016, 1, 7, 11, 36, 23, 2).getTime() &&
         date.setMonth(false) === new Date(2016, 0, 7, 11, 36, 23, 2).getTime() &&
         date.setMonth('   +00200.000E-0002\t') === new Date(2016, 2, 7, 11, 36, 23, 2).getTime()",
    );
    assert_true(
        "var date = new Date(2016, 6, 7, 11, 36, 23, 2);
         date.setFullYear(2016, 2) === new Date(2016, 2, 7, 11, 36, 23, 2).getTime() &&
         date.setFullYear(2016, null) === new Date(2016, 0, 7, 11, 36, 23, 2).getTime() &&
         date.setFullYear(2016, true) === new Date(2016, 1, 7, 11, 36, 23, 2).getTime() &&
         date.setFullYear(2016, false) === new Date(2016, 0, 7, 11, 36, 23, 2).getTime() &&
         date.setFullYear(2016, '   +00200.000E-0002\t') === new Date(2016, 2, 7, 11, 36, 23, 2).getTime()",
    );
    assert_true(
        "var date = new Date(2016, 6, 7, 11, 36, 23, 2);
         var args, thisValue, callCount = 0;
         var arg = { valueOf: function () { args = arguments; thisValue = this; callCount += 1; return 2; } };
         date.setMonth(arg);
         callCount === 1 && args.length === 0 && thisValue === arg",
    );
    assert_true(
        "new Date(2016, 6, 7).setMonth(undefined) !== new Date(2016, 6, 7).setMonth(undefined)",
    );
    assert_true("new Date(2016, 6, 7).setFullYear(undefined) !== new Date(2016, 6, 7).setFullYear(undefined)");
    assert_true("new Date(2016, 6, 7).setFullYear(2016, undefined) !== new Date(2016, 6, 7).setFullYear(2016, undefined)");
}

#[test]
fn date_objects_are_ordinary_objects_for_existing_builtins() {
    assert_true("Array.isArray(new Date()) === false");
    assert_true(
        "var d = new Date(); d.answer = 42; d.answer === 42 && Object.keys(d)[0] === 'answer'",
    );
    assert_true(
        "var date = new Date(); [1].every(function (value, index, array) { return this === date && value === 1 && index === 0 && Array.isArray(array); }, date)",
    );
}

#[test]
fn native_error_constructors_inherit_from_error_constructor() {
    for name in [
        "TypeError",
        "RangeError",
        "ReferenceError",
        "SyntaxError",
        "EvalError",
        "URIError",
        "Test262Error",
        "AggregateError",
    ] {
        assert_true(&format!("Object.getPrototypeOf({name}) === Error"));
    }
}

#[test]
fn native_error_builtin_property_descriptors_are_installed() {
    for name in [
        "TypeError",
        "RangeError",
        "ReferenceError",
        "SyntaxError",
        "EvalError",
        "URIError",
        "Test262Error",
        "AggregateError",
    ] {
        assert_true(&format!(
            "Object.getOwnPropertyDescriptor({name}, 'prototype').writable === false"
        ));
        assert_true(&format!(
            "Object.getOwnPropertyDescriptor({name}.prototype, 'constructor').value === {name}"
        ));
        assert_true(&format!(
            "Object.getOwnPropertyDescriptor({name}.prototype, 'constructor').enumerable === false"
        ));
        assert_true(&format!(
            "Object.getOwnPropertyDescriptor({name}.prototype, 'name').value === '{name}'"
        ));
        assert_true(&format!(
            "Object.getOwnPropertyDescriptor({name}.prototype, 'message').value === ''"
        ));
    }
}

#[test]
fn error_is_error_uses_error_internal_slot() {
    assert_true("typeof Error.isError === 'function'");
    assert_true("Error.isError(new Error())");
    assert_true("Error.isError(new TypeError())");
    assert_true("Error.isError(new AggregateError([]))");
    assert_true("Error.isError({ name: 'Error', message: '' }) === false");
    assert_true("Error.isError(0n) === false");
    assert_true("Object.prototype.toString.call(new Error()) === '[object Error]'");
}

#[test]
fn block7_numeric_builtins_follow_ecmascript_integer_and_whitespace_edges() {
    assert_eval(
        "parseInt(String.fromCharCode(0x2028) + '1')",
        Value::Number(1.0),
    );
    assert_eval(
        "parseInt(String.fromCharCode(0x2029) + String.fromCharCode(0x2029) + '-1')",
        Value::Number(-1.0),
    );
    assert_true("var ls = String.fromCharCode(0x2028); parseInt(ls) !== parseInt(ls)");
    assert_eval("parseInt('11', Infinity)", Value::Number(11.0));
    assert_eval("parseInt('11', -Infinity)", Value::Number(11.0));
    assert_eval("parseInt('11', 4294967298)", Value::Number(3.0));
    assert_eval("Math.clz32(4294967296)", Value::Number(32.0));
    assert_eval("Math.clz32(4294967297)", Value::Number(31.0));
    assert_eval("Math.clz32(-4294967297)", Value::Number(0.0));
    assert_eval("Math.imul(4294967295, 5)", Value::Number(-5.0));
}

#[test]
fn block7_math_hypot_infinity_wins_after_all_argument_conversions() {
    assert_eval("Math.hypot(NaN, Infinity)", Value::Number(f64::INFINITY));
    assert_eval("Math.hypot(NaN, 1)", Value::Number(f64::NAN));
    assert_eval(
        "var calls = 0; var value = { valueOf: function () { calls += 1; return Infinity; } }; Math.hypot(NaN, value); calls;",
        Value::Number(1.0),
    );
}
