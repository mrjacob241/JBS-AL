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

fn assert_error_name(source: &str, expected: &str) {
    let wrapped = format!("try {{ {source}; 'no throw'; }} catch (error) {{ error.name; }}");
    assert_eval(&wrapped, Value::String(expected.to_owned()));
}

#[test]
fn block0_try_finally_and_check_sequence_harness_paths_run() {
    assert_eval(
        "var result = foo(); function foo() { return 7; } result;",
        Value::Number(7.0),
    );
    assert_eval(
        "function outer() { return inner(); function inner() { return 9; } } outer();",
        Value::Number(9.0),
    );
    assert_true("Foo.prototype.x = 1; function Foo() {} new Foo().x === 1");
    assert_eval(
        "var x = 0; try { x = 1; } finally { x = x + 2; } x;",
        Value::Number(3.0),
    );
    assert_eval(
        "var x = 0; try { throw 'stop'; } catch (e) { x = 4; } finally { x = x + 1; } x;",
        Value::Number(5.0),
    );
    assert_eval(
        "var x = 0; try { x = 1; return 9; } finally { x = 2; }",
        Value::Number(9.0),
    );
    assert_true("var sequence = [1, 2, 3]; checkSequence(sequence); true");
    assert_true("verifyPrimordialCallableProperty(parseInt, 'parseInt', 2); true");
    assert_true(
        "verifyPrimordialProperty(Array.prototype, 'every', { writable: true, enumerable: false, configurable: true }); true",
    );
    assert_eval("eval('1 + 2');", Value::Number(3.0));
    assert_eval("eval(42);", Value::Number(42.0));
    assert_true("verifyPrimordialCallableProperty(eval, 'eval', 1); true");
    assert_eval(
        "delete eval.length; eval.hasOwnProperty('length') === false && eval.length === Function.prototype.length;",
        Value::Boolean(true),
    );
    assert_eval("0..toString();", Value::String("0".to_owned()));
    assert_eval("var x = 1; x += 4; x;", Value::Number(5.0));
    assert_eval("var x = 8; x >>= 1; x;", Value::Number(4.0));
    assert_eval("1 << 5;", Value::Number(32.0));
    assert_eval("var µ = 7; µ;", Value::Number(7.0));
    assert_eval("var [a, b] = [1, 2]; a + b;", Value::Number(3.0));
    assert_eval("var [, b] = [1, 4]; b;", Value::Number(4.0));
    assert_eval("var o = { a: 1, ...{ b: 2 } }; o.b;", Value::Number(2.0));
    assert_eval(
        "var s = Symbol(); var o = { ...{ a: 1 }, [s]: 3 }; o.a + o[s];",
        Value::Number(4.0),
    );
    assert_eval(
        "var o = { index: 0, get val() { this.index++; return 1 << this.index; } }; o.val;",
        Value::Number(2.0),
    );
    assert_eval(
        "var o = { set val(v) { this.saved = v + 1; } }; o.val = 4; o.saved;",
        Value::Number(5.0),
    );
}

