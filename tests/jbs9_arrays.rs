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
fn array_exotic_index_keys_update_length_only_for_canonical_array_indices() {
    assert_eval("var a = []; a[0] = 'first'; a.length;", Value::Number(1.0));
    assert_eval("var a = []; a[3] = 'far'; a.length;", Value::Number(4.0));
    assert_eval(
        "var a = []; a['01'] = 'not an index'; a.length;",
        Value::Number(0.0),
    );
    assert_eval(
        "var a = []; a[4294967294] = 'last'; a.length;",
        Value::Number(4294967295.0),
    );
    assert_eval(
        "var a = []; a[4294967295] = 'plain'; a.length;",
        Value::Number(0.0),
    );
    assert_eval(
        "var a = []; a[4294967295] = 'plain'; a[4294967295];",
        Value::String("plain".to_owned()),
    );
}

#[test]
fn array_constructor_validates_single_numeric_length_argument() {
    assert_eval("Array(0).length;", Value::Number(0.0));
    assert_eval("Array(3).length;", Value::Number(3.0));
    assert_eval("Array('3').length;", Value::Number(1.0));
    assert_eval("Array('3')[0];", Value::String("3".to_owned()));
    assert_error_name("Array(-1)", "RangeError");
    assert_error_name("Array(1.5)", "RangeError");
    assert_error_name("Array(4294967296)", "RangeError");
}

#[test]
fn length_assignment_validates_uint32_and_truncates_index_properties() {
    assert_eval(
        "var a = [1, 2, 3]; a.length = 1; a.length;",
        Value::Number(1.0),
    );
    assert_true("var a = [1, 2, 3]; a.length = 1; !(1 in a) && !(2 in a)");
    assert_eval(
        "var a = [1, 2, 3]; a.extra = 9; a.length = 1; a.extra;",
        Value::Number(9.0),
    );
    assert_error_name("var a = []; a.length = -1", "RangeError");
    assert_error_name("var a = []; a.length = 1.5", "RangeError");
    assert_error_name("var a = []; a.length = 4294967296", "RangeError");
}

#[test]
fn length_truncation_deletes_sparse_high_indexes_without_dense_scan() {
    assert_true(
        "var a = [0, 1, 2];
         a[4294967294] = 9;
         a.length = 2;
         a.length === 2 && a[0] === 0 && a[1] === 1 &&
         !(2 in a) && !(4294967294 in a)",
    );
}

#[test]
fn length_definition_rolls_back_when_truncation_hits_non_configurable_index() {
    assert_true(
        "var a = [0, 1];
         Object.defineProperty(a, '1', { configurable: false });
         try { Object.defineProperty(a, 'length', { value: 1 }); } catch (e) {}
         a.length === 2 && a[1] === 1",
    );
    assert_true(
        "var a = [0, 1, 2];
         Object.defineProperty(a, '2', { configurable: false });
         try { Object.defineProperties(a, { length: { value: 1 } }); } catch (e) {}
         a.length === 3 && a[2] === 2",
    );
}

#[test]
fn array_length_definition_coerces_value_before_writable_validation() {
    assert_true(
        "var a = [1, 2];
         var calls = 0;
         var length = {
           valueOf: function () {
             calls = calls + 1;
             if (calls !== 1) {
               Object.defineProperty(a, 'length', { writable: false });
             }
             return a.length;
           }
         };
         try { Object.defineProperty(a, 'length', { value: length, writable: true }); } catch (e) {}
         calls === 2 && a.length === 2",
    );
}

#[test]
fn length_writability_controls_extension_and_truncation() {
    assert_true(
        "var a = [1, 2]; Object.defineProperty(a, 'length', { writable: false }); a[2] = 3; a.length === 2 && !(2 in a)",
    );
    assert_true(
        "var a = [1, 2]; Object.defineProperty(a, 'length', { writable: false }); a.length = 1; a.length === 2 && (1 in a)",
    );
}

#[test]
fn array_from_and_of_currently_feasible_rows_remain_aligned() {
    assert_eval("Array.from([2, 4, 6])[2];", Value::Number(6.0));
    assert_eval("Array.from('abc')[1];", Value::String("b".to_owned()));
    assert_eval(
        "Array.from({0: 'x', 1: 'y', length: 2})[1];",
        Value::String("y".to_owned()),
    );
    assert_eval(
        "Array.from([1, 2], function (value, index) { return value + index; })[1];",
        Value::Number(3.0),
    );
    assert_eval("Array.of().length;", Value::Number(0.0));
    assert_eval("Array.of('a', 'b')[1];", Value::String("b".to_owned()));
}

