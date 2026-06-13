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
fn map_constructor_and_basic_operations() {
    assert_true(
        "var m = new Map();
         m.size === 0 &&
         m.set('a', 1) === m &&
         m.size === 1 &&
         m.get('a') === 1 &&
         m.has('a') === true",
    );
    assert_true(
        "var m = new Map();
         m.set('a', 1);
         m.set('a', 2);
         m.size === 1 && m.get('a') === 2",
    );
    assert_true(
        "var m = new Map();
         m.set('a', 1);
         m.delete('a') === true &&
         m.delete('a') === false &&
         m.get('a') === undefined &&
         m.has('a') === false &&
         m.size === 0",
    );
    assert_true(
        "var m = new Map();
         m.set('a', 1);
         m.set('b', 2);
         m.clear();
         m.size === 0 && !m.has('a') && !m.has('b')",
    );
}

#[test]
fn set_constructor_and_basic_operations() {
    assert_true(
        "var s = new Set();
         s.size === 0 &&
         s.add('a') === s &&
         s.size === 1 &&
         s.has('a') === true",
    );
    assert_true(
        "var s = new Set();
         s.add('a');
         s.add('a');
         s.size === 1 && s.has('a') === true",
    );
    assert_true(
        "var s = new Set();
         s.add('a');
         s.delete('a') === true &&
         s.delete('a') === false &&
         s.has('a') === false &&
         s.size === 0",
    );
    assert_true(
        "var s = new Set();
         s.add('a');
         s.add('b');
         s.clear();
         s.size === 0 && !s.has('a') && !s.has('b')",
    );
}

#[test]
fn map_iteration_preserves_insertion_order() {
    assert_eval(
        "var m = new Map();
         m.set('a', 1);
         m.set('b', 2);
         m.set('a', 3);
         var text = '';
         for (var key of m.keys()) { text = text + key; }
         text;",
        Value::String("ab".to_owned()),
    );
    assert_eval(
        "var m = new Map();
         m.set('a', 1);
         m.set('b', 2);
         m.set('a', 3);
         var text = '';
         for (var value of m.values()) { text = text + value; }
         text;",
        Value::String("32".to_owned()),
    );
    assert_eval(
        "var m = new Map();
         m.set('a', 1);
         m.set('b', 2);
         var text = '';
         for (var pair of m.entries()) { text = text + pair[0] + pair[1]; }
         text;",
        Value::String("a1b2".to_owned()),
    );
    assert_eval(
        "var m = new Map([['a', 1], ['b', 2]]);
         var text = '';
         for (var pair of m) { text = text + pair[0] + pair[1]; }
         text;",
        Value::String("a1b2".to_owned()),
    );
}

#[test]
fn set_iteration_preserves_insertion_order() {
    assert_eval(
        "var s = new Set();
         s.add('a');
         s.add('b');
         s.add('a');
         var text = '';
         for (var value of s.values()) { text = text + value; }
         text;",
        Value::String("ab".to_owned()),
    );
    assert_eval(
        "var s = new Set();
         s.add('a');
         s.add('b');
         var text = '';
         for (var pair of s.entries()) { text = text + pair[0] + pair[1]; }
         text;",
        Value::String("aabb".to_owned()),
    );
    assert_eval(
        "var s = new Set(['a', 'b']);
         var text = '';
         for (var value of s) { text = text + value; }
         text;",
        Value::String("ab".to_owned()),
    );
}

#[test]
fn map_and_set_for_each_use_insertion_order_and_this_arg() {
    assert_eval(
        "var m = new Map([['a', 1], ['b', 2]]);
         var receiver = { prefix: 'm' };
         var text = '';
         m.forEach(function (value, key, collection) {
           text = text + this.prefix + key + value + (collection === m);
         }, receiver);
         text;",
        Value::String("ma1truemb2true".to_owned()),
    );
    assert_eval(
        "var s = new Set(['a', 'b']);
         var receiver = { prefix: 's' };
         var text = '';
         s.forEach(function (value, key, collection) {
           text = text + this.prefix + key + value + (collection === s);
         }, receiver);
         text;",
        Value::String("saatruesbbtrue".to_owned()),
    );
}