#[test]
fn block2_descriptor_targets_are_strict_but_property_bags_are_to_object() {
    assert_eval(
        "try { Object.defineProperty(true, 'x', { value: 1 }); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "try { Object.defineProperties(false, {}); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var proto = {}; var obj = Object.create(proto, true); Object.getPrototypeOf(obj) === proto && Object.getOwnPropertySymbols(obj).length === 0;",
        Value::Boolean(true),
    );
    assert_eval(
        "var obj = {}; Object.defineProperty(obj, 'foo', { set: function (v) { obj.seen = v; }, configurable: true }); verifyWritable(obj, 'foo', 'seen'); obj.seen;",
        Value::Number(8675309.0),
    );
    assert_eval(
        "var o = {}; Object.defineProperty(o, 'x', { get: function () { return 1; }, configurable: false }); try { Object.defineProperty(o, 'x', { get: function () { return 2; } }); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_true(
        "var getter = function () { return 1; }; var o = {}; Object.defineProperty(o, 'x', { get: getter }); var d = Object.getOwnPropertyDescriptor(o, 'x'); d.get === getter && d.set === undefined && d.enumerable === false && d.configurable === false",
    );
    assert_eval("Object.preventExtensions(1);", Value::Number(1.0));
    assert_eval("Object.freeze('x');", Value::String("x".to_owned()));
    assert_eval("Object.seal(true);", Value::Boolean(true));
    assert_true(
        "var o = {}; var getter = function () { return 7; }; o.__defineGetter__('x', getter); o.x === 7 && o.__lookupGetter__('x') === getter",
    );
    assert_true(
        "var o = {}; var setter = function (v) { this.y = v; }; o.__defineSetter__('x', setter); o.__lookupSetter__('x') === setter",
    );
    assert_true(
        "var parent = {}; var getter = function () { return 3; }; parent.__defineGetter__('x', getter); var child = Object.create(parent); child.__lookupGetter__('x') === getter",
    );
    assert_true(
        "var get = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').get; get.call({}) === Object.prototype",
    );
    assert_true(
        "var set = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').set; var proto = {}; var subject = {}; set.call(subject, proto) === undefined && Object.getPrototypeOf(subject) === proto",
    );
    assert_eval(
        "var groups = Object.groupBy([1, 2, 3, 4], function (v) { return v % 2 ? 'odd' : 'even'; }); groups.odd.length + ':' + groups.even.length;",
        Value::String("2:2".to_owned()),
    );
    assert_eval(
        "Object.getPrototypeOf(Object.groupBy([], function () { return 'x'; })) === null;",
        Value::Boolean(true),
    );
    assert_eval(
        "var s = Symbol(); var groups = Object.groupBy([1], function () { return s; }); groups[s][0];",
        Value::Number(1.0),
    );
}

#[test]
fn block7_math_and_global_numeric_functions_are_installed_as_globals() {
    assert_true("Array.isArray(Math) === false");
    assert_eval("BigInt(3) + BigInt(4);", Value::BigInt(7));
    assert_eval("typeof BigInt(1);", Value::String("bigint".to_owned()));
    assert_eval("BigInt('')", Value::BigInt(0));
    assert_eval("BigInt('0x10')", Value::BigInt(16));
    assert_eval("BigInt('0o10')", Value::BigInt(8));
    assert_eval("BigInt('0b10')", Value::BigInt(2));
    assert_eval("BigInt.asIntN(2, 3n)", Value::BigInt(-1));
    assert_eval("BigInt.asUintN(2, -1n)", Value::BigInt(3));
    assert_eval("-1n", Value::BigInt(-1));
    assert_error_name("BigInt.asIntN(-1, 1n)", "RangeError");
    assert_error_name("BigInt.asIntN(1, 1)", "TypeError");
    assert_true(
        "var i = 0;
         var bits = { valueOf: function () { return ++i; } };
         var value = { valueOf: function () { i = i * 10; return 3n; } };
         BigInt.asIntN(bits, value) === -1n && i === 10",
    );
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(BigInt, 'asIntN');
         typeof BigInt.asIntN === 'function' && BigInt.asIntN.length === 2 &&
         BigInt.asIntN.name === 'asIntN' &&
         d.writable === true && d.enumerable === false && d.configurable === true",
    );
    assert_true(
        "var once = 0;
         var value = { [Symbol.toPrimitive]: function () { once++; return '42'; } };
         BigInt(value) === 42n && once === 1",
    );
    assert_true(
        "BigInt.prototype.constructor === BigInt &&
         Object.prototype.toString.call(BigInt.prototype) === '[object BigInt]'",
    );
    assert_eval(
        "BigInt.prototype.valueOf.call(Object(3n))",
        Value::BigInt(3),
    );
    assert_eval("(255n).toString(16)", Value::String("ff".to_owned()));
    assert_eval("(-10n).toString()", Value::String("-10".to_owned()));
    assert_eval("Math.max(1, 4, 2);", Value::Number(4.0));
    assert_eval("Math.min(1, -4, 2);", Value::Number(-4.0));
    assert_eval("Math.pow(2, 5);", Value::Number(32.0));
    assert_eval("Math.abs(-1e-17);", Value::Number(1e-17));
    assert_eval("Math.trunc(3.9);", Value::Number(3.0));
    assert_eval("Math.clz32(1);", Value::Number(31.0));
    assert_eval("Math.imul(2, 4);", Value::Number(8.0));
    assert_eval("Math.f16round(1.337);", Value::Number(1.3369140625));
    assert_true("Math.random() >= 0 && Math.random() < 1");
    assert_eval(
        "Math.sumPrecise([0.1, 0.2]);",
        Value::Number(0.30000000000000004),
    );
    assert_eval("parseInt('0x10');", Value::Number(16.0));
    assert_eval("0x10 + 0b10 + 0o7;", Value::Number(25.0));
    assert_eval(
        "var obj = { 2574: 0x000A0E, length: '0x000A0E' }; obj[2574];",
        Value::Number(2574.0),
    );
    assert_true("0n === BigInt(0) && 0x10n === BigInt(16)");
    assert_eval("parseInt('432', 10);", Value::Number(432.0));
    assert_eval("parseFloat('3.5px');", Value::Number(3.5));
    assert_eval("parseFloat(.01e+2);", Value::Number(1.0));
    assert_eval("parseFloat('\\u00091.1');", Value::Number(1.1));
    assert_true("isNaN(NaN)");
    assert_true("isFinite(42)");
    assert_true("Number.isFinite(42) && !Number.isFinite('42')");
    assert_true("Number.isInteger(42) && !Number.isInteger(42.5)");
    assert_true("Number.isNaN(NaN) && !Number.isNaN('NaN')");
    assert_true(
        "Number.isSafeInteger(9007199254740991) && !Number.isSafeInteger(9007199254740992)",
    );
    assert_true("Number.parseInt === parseInt && Number.parseFloat === parseFloat");
    assert_eval("Number('0b101');", Value::Number(5.0));
    assert_eval("Number('0o77');", Value::Number(63.0));
    assert_eval("Number('0x10');", Value::Number(16.0));
    assert_eval("(12.345).toFixed(2);", Value::String("12.35".to_owned()));
    assert_eval(
        "(12.345).toExponential(2);",
        Value::String("1.23e1".to_owned()),
    );
    assert_eval(
        "(12.345).toPrecision(3);",
        Value::String("1.23e1".to_owned()),
    );
    assert_eval("decodeURIComponent('%41');", Value::String("A".to_owned()));
    assert_eval(
        "encodeURIComponent('a b');",
        Value::String("a%20b".to_owned()),
    );
    assert_eval(
        "encodeURI('https://x.test/a b');",
        Value::String("https://x.test/a%20b".to_owned()),
    );
}