#[test]
fn generic_array_methods_use_indexed_length_behavior_on_plain_objects() {
    assert_eval(
        "var o = { length: 0 }; Array.prototype.push.call(o, 'a'); o[0];",
        Value::String("a".to_owned()),
    );
    assert_eval(
        "var o = { length: 1, 0: 'a' }; Array.prototype.push.call(o, 'b'); o.length;",
        Value::Number(2.0),
    );
    assert_eval(
        "var o = { length: 2, 0: 'a', 1: 'b' }; Array.prototype.pop.call(o);",
        Value::String("b".to_owned()),
    );
    assert_true(
        "var o = { length: 2, 0: 'a', 1: 'b' }; Array.prototype.pop.call(o); o.length === 1 && !(1 in o)",
    );
    assert_eval(
        "var o = { length: 3, 0: 'a', 2: 'c' }; Array.prototype.slice.call(o, 1).length;",
        Value::Number(2.0),
    );
    assert_true(
        "var o = { length: 5, 2: 'c', 3: 'd', 4: 'e' };
         var r = Array.prototype.slice.call(o, 2, undefined);
         r.length === 3 && r[0] === 'c' && r[2] === 'e'",
    );
    assert_true(
        "var o = { length: 2, 1: 'x' };
         var r = Array.prototype.slice.call(o, 0);
         r.length === 2 && !(0 in r) && r[1] === 'x'",
    );
    assert_true(
        "var instance = [];
         var calls = 0;
         function C(length) { calls++; this.seenLength = length; return instance; }
         var a = [1, 2, 3, 4];
         a.constructor = {};
         a.constructor[Symbol.species] = C;
         var r = a.slice(1, -1);
         r === instance && calls === 1 && instance[0] === 2 && instance[1] === 3",
    );
    assert_true(
        "var a = [1];
         a.constructor = {};
         a.constructor[Symbol.species] = undefined;
         var r = a.slice();
         Array.isArray(r) && Object.getPrototypeOf(r) === Array.prototype && r[0] === 1",
    );
    assert_true(
        "var o = { length: 9007199254740993 };
         o[9007199254740989] = 'a';
         o[9007199254740990] = 'b';
         var r = Array.prototype.slice.call(o, 9007199254740989);
         r.length === 2 && r[0] === 'a' && r[1] === 'b'",
    );
}

#[test]
fn generic_array_scan_and_search_methods_are_array_like() {
    assert_true("[2, 4, 6].every(function (value) { return value % 2 === 0; })");
    assert_true("[1, 3, 4].some(function (value) { return value % 2 === 0; })");
    assert_true(
        "Math.length = 1; Math[0] = 1; Array.prototype.every.call(Math, function (value, index, object) { return Object.prototype.toString.call(object) !== '[object Math]'; }) === false",
    );
    assert_true(
        "var lengthAccessed = false; var loopAccessed = false; var obj = {};
         Object.defineProperty(obj, 'length', { get: function () { lengthAccessed = true; return 20; } });
         Object.defineProperty(obj, '0', { get: function () { loopAccessed = true; return 10; } });
         try { Array.prototype.every.call(obj); } catch (error) {}
         lengthAccessed && !loopAccessed",
    );
    assert_eval("['a', 'b', 'a'].indexOf('a', 1);", Value::Number(2.0));
    assert_eval("['a', 'b', 'a'].lastIndexOf('a');", Value::Number(2.0));
    assert_eval("[true].lastIndexOf(true, -Infinity);", Value::Number(-1.0));
    assert_eval(
        "[true].lastIndexOf(true, '-Infinity');",
        Value::Number(-1.0),
    );
    assert_eval(
        "var o = { length: 3, 0: 'x', 2: 'z' }; Array.prototype.indexOf.call(o, 'z');",
        Value::Number(2.0),
    );
    assert_true(
        "var stepTwo = false;
         var stepFive = false;
         var obj = {};
         Object.defineProperty(obj, 'length', { get: function () { stepTwo = true; if (stepFive) { throw new Error('bad'); } return 20; } });
         var fromIndex = { valueOf: function () { stepFive = true; return 0; } };
         Array.prototype.indexOf.call(obj, undefined, fromIndex);
         stepTwo && stepFive",
    );
    assert_true(
        "var stepTwo = false;
         var stepFive = false;
         var obj = {};
         Object.defineProperty(obj, 'length', { get: function () { stepTwo = true; if (stepFive) { throw new Error('bad'); } return 20; } });
         var fromIndex = { valueOf: function () { stepFive = true; return 0; } };
         Array.prototype.lastIndexOf.call(obj, undefined, fromIndex);
         stepTwo && stepFive",
    );
    assert_true(
        "var seen = 0; var o = { length: 3, 1: 4 }; Array.prototype.every.call(o, function (value) { seen = seen + 1; return value === 4; }) && seen === 1",
    );
}