#[test]
fn iterable_constructors_consume_map_entries_and_set_values() {
    assert_true(
        "var source = {};
         source[Symbol.iterator] = function () {
           var index = 0;
           return {
             next: function () {
               index = index + 1;
               if (index === 1) return { value: ['a', 1], done: false };
               if (index === 2) return { value: ['b', 2], done: false };
               return { value: undefined, done: true };
             }
           };
         };
         var m = new Map(source);
         m.size === 2 && m.get('a') === 1 && m.get('b') === 2",
    );
    assert_true(
        "var source = {};
         source[Symbol.iterator] = function () {
           var index = 0;
           return {
             next: function () {
               index = index + 1;
               if (index === 1) return { value: 'a', done: false };
               if (index === 2) return { value: 'b', done: false };
               return { value: undefined, done: true };
             }
           };
         };
         var s = new Set(source);
         s.size === 2 && s.has('a') && s.has('b')",
    );
}

#[test]
fn collection_constructors_close_iterators_on_failure() {
    assert_eval(
        "var closed = 0;
         var source = {};
         source[Symbol.iterator] = function () {
           return {
             next: function () { return { value: 1, done: false }; },
             return: function () { closed = closed + 1; return {}; }
           };
         };
         try { new Map(source); } catch (error) {}
         closed;",
        Value::Number(1.0),
    );
    assert_eval(
        "var closed = 0;
         var originalAdd = Set.prototype.add;
         var source = {};
         source[Symbol.iterator] = function () {
           return {
             next: function () { return { value: 'a', done: false }; },
             return: function () { closed = closed + 1; return {}; }
           };
         };
         Set.prototype.add = function () { throw new Test262Error(); };
         try { new Set(source); } catch (error) {}
         Set.prototype.add = originalAdd;
         closed;",
        Value::Number(1.0),
    );
}

#[test]
fn collection_methods_reject_wrong_receivers() {
    assert_error_name("Map.prototype.get.call({})", "TypeError");
    assert_error_name("Map.prototype.set.call({}, 'a', 1)", "TypeError");
    assert_error_name("Map.prototype.has.call({}, 'a')", "TypeError");
    assert_error_name("Map.prototype.delete.call({}, 'a')", "TypeError");
    assert_error_name("Map.prototype.clear.call({})", "TypeError");
    assert_error_name(
        "Map.prototype.forEach.call({}, function () {})",
        "TypeError",
    );
    assert_error_name("Map.prototype.keys.call({})", "TypeError");
    assert_error_name("Map.prototype.values.call({})", "TypeError");
    assert_error_name("Map.prototype.entries.call({})", "TypeError");
    assert_error_name("Set.prototype.add.call({}, 'a')", "TypeError");
    assert_error_name("Set.prototype.has.call({}, 'a')", "TypeError");
    assert_error_name("Set.prototype.delete.call({}, 'a')", "TypeError");
    assert_error_name("Set.prototype.clear.call({})", "TypeError");
    assert_error_name(
        "Set.prototype.forEach.call({}, function () {})",
        "TypeError",
    );
    assert_error_name("Set.prototype.keys.call({})", "TypeError");
    assert_error_name("Set.prototype.values.call({})", "TypeError");
    assert_error_name("Set.prototype.entries.call({})", "TypeError");
}

#[test]
fn weak_collections_support_basic_object_and_symbol_keys() {
    assert_true(
        "var key = {};
         var wm = new WeakMap();
         wm.set(key, 7) === wm &&
         wm.has(key) &&
         wm.get(key) === 7",
    );
    assert_true(
        "var key = {};
         var ws = new WeakSet();
         ws.add(key) === ws &&
         ws.has(key)",
    );
    assert_true(
        "var key = Symbol();
         var wm = new WeakMap();
         wm.set(key, 3);
         wm.get(key) === 3",
    );
    assert_true(
        "var key = {};
         var wm = new WeakMap();
         wm.set(key, 1);
         wm.delete(key) === true &&
         wm.delete(key) === false &&
         wm.has(key) === false",
    );
}

#[test]
fn weak_collection_constructors_and_receiver_checks() {
    assert_true(
        "var key = {};
         var wm = new WeakMap([[key, 9]]);
         wm.get(key) === 9",
    );
    assert_true(
        "var key = {};
         var ws = new WeakSet([key]);
         ws.has(key)",
    );
    assert_true("Object.prototype.toString.call(new WeakMap()) === '[object WeakMap]'");
    assert_true("Object.prototype.toString.call(new WeakSet()) === '[object WeakSet]'");
    assert_error_name("WeakMap.prototype.set.call({}, {}, 1)", "TypeError");
    assert_error_name("WeakSet.prototype.add.call({}, {})", "TypeError");
    assert_error_name("new WeakMap([[1, 2]])", "TypeError");
    assert_error_name("new WeakSet([1])", "TypeError");
}