#[test]
fn block9_generic_array_callbacks_skip_missing_indexes() {
    assert_eval(
        "var seen = 0; var a = [1,,3]; a.forEach(function () { seen = seen + 1; }); seen;",
        Value::Number(2.0),
    );
    assert_eval(
        "var seen = 0; var a = [1,,3]; a.map(function (value) { seen = seen + 1; return value + 1; }); seen;",
        Value::Number(2.0),
    );
    assert_eval(
        "var seen = 0; var a = [1,,3]; a.filter(function () { seen = seen + 1; return true; }).length; seen;",
        Value::Number(2.0),
    );
    assert_eval(
        "var a = [, 2, 3]; a.reduce(function (acc, value) { return acc + value; });",
        Value::Number(5.0),
    );
    assert_eval(
        "var calls = 0; var obj = Object.defineProperty({}, 'length', { get: function () { return Math.pow(2, 32); }, set: function () { calls += 1; } }); try { Array.prototype.slice.call(obj); 'bad'; } catch (e) { e.name + ':' + calls; }",
        Value::String("RangeError:0".to_owned()),
    );
    assert_true(
        "var C = function () {}; var items = []; var result = Array.from.call(C, items); result instanceof C",
    );
}

#[test]
fn block8_string_trim_basic() {
    assert_eval("'  ok  '.trim();", Value::String("ok".to_owned()));
    assert_eval(
        "String({ toString: function () { return 'obj'; } });",
        Value::String("obj".to_owned()),
    );
    assert_eval("String(1 / 'a');", Value::String("NaN".to_owned()));
    assert_eval("'b' * null;", Value::Number(f64::NAN));
    assert_eval("+'123\\u180E';", Value::Number(f64::NAN));
    assert_true("'a' < 'b'");
    assert_true("'10' < '2'");
    assert_true("'💩' < '🙏'");
    assert_true("!('🥰' < '🙏')");
    assert_eval(
        "var groups = Object.groupBy('🥰💩🙏😈', function (char) { return char < '🙏' ? 'before' : 'after'; }); groups.before.join('') + ':' + groups.after.join('');",
        Value::String("💩😈:🥰🙏".to_owned()),
    );
}