#[test]
fn generic_array_mutation_and_find_methods_are_array_like() {
    assert_eval(
        "var obj = {0: 'a', 1: 'b', 2: 'c', length: 3};
         Array.prototype.copyWithin.call(obj, 1, 0, 2);
         obj[0] + obj[1] + obj[2];",
        Value::String("aab".to_owned()),
    );
    assert_eval(
        "var obj = {0: 'a', 2: 'c', length: 3};
         Array.prototype.fill.call(obj, 'x', 1);
         obj[0] + obj[1] + obj[2];",
        Value::String("axx".to_owned()),
    );
    assert_eval(
        "[1, 2, 3].find(function (value) { return value > 1; });",
        Value::Number(2.0),
    );
    assert_eval(
        "[1, 2, 3].findIndex(function (value) { return value > 1; });",
        Value::Number(1.0),
    );
    assert_eval(
        "[1, 2, 3].findLast(function (value) { return value < 3; });",
        Value::Number(2.0),
    );
    assert_eval(
        "[1, 2, 3].findLastIndex(function (value) { return value < 3; });",
        Value::Number(1.0),
    );
    assert_eval("[0, 1, 2, 3].copyWithin({}, 1)[0];", Value::Number(1.0));
    assert_eval(
        "var arr = ['Shoes', 'Car', 'Bike'];
         var out = '';
         arr.find(function (value) {
           if (out === '') { arr.splice(1, 1); }
           out = out + value;
           return false;
         });
         out;",
        Value::String("ShoesBikeundefined".to_owned()),
    );
}