#[test]
fn block8_string_wrapper_common_methods() {
    assert_eval("'abc'.charAt(1);", Value::String("b".to_owned()));
    assert_eval("'abc'.charCodeAt(1);", Value::Number(98.0));
    assert_eval("'abc'.charCodeAt(9);", Value::Number(f64::NAN));
    assert_eval("'😀'.codePointAt(0);", Value::Number(128512.0));
    assert_eval("'ab'.concat('c', 4);", Value::String("abc4".to_owned()));
    assert_eval("'abcabc'.indexOf('bc', 2);", Value::Number(4.0));
    assert_eval("'abcabc'.lastIndexOf('bc', 3);", Value::Number(1.0));
    assert_eval("'abcdef'.slice(1, -1);", Value::String("bcde".to_owned()));
    assert_eval("'abcdef'.substring(4, 1);", Value::String("bcd".to_owned()));
    assert_eval("'abcdef'.includes('cd');", Value::Boolean(true));
    assert_eval("'abcdef'.startsWith('bc', 1);", Value::Boolean(true));
    assert_eval("'abcdef'.endsWith('de', 5);", Value::Boolean(true));
    assert_eval(
        "'Thu, 01 Jan 1970 00:00:00 GMT'.split(' ')[3];",
        Value::String("1970".to_owned()),
    );
    assert_eval("' AbC '.trimStart();", Value::String("AbC ".to_owned()));
    assert_eval("' AbC '.trimEnd();", Value::String(" AbC".to_owned()));
    assert_eval("'AbC'.toLowerCase();", Value::String("abc".to_owned()));
    assert_eval("'AbC'.toUpperCase();", Value::String("ABC".to_owned()));
    assert_eval("'abc'.at(-1);", Value::String("c".to_owned()));
    assert_eval("'ab'.padStart(5, '0');", Value::String("000ab".to_owned()));
    assert_eval("'ab'.padEnd(5, '01');", Value::String("ab010".to_owned()));
    assert_eval("'ab'.repeat(3);", Value::String("ababab".to_owned()));
    assert_eval("'a'.localeCompare('b');", Value::Number(-1.0));
    assert_eval("'abc'.normalize();", Value::String("abc".to_owned()));
    assert_eval("'abc'.isWellFormed();", Value::Boolean(true));
    assert_eval("'abc'.toWellFormed();", Value::String("abc".to_owned()));
    assert_eval("'﻿ ok  '.trim();", Value::String("ok".to_owned()));
    assert_eval(
        "(new String('boxed')).slice(1, 4);",
        Value::String("oxe".to_owned()),
    );
    assert_eval(
        "String.prototype.trim.call(42);",
        Value::String("42".to_owned()),
    );
    assert_eval(
        "String.prototype.indexOf.call(true, 'r');",
        Value::Number(1.0),
    );
    assert_eval(
        "String.prototype.slice.call({ toString: function () { return 'object'; } }, 1, 4);",
        Value::String("bje".to_owned()),
    );
}

#[test]
fn block8_string_statics_common_paths() {
    assert_eval(
        "String.fromCharCode(65, 66, 67);",
        Value::String("ABC".to_owned()),
    );
    assert_eval("String.fromCodePoint(9731);", Value::String("☃".to_owned()));
    assert_eval(
        "String.raw({ raw: ['a', 'c'], length: 2 }, 'b');",
        Value::String("abc".to_owned()),
    );
}

#[test]
fn block4_global_readonly_undefined_assignment_is_ignored() {
    assert_eval("undefined = 1; undefined;", Value::Undefined);
}

#[test]
fn block3_object_and_symbol_basics() {
    assert_eval(
        "typeof Symbol.isConcatSpreadable;",
        Value::String("symbol".to_owned()),
    );
    assert_eval(
        "var s = Symbol(); var source = {}; source[s] = 3; var target = Object.assign({}, source); target[s];",
        Value::Number(3.0),
    );
    assert_eval(
        "Object.keys(Object.fromEntries([])).length;",
        Value::Number(0.0),
    );
    assert_eval(
        "var s = Symbol(); var o = Object.fromEntries([[s, 9], ['x', 2]]); o[s] + o.x;",
        Value::Number(11.0),
    );
}