#[test]
fn flat_uses_species_and_throws_when_result_creation_fails() {
    assert_eval(
        "var a = []; a.constructor = null; try { a.flat(); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var A = function () { this.length = 0; Object.preventExtensions(this); };
         var arr = [1]; arr.constructor = {}; arr.constructor[Symbol.species] = A;
         try { arr.flat(1); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var A = function () { Object.defineProperty(this, '0', { set: function () {}, configurable: false }); };
         var arr = [[1]]; arr.constructor = {}; arr.constructor[Symbol.species] = A;
         try { arr.flat(1); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_true(
        "function C(len) { this.seenNewTarget = new.target === C; this.seenLength = len; }
         var a = [[42, 1], [42, 2]];
         var calls = 0;
         a.constructor = {};
         Object.defineProperty(a.constructor, Symbol.species, { get: function () { calls++; return C; } });
         var r = a.flatMap(function (entry) { return entry; });
         r instanceof C && r.seenNewTarget && r.seenLength === 0 && calls === 1 && r[0] === 42 && r[3] === 2",
    );
}

#[test]
fn array_like_lengths_use_shared_to_primitive_protocols() {
    assert_eval(
        "Symbol.toPrimitive === Symbol.toPrimitive;",
        Value::Boolean(true),
    );
    assert_eval(
        "var calls = 0;
         var source = { 0: 1, length: { [Symbol.toPrimitive]: function (hint) { calls++; if (hint === 'number') { throw new TypeError('length'); } return 1; } } };
         try { Array.prototype.flatMap.call(source, function (value) { return value; }); 'no'; } catch (error) { error.name + ':' + calls; }",
        Value::String("TypeError:1".to_owned()),
    );
}

#[test]
fn splice_handles_sparse_array_like_integer_limit_edges() {
    assert_eval(
        "var o = { length: 2 ** 53 + 2 };
         Array.prototype.splice.call(o);
         o.length;",
        Value::Number(9_007_199_254_740_991.0),
    );
    assert_error_name(
        "var o = { length: 2 ** 32 }; Array.prototype.splice.call(o, 0)",
        "RangeError",
    );
    assert_error_name(
        "var o = { length: 2 ** 53 - 1 }; Array.prototype.splice.call(o, 0, 0, null)",
        "TypeError",
    );
    assert_true(
        "var o = {};
         o[0] = 'x';
         o[4294967295] = 'y';
         o.length = 4294967296;
         var deleted = Array.prototype.splice.call(o, 4294967295, 1);
         deleted.length === 1 && deleted[0] === 'y' &&
         o.length === 4294967295 && o[0] === 'x' && !(4294967295 in o)",
    );
    assert_true(
        "var o = {
           '9007199254740985': 'a',
           '9007199254740986': 'b',
           '9007199254740987': 'c',
           '9007199254740989': 'd',
           length: 2 ** 53 - 2
         };
         var deleted = Array.prototype.splice.call(o, 9007199254740986, 0, 'new');
         deleted.length === 0 && o.length === 2 ** 53 - 1 &&
         o['9007199254740986'] === 'new' &&
         o['9007199254740987'] === 'b' &&
         o['9007199254740988'] === 'c' &&
         !('9007199254740989' in o) &&
         o['9007199254740990'] === 'd'",
    );
    assert_true(
        "var o = {
           '9007199254740986': 'a',
           '9007199254740987': 'b',
           '9007199254740988': 'c',
           '9007199254740990': 'd',
           '9007199254740991': 'e',
           length: 2 ** 53 + 2
         };
         var deleted = Array.prototype.splice.call(o, 9007199254740987, 1);
         deleted.length === 1 && deleted[0] === 'b' &&
         o.length === 2 ** 53 - 2 &&
         o['9007199254740987'] === 'c' &&
         !('9007199254740988' in o) &&
         o['9007199254740989'] === 'd' &&
         !('9007199254740990' in o) &&
         o['9007199254740991'] === 'e'",
    );
    assert_true(
        "var instance = [];
         var calls = 0;
         function C(length) { calls++; this.seenLength = length; return instance; }
         var a = [1, 2, 3, 4];
         a.constructor = {};
         a.constructor[Symbol.species] = C;
         var r = a.splice(1, 2);
         r === instance && calls === 1 && instance[0] === 2 && instance[1] === 3 && a.length === 2",
    );
    assert_error_name(
        "function C(length) { Object.preventExtensions(this); }
         var a = [1];
         a.constructor = {};
         a.constructor[Symbol.species] = C;
         a.splice(0)",
        "TypeError",
    );
}

#[test]
fn array_mutators_throw_when_required_set_or_delete_fails() {
    assert_error_name(
        "var o = { length: 43 };
         Object.defineProperty(o, '42', { configurable: false, writable: true });
         Array.prototype.copyWithin.call(o, 42, 0)",
        "TypeError",
    );
    assert_error_name(
        "var o = { length: 1 };
         Object.defineProperty(o, '0', { value: 1, writable: false, configurable: true });
         Array.prototype.fill.call(o, 2)",
        "TypeError",
    );
    assert_error_name(
        "var o = { length: 1 };
         Object.preventExtensions(o);
         Array.prototype.splice.call(o, 0, 0, 'x')",
        "TypeError",
    );
    assert_error_name(
        "var a = [1];
         Object.defineProperty(a, '0', { configurable: false });
         Array.prototype.pop.call(a)",
        "TypeError",
    );
    assert_error_name(
        "var a = [];
         Object.defineProperty(a, 'length', { writable: false });
         Array.prototype.push.call(a, 1)",
        "TypeError",
    );
}

#[test]
fn reverse_rechecks_presence_after_side_effecting_getters() {
    assert_true(
        "var array = ['first', 'second'];
         Object.defineProperty(array, '0', {
           get: function () {
             array.length = 0;
             return 'first';
           },
           configurable: true
         });
         array.reverse();
         !(0 in array) && (1 in array) && array[1] === 'first'",
    );
}

#[test]
fn copy_by_value_array_methods_materialize_holes() {
    assert_true(
        "var a = [0,,2];
         var r = a.toReversed();
         r.length === 3 && r[0] === 2 && r.hasOwnProperty(1) && r[1] === undefined && r[2] === 0",
    );
    assert_true(
        "var a = [3,,1];
         var r = a.toSorted();
         r.length === 3 && r[0] === 1 && r[1] === 3 && r.hasOwnProperty(2) && r[2] === undefined",
    );
    assert_true(
        "var a = [0,,2];
         var r = a.with(2, 6);
         r.length === 3 && r[0] === 0 && r.hasOwnProperty(1) && r[1] === undefined && r[2] === 6",
    );
    assert_true(
        "var a = [0,,2];
         var r = a.toSpliced(0, 0, -1);
         r.length === 4 && r[0] === -1 && r[1] === 0 && r.hasOwnProperty(2) && r[2] === undefined && r[3] === 2",
    );
    assert_true(
        "var o = { length: 9007199254740993 };
         o[9007199254740989] = 'a';
         o[9007199254740990] = 'b';
         var r = Array.prototype.toSpliced.call(o, 0, 9007199254740989);
         r.length === 2 && r[0] === 'a' && r[1] === 'b'",
    );
}

#[test]
fn array_to_locale_string_invokes_element_methods() {
    assert_true(
        "var d = Object.getOwnPropertyDescriptor(Array.prototype, 'toLocaleString');
         typeof Array.prototype.toLocaleString === 'function' &&
         d.writable === true && d.enumerable === false && d.configurable === true",
    );
    assert_eval(
        "[true, false].toLocaleString();",
        Value::String("true,false".to_owned()),
    );
    assert_true(
        "var calls = 0;
         var obj = { toLocaleString: function () { calls++; return 'x'; } };
         [undefined, obj, null, obj, obj].toLocaleString() === ',x,,x,x' && calls === 3",
    );
    assert_true(
        "var calls = 0;
         var obj = { toLocaleString: function () { calls++; return 'p'; } };
         Array.prototype[1] = obj;
         var a = [obj];
         a.length = 2;
         var result = a.toLocaleString();
         delete Array.prototype[1];
         result === 'p,p' && calls === 2",
    );
}

#[test]
fn fill_and_copy_within_support_safe_integer_array_like_lengths() {
    assert_true(
        "var start = 9007199254740988;
         var o = { length: 9007199254740991 };
         var value = {};
         Array.prototype.fill.call(o, value, start, start + 3);
         o[start] === value && o[start + 1] === value && o[start + 2] === value",
    );
    assert_true(
        "var start = 9007199254740988;
         var o = { 0: 0, 1: 1, 2: 2, length: 9007199254740991 };
         o[start] = -3;
         o[start + 2] = -1;
         Array.prototype.copyWithin.call(o, 0, start, start + 3);
         o[0] === -3 && !(1 in o) && o[2] === -1",
    );
}

#[test]
fn reverse_array_scans_support_safe_integer_array_like_lengths() {
    assert_true(
        "var o = { length: 9007199254740991 };
         o[9007199254740990] = 'c';
         Array.prototype.includes.call(o, 'c', 9007199254740990)",
    );
    assert_true(
        "var called = [];
         var o = { length: Number.MAX_VALUE };
         Array.prototype.findLast.call(o, function (value, index) { called.push(index); return true; });
         called.length === 1 && called[0] === 9007199254740990",
    );
    assert_true(
        "var called = [];
         var o = { length: Number.MAX_VALUE };
         var found = Array.prototype.findLastIndex.call(o, function (value, index) { called.push(index); return true; });
         found === 9007199254740990 && called.length === 1 && called[0] === 9007199254740990",
    );
    assert_true(
        "var value = {};
         var index = 9007199254740990;
         var o = { length: 9007199254740991 };
         o[index] = value;
         Array.prototype.indexOf.call(o, value, 9007199254740988) === index",
    );
    assert_true(
        "var o = { length: 9007199254740991 };
         o[9007199254740990] = 'x';
         Array.prototype.pop.call(o) === 'x' && o.length === 9007199254740990 && !(9007199254740990 in o)",
    );
    assert_true(
        "var o = { length: 9007199254740990 };
         Array.prototype.push.call(o, 'x') === 9007199254740991 &&
         o[9007199254740990] === 'x' && o.length === 9007199254740991",
    );
    assert_error_name(
        "var o = { length: 9007199254740991 };
         Array.prototype.push.call(o, 'x')",
        "TypeError",
    );
    assert_true(
        "function StopUnshift() {}
         var o = {
           get '9007199254740986' () { throw new StopUnshift(); },
           '9007199254740987': '9007199254740987',
           '9007199254740989': '9007199254740989',
           '9007199254740991': '9007199254740991',
           length: 9007199254740990
         };
         try { Array.prototype.unshift.call(o, null); } catch (error) {}
         o.length === 9007199254740990 &&
         o['9007199254740987'] === '9007199254740987' &&
         o['9007199254740988'] === '9007199254740987' &&
         !('9007199254740989' in o) &&
         o['9007199254740990'] === '9007199254740989' &&
         o['9007199254740991'] === '9007199254740991'",
    );
    assert_true(
        "var value = {};
         var index = 9007199254740988;
         var o = { length: 9007199254740991 };
         o[index] = value;
         Array.prototype.lastIndexOf.call(o, value, 9007199254740990) === index",
    );
    assert_eval(
        "var o = { length: 9007199254740991 };
         o[9007199254740990] = 1;
         o[9007199254740988] = 3;
         Array.prototype.reduceRight.call(o, function (acc, value, index) {
           return acc + value + ':' + index + ';';
         }, '');",
        Value::String("1:9007199254740990;3:9007199254740988;".to_owned()),
    );
}

#[test]
fn array_unscopables_descriptor_is_installed() {
    assert_eval(
        "var u = Array.prototype[Symbol.unscopables];
         u.copyWithin === true && u.fill === true && u.find === true &&
         u.findIndex === true && u.findLast === true && u.findLastIndex === true &&
         u.values === true;",
        Value::Boolean(true),
    );
    assert_eval(
        "var d = Object.getOwnPropertyDescriptor(Array.prototype, Symbol.unscopables);
         d.writable === false && d.enumerable === false && d.configurable === true;",
        Value::Boolean(true),
    );
}

#[test]
fn reduce_right_uses_right_to_left_array_like_order() {
    assert_eval(
        "['a', 'b', 'c'].reduceRight(function (acc, value) { return acc + value; }, '');",
        Value::String("cba".to_owned()),
    );
    assert_eval(
        "var o = { length: 3, 0: 'a', 2: 'c' }; Array.prototype.reduceRight.call(o, function (acc, value) { return acc + value; }, '');",
        Value::String("ca".to_owned()),
    );
    assert_error_name(
        "Array.prototype.reduceRight.call({ length: 0 }, function (a, b) { return a + b; })",
        "TypeError",
    );
}

#[test]
fn missing_array_surface_methods_are_array_like() {
    assert_eval("[10, 20, 30].at(-1);", Value::Number(30.0));
    assert_eval("[1, 2, 3].includes(2);", Value::Boolean(true));
    assert_eval("[NaN].includes(NaN);", Value::Boolean(true));
    assert_eval("[NaN].indexOf(NaN);", Value::Number(-1.0));
    assert_eval("[NaN].lastIndexOf(NaN);", Value::Number(-1.0));
    assert_eval(
        "var a = ['a', 'b']; a.unshift('x'); a.join('');",
        Value::String("xab".to_owned()),
    );
    assert_eval(
        "var a = ['a', 'b']; a.shift(); a.join('');",
        Value::String("b".to_owned()),
    );
    assert_eval(
        "var a = ['a', 'b', 'c']; a.reverse(); a.join('');",
        Value::String("cba".to_owned()),
    );
    assert_eval(
        "[3, 1, 2].sort().join('');",
        Value::String("123".to_owned()),
    );
    assert_eval(
        "[3, 1, 2].sort(function (a, b) { return b - a; }).join('');",
        Value::String("321".to_owned()),
    );
    assert_eval(
        "[1, 2, 3].toReversed().join('');",
        Value::String("321".to_owned()),
    );
    assert_eval(
        "[3, 1, 2].toSorted().join('');",
        Value::String("123".to_owned()),
    );
    assert_eval(
        "[1, 2, 3].toSpliced(1, 1, 'x', 'y').join('');",
        Value::String("1xy3".to_owned()),
    );
    assert_eval(
        "[1, 2, 3].with(1, 9).join('');",
        Value::String("193".to_owned()),
    );
    assert_eval(
        "[1, 2, 3].flatMap(function (v) { return [v, v + 10]; }).join(',');",
        Value::String("1,11,2,12,3,13".to_owned()),
    );
    assert_eval(
        "var o = { length: 2, 0: 'b', 1: 'a' }; Array.prototype.sort.call(o); o[0] + o[1];",
        Value::String("ab".to_owned()),
    );
}

#[test]
fn huge_array_like_methods_do_not_dense_loop() {
    assert_error_name(
        "Array.prototype.map.call({ length: Math.pow(2, 32) }, function () { return 1; })",
        "RangeError",
    );
    assert_error_name(
        "Array.prototype.unshift.call({ length: 9007199254740991 }, null)",
        "TypeError",
    );
    assert_error_name(
        "var o = { length: 9007199254740991 };
         Object.defineProperty(o, '9007199254740990', { get: function () { throw new TypeError('stop'); } });
         Array.prototype.reverse.call(o)",
        "TypeError",
    );
    assert_eval(
        "var o = { length: 9007199254740989 };
         o[9007199254740987] = 'a';
         Array.prototype.unshift.call(o, 'x');
         o[0] + o[9007199254740988];",
        Value::String("xa".to_owned()),
    );
}