#[test]
fn block5_bound_constructor_path() {
    assert_true("function C(v) { this.v = v; } var B = C.bind(null, 9); new B() instanceof C");
    assert_true(
        "function f(a, b, c) {}
         var b = f.bind(null, 1);
         var d = Object.getOwnPropertyDescriptor(b, 'length');
         b.length === 2 && d.writable === false && d.enumerable === false && d.configurable === true",
    );
    assert_true(
        "function f() {}
         Object.defineProperty(f, 'length', { value: 3.66 });
         f.bind(null).length === 3 && f.bind(null, 1).length === 2",
    );
    assert_true(
        "function f() {}
         Object.defineProperty(f, 'length', { value: Infinity });
         f.bind(null, 1).length === Infinity",
    );
    assert_true(
        "function f() {}
         Object.defineProperty(f, 'length', { value: '3' });
         f.bind(null).length === 0",
    );
    assert_true(
        "function f() {}
         Object.setPrototypeOf(f, { length: 9 });
         delete f.length;
         Function.prototype.bind.call(f, null).length === 0",
    );
    assert_true(
        "var target = Object.defineProperty(function () {}, 'name', { value: 'target' });
         var b = target.bind(null).bind(null);
         var d = Object.getOwnPropertyDescriptor(b, 'name');
         b.name === 'bound bound target' && d.writable === false && d.enumerable === false && d.configurable === true",
    );
    assert_true(
        "var target = Object.defineProperty(function () {}, 'name', { value: 23 });
         target.bind(null).name === 'bound '",
    );
    assert_eval(
        "var target = Object.defineProperty(function () {}, 'name', { get: function () { throw new TypeError('name'); } });
         try { target.bind(null); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "Object.prototype.toString.call(Function.prototype);",
        Value::String("[object Function]".to_owned()),
    );
    assert_eval(
        "Function('a', 'b', 'return a + b;')(2, 3);",
        Value::Number(5.0),
    );
    assert_true("var f = Function('return this;'); f() === globalThis");
    assert_eval(
        "Function('this.field = \"strawberry\"').call(); globalThis.field;",
        Value::String("strawberry".to_owned()),
    );
    assert_eval("void 0;", Value::Undefined);
    assert_eval("var side = 0; void (side = 5); side;", Value::Number(5.0));
    assert_eval(
        "Function('this.field = \"battle\"').call(void 0); globalThis.field;",
        Value::String("battle".to_owned()),
    );
    assert_eval(
        "Function('a1', 'this.shifted = a1;').apply(null, [[1]]); globalThis.shifted[0];",
        Value::Number(1.0),
    );
    assert_eval(
        "Function('return Object.prototype.toString.call(this);').call(7);",
        Value::String("[object Number]".to_owned()),
    );
    assert_eval("Number.bind(null)(42);", Value::Number(42.0));
    assert_eval(
        "String.bind({ ignored: true })(123);",
        Value::String("123".to_owned()),
    );
    assert_eval("Boolean.bind(null)(0);", Value::Boolean(false));
    assert_true(
        "var n = new (Number.bind(null, 5))();
         Object.prototype.toString.call(n) === '[object Number]' && n.valueOf() === 5",
    );
    assert_eval("Number.call({ nope: true }, '7');", Value::Number(7.0));
    assert_eval(
        "String.call({ nope: true }, true);",
        Value::String("true".to_owned()),
    );
    assert_eval("Boolean.call({ nope: true }, 'x');", Value::Boolean(true));
    assert_eval(
        "typeof BigInt.call({ nope: true }, 3);",
        Value::String("bigint".to_owned()),
    );
    assert_eval(
        "typeof Symbol.call({ nope: true }, 's');",
        Value::String("symbol".to_owned()),
    );
    assert_eval(
        "try { Map.call({}, []); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
}

#[test]
fn block6_object_length_to_primitive() {
    assert_true(
        "var o = { 0: 1, length: { valueOf: function () { return 1; } } }; Array.prototype.every.call(o, function (v) { return v === 1; })",
    );
    assert_eval(
        "'value:' + { toString: function () { return 'ok'; } };",
        Value::String("value:ok".to_owned()),
    );
    assert_eval(
        "({ valueOf: function () { return 4; } }) + 6;",
        Value::Number(10.0),
    );
}

#[test]
fn block9_array_flat_basic() {
    assert_eval("[1, [2, 3]].flat()[2];", Value::Number(3.0));
    assert_eval("[1, [2, [3]]].flat(2)[2];", Value::Number(3.0));
    assert_eval("var a = [1,,[2,,3]]; a.flat().length;", Value::Number(3.0));
}
