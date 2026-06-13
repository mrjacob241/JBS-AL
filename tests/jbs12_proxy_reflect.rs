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

fn assert_type_error(source: &str) {
    assert_eval(source, Value::String("TypeError".to_owned()));
}

#[test]
fn reflect_wraps_ordinary_internal_methods() {
    assert_eval("var o = { a: 1 }; Reflect.get(o, 'a');", Value::Number(1.0));
    assert_true("var o = {}; Reflect.set(o, 'a', 2) && o.a === 2");
    assert_true("var o = {}; Reflect.defineProperty(o, 'x', { value: 7 }); o.x === 7");
    assert_true("var o = { x: 1 }; Reflect.deleteProperty(o, 'x') && !('x' in o)");
    assert_true("var proto = {}; var o = {}; Reflect.setPrototypeOf(o, proto) && Reflect.getPrototypeOf(o) === proto");
    assert_true("var o = { a: 1 }; Reflect.ownKeys(o)[0] === 'a'");
    assert_true("var o = {}; Reflect.preventExtensions(o) && !Reflect.isExtensible(o)");
}

#[test]
fn reflect_apply_and_construct_use_shared_call_paths() {
    assert_eval(
        "function add(a, b) { return this.base + a + b; } Reflect.apply(add, { base: 3 }, [4, 5]);",
        Value::Number(12.0),
    );
    assert_true(
        "function C(v) { this.v = v; } var o = Reflect.construct(C, [9]); o instanceof C && o.v === 9",
    );
    assert_true(
        "function C() { this.seen = Object.getPrototypeOf(this); }
         var o = Reflect.construct(C, [], Array);
         Object.getPrototypeOf(o) === Array.prototype && o.seen === Array.prototype",
    );
    assert_eval(
        "function f() {} try { Reflect.construct(f, 1); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "function f() {} try { Reflect.construct(f, [], {}); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "function f() {} try { Reflect.apply(f, null, 1); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
}

#[test]
fn transparent_proxy_forwards_internal_methods_to_target() {
    assert_true("var target = { a: 1 }; var p = new Proxy(target, {}); Reflect.get(p, 'a') === 1");
    assert_true(
        "var target = {}; var p = new Proxy(target, {}); Reflect.set(p, 'x', 4) && target.x === 4",
    );
    assert_true("var target = {}; var p = new Proxy(target, {}); Reflect.defineProperty(p, 'x', { value: 5 }); target.x === 5");
    assert_true("var target = { x: 1 }; var p = new Proxy(target, {}); Reflect.deleteProperty(p, 'x') && !('x' in target)");
    assert_true("var target = []; var p = new Proxy(target, {}); Array.isArray(p)");
}

#[test]
fn callable_proxy_forwards_and_dispatches_apply_trap() {
    assert_eval(
        "function add(a, b) { return this.base + a + b; }
         var p = new Proxy(add, {});
         p.call({ base: 3 }, 4, 5);",
        Value::Number(12.0),
    );
    assert_eval(
        "function add(a, b) { return this.base + a + b; }
         var p = new Proxy(add, {});
         Reflect.apply(p, { base: 3 }, [4, 5]);",
        Value::Number(12.0),
    );
    assert_eval(
        "function add(a, b) { return this.base + a + b; }
         var seen = '';
         var p = new Proxy(add, {
             apply: function (target, thisArg, args) {
                 seen = thisArg.base + ':' + args.length;
                 return Reflect.apply(target, thisArg, args) + 1;
             }
         });
         p.call({ base: 3 }, 4, 5) === 13 && seen === '3:2';",
        Value::Boolean(true),
    );
}

#[test]
fn constructible_proxy_forwards_and_dispatches_construct_trap() {
    assert_true(
        "function C(v) { this.v = v; }
         var p = new Proxy(C, {});
         var o = new p(9);
         o instanceof C && o.v === 9",
    );
    assert_true(
        "function C(v) { this.v = v; }
         var p = new Proxy(C, {});
         var o = Reflect.construct(p, [9]);
         o instanceof C && o.v === 9",
    );
    assert_true(
        "function C(v) { this.v = v; }
         var seen = '';
         var replacement = { v: 12 };
         var p = new Proxy(C, {
             construct: function (target, args, newTarget) {
                 seen = args.length + ':' + args[0] + ':' + (newTarget === p);
                 return replacement;
             }
         });
         new p(9) === replacement && seen === '1:9:true'",
    );
    assert_true(
        "var C = new Function();
         var P = new Proxy(function () {}, {});
         Object.getPrototypeOf(Reflect.construct(P, [], C)) === C.prototype",
    );
}

#[test]
fn proxy_mutating_and_getting_traps_are_called() {
    assert_eval(
        "var p = new Proxy({}, { get: function (target, key, receiver) { return key === 'x' ? 11 : 0; } }); p.x;",
        Value::Number(11.0),
    );
    assert_true(
        "var seen = '';
         var p = new Proxy({}, { set: function (target, key, value, receiver) { seen = key + ':' + value; return true; } });
         Reflect.set(p, 'x', 4) && seen === 'x:4'",
    );
    assert_true(
        "var seen = '';
         var p = new Proxy({}, { defineProperty: function (target, key, desc) { seen = key + ':' + desc.value; return true; } });
         Reflect.defineProperty(p, 'x', { value: 8 }) && seen === 'x:8'",
    );
    assert_eval(
        "var p = new Proxy({}, { deleteProperty: function () { return false; } }); Reflect.deleteProperty(p, 'x');",
        Value::Boolean(false),
    );
    assert_eval(
        "var p = new Proxy({}, { preventExtensions: function () { return false; } }); Reflect.preventExtensions(p);",
        Value::Boolean(false),
    );
}

#[test]
fn proxy_get_null_trap_forwards_to_proxy_target_with_switch_handler() {
    assert_true(
        "var sym = Symbol();
         var target = new Proxy({}, {
             get: function (_target, key) {
                 switch (key) {
                     case sym: return 1;
                     case '10': return 2;
                     case 'foo': return 3;
                 }
             }
         });
         var proxy = new Proxy(target, { get: null });
         proxy[sym] === 1 && proxy[10] === 2 && Object.create(proxy).foo === 3 && proxy.bar === undefined",
    );
}

#[test]
fn proxy_reflection_traps_are_called() {
    assert_true(
        "var seen = '';
         var p = new Proxy({}, { has: function (target, key) { seen = key; return true; } });
         Reflect.has(p, 'x') && seen === 'x'",
    );
    assert_true(
        "var p = new Proxy({}, { getOwnPropertyDescriptor: function () { return { value: 3, configurable: true }; } });
         Reflect.getOwnPropertyDescriptor(p, 'x').value === 3",
    );
    assert_true(
        "var p = new Proxy({}, { ownKeys: function () { return ['b', 'a']; } });
         var keys = Reflect.ownKeys(p); keys[0] === 'b' && keys[1] === 'a'",
    );
    assert_true(
        "var proto = {};
         var p = new Proxy({}, { getPrototypeOf: function () { return proto; } });
         Reflect.getPrototypeOf(p) === proto",
    );
    assert_eval(
        "var p = new Proxy({}, { isExtensible: function () { return true; } }); Reflect.isExtensible(p);",
        Value::Boolean(true),
    );
}

#[test]
fn with_statement_uses_proxy_has_for_identifier_resolution() {
    assert_true(
        "var p = new Proxy({ x: 7 }, { has: function (target, key) { return key === 'x'; } });
         var value;
         with (p) { value = x; }
         value === 7",
    );
    assert_true(
        "var x = 3;
         var p = new Proxy({}, { has: function (target, key) { return false; } });
         with (p) { x === 3; }",
    );
    assert_true(
        "var target = { x: 1 };
         var p = new Proxy(target, { has: function () { return true; } });
         with (p) { x = 4; }
         target.x === 4",
    );
    assert_true(
        "var target = { x: 1 };
         var p = new Proxy(target, { has: function () { return true; } });
         with (p) { delete x; }
         !('x' in target)",
    );
}

#[test]
fn proxy_mutating_traps_enforce_basic_target_invariants() {
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var p = new Proxy(target, { deleteProperty: function () { return true; } });
         try { Reflect.deleteProperty(p, 'x'); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; var p = new Proxy(target, { preventExtensions: function () { return true; } });
         try { Reflect.preventExtensions(p); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, writable: false, configurable: false });
         var p = new Proxy(target, { set: function () { return true; } });
         try { Reflect.set(p, 'x', 2); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_true(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, writable: false, configurable: false });
         var p = new Proxy(target, { set: function () { return true; } });
         Reflect.set(p, 'x', 1)",
    );
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { get: function () { return 1; }, set: undefined, configurable: false });
         var p = new Proxy(target, { set: function () { return true; } });
         try { Reflect.set(p, 'x', 2); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = { x: 1 }; Object.preventExtensions(target);
         var p = new Proxy(target, { deleteProperty: function () { return true; } });
         try { Reflect.deleteProperty(p, 'x'); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_true(
        "var target = {}; Object.preventExtensions(target);
         var p = new Proxy(target, { deleteProperty: function () { return true; } });
         Reflect.deleteProperty(p, 'x')",
    );
}

#[test]
fn proxy_missing_nullish_mutating_traps_forward_to_proxy_targets() {
    assert_true(
        "var target = {};
         var inner = new Proxy(target, { set: function (t, k, v) { t[k] = v + 1; return true; } });
         var outer = new Proxy(inner, {});
         Reflect.set(outer, 'x', 4) && target.x === 5",
    );
    assert_true(
        "var target = {};
         var inner = new Proxy(target, { set: function (t, k, v) { t[k] = v + 1; return true; } });
         var outer = new Proxy(inner, { set: null });
         Reflect.set(outer, 'x', 4) && target.x === 5",
    );
    assert_true(
        "var target = {};
         var inner = new Proxy(target, { set: function (t, k, v) { t[k] = v + 1; return true; } });
         var outer = new Proxy(inner, { set: undefined });
         Reflect.set(outer, 'x', 4) && target.x === 5",
    );
    assert_true(
        "var target = { x: 1 };
         var inner = new Proxy(target, { deleteProperty: function (t, k) { return delete t[k]; } });
         var outer = new Proxy(inner, {});
         Reflect.deleteProperty(outer, 'x') && !('x' in target)",
    );
    assert_true(
        "var target = { x: 1 };
         var inner = new Proxy(target, { deleteProperty: function (t, k) { return delete t[k]; } });
         var outer = new Proxy(inner, { deleteProperty: null });
         Reflect.deleteProperty(outer, 'x') && !('x' in target)",
    );
    assert_true(
        "var target = { x: 1 };
         var inner = new Proxy(target, { deleteProperty: function (t, k) { return delete t[k]; } });
         var outer = new Proxy(inner, { deleteProperty: undefined });
         Reflect.deleteProperty(outer, 'x') && !('x' in target)",
    );
}

#[test]
fn proxy_missing_nullish_traps_forward_inner_target_invariants() {
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var inner = new Proxy(target, { deleteProperty: function () { return true; } });
         var outer = new Proxy(inner, {});
         try { Reflect.deleteProperty(outer, 'x'); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var inner = new Proxy(target, { deleteProperty: function () { return true; } });
         var outer = new Proxy(inner, { deleteProperty: null });
         try { Reflect.deleteProperty(outer, 'x'); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var inner = new Proxy(target, { deleteProperty: function () { return true; } });
         var outer = new Proxy(inner, { deleteProperty: undefined });
         try { Reflect.deleteProperty(outer, 'x'); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, writable: false, configurable: false });
         var inner = new Proxy(target, { set: function () { return true; } });
         var outer = new Proxy(inner, {});
         try { Reflect.set(outer, 'x', 2); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, writable: false, configurable: false });
         var inner = new Proxy(target, { set: function () { return true; } });
         var outer = new Proxy(inner, { set: null });
         try { Reflect.set(outer, 'x', 2); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, writable: false, configurable: false });
         var inner = new Proxy(target, { set: function () { return true; } });
         var outer = new Proxy(inner, { set: undefined });
         try { Reflect.set(outer, 'x', 2); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {};
         var inner = new Proxy(target, { preventExtensions: function () { return true; } });
         var outer = new Proxy(inner, {});
         try { Reflect.preventExtensions(outer); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {};
         var inner = new Proxy(target, { preventExtensions: function () { return true; } });
         var outer = new Proxy(inner, { preventExtensions: null });
         try { Reflect.preventExtensions(outer); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {};
         var inner = new Proxy(target, { preventExtensions: function () { return true; } });
         var outer = new Proxy(inner, { preventExtensions: undefined });
         try { Reflect.preventExtensions(outer); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var inner = new Proxy(target, { has: function () { return false; } });
         var outer = new Proxy(inner, {});
         try { Reflect.has(outer, 'x'); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var inner = new Proxy(target, { has: function () { return false; } });
         var outer = new Proxy(inner, { has: null });
         try { Reflect.has(outer, 'x'); 'bad'; } catch (e) { e.name; }",
    );
    assert_type_error(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var inner = new Proxy(target, { has: function () { return false; } });
         var outer = new Proxy(inner, { has: undefined });
         try { Reflect.has(outer, 'x'); 'bad'; } catch (e) { e.name; }",
    );
}

#[test]
fn proxy_reflection_traps_enforce_target_invariants() {
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var p = new Proxy(target, { has: function () { return false; } });
         try { Reflect.has(p, 'x'); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var p = new Proxy(target, { getOwnPropertyDescriptor: function () { return undefined; } });
         try { Reflect.getOwnPropertyDescriptor(p, 'x'); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, writable: true, configurable: false });
         var p = new Proxy(target, { getOwnPropertyDescriptor: function () { return { value: 1, writable: false, configurable: false }; } });
         try { Reflect.getOwnPropertyDescriptor(p, 'x'); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, configurable: false });
         var p = new Proxy(target, { ownKeys: function () { return []; } });
         try { Reflect.ownKeys(p); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; Object.preventExtensions(target);
         var p = new Proxy(target, { ownKeys: function () { return ['extra']; } });
         try { Reflect.ownKeys(p); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var proto = {}; var target = Object.create(proto); Object.preventExtensions(target);
         var p = new Proxy(target, { getPrototypeOf: function () { return {}; } });
         try { Reflect.getPrototypeOf(p); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; Object.preventExtensions(target);
         var p = new Proxy(target, { isExtensible: function () { return true; } });
         try { Reflect.isExtensible(p); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = {}; Object.defineProperty(target, 'x', { value: 1, writable: false, configurable: false });
         var p = new Proxy(target, { get: function () { return 2; } });
         try { p.x; 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
}

#[test]
fn proxy_revocation_invalidates_forwarding_internal_methods() {
    assert_eval(
        "var pair = Proxy.revocable({ x: 1 }, {}); pair.revoke(); try { Reflect.get(pair.proxy, 'x'); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var pair = Proxy.revocable([], {}); pair.revoke(); try { Array.isArray(pair.proxy); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
}

#[test]
fn proxy_can_be_created_with_revoked_proxy_target() {
    assert_eval(
        "var pair = Proxy.revocable({}, {}); pair.revoke(); typeof new Proxy(pair.proxy, {});",
        Value::String("object".to_owned()),
    );
    assert_eval(
        "var pair = Proxy.revocable(function () {}, {}); pair.revoke(); typeof new Proxy(pair.proxy, {});",
        Value::String("function".to_owned()),
    );
    assert_eval(
        "var pair = Proxy.revocable({}, {}); pair.revoke(); var outer = Proxy.revocable(pair.proxy, {}); typeof outer.proxy;",
        Value::String("object".to_owned()),
    );
    assert_eval(
        "var pair = Proxy.revocable(function () {}, {}); pair.revoke(); var outer = Proxy.revocable(pair.proxy, {}); typeof outer.proxy;",
        Value::String("function".to_owned()),
    );
}

#[test]
fn proxy_constructor_requires_new_and_revoker_has_spec_name() {
    assert_eval(
        "try { Proxy({}, {}); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_true(
        "var desc = Object.getOwnPropertyDescriptor(Proxy.revocable({}, {}).revoke, 'name');
         desc.value === '' && desc.writable === false && desc.enumerable === false && desc.configurable === true",
    );
}

#[test]
fn strict_failed_set_delete_and_object_prevent_extensions_throw() {
    assert_eval(
        "var f = function() {};
         var p = new Proxy(new Proxy(f, {}), {});
         try { (function() { 'use strict'; delete p.prototype; })(); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = { get x() {} };
         var p = new Proxy(new Proxy(target, {}), {});
         try { (function() { 'use strict'; p.x = 1; })(); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_eval(
        "var target = new Proxy({}, { preventExtensions: function () { return false; } });
         var p = new Proxy(target, {});
         try { Object.preventExtensions(p); 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
}

#[test]
fn ordinary_prototype_index_operations_forward_to_proxy_traps() {
    assert_true(
        "var called = false;
         var proxy = new Proxy({}, { getPrototypeOf: function () { called = true; return null; } });
         var array = [];
         Object.setPrototypeOf(array, proxy) === array && called === false",
    );
    assert_true(
        "var seen = '';
         var target = Object.create([14]);
         var handler = { has: function (target, prop) { seen = prop; return false; } };
         var proxy = new Proxy(target, handler);
         var array = [];
         Object.setPrototypeOf(array, proxy);
         (1 in array) === false && seen === '1'",
    );
    assert_true(
        "var seen = '';
         var target = {};
         var handler = { set: function (target, prop, value, receiver) { seen = prop + ':' + value + ':' + (receiver === array); return true; } };
         var proxy = new Proxy(target, handler);
         var array = new Array(1);
         Object.setPrototypeOf(array, proxy);
         array[0] = 1;
         seen === '0:1:true'",
    );
}

#[test]
fn regexp_prototype_flags_and_symbol_replace_are_visible_through_proxy_targets() {
    assert_true(
        "var regExp = /(?:)/m;
         var p = new Proxy(new Proxy(regExp, {}), {});
         Reflect.has(p, 'ignoreCase') && Symbol.replace in p && ('lastIndex' in Object.create(p))",
    );
    assert_true(
        "var regExp = /(?:)/g;
         var p = new Proxy(new Proxy(regExp, {}), {});
         Reflect.set(p, 'global', true) === false",
    );
}

#[test]
fn assignment_creation_uses_proxy_receiver_define_own_property_invariants() {
    assert_eval(
        "var target = {};
         var p = new Proxy(target, { defineProperty: function () { return true; } });
         Object.preventExtensions(target);
         try { p.prop = null; 'bad'; } catch (e) { e.name; }",
        Value::String("TypeError".to_owned()),
    );
    assert_true(
        "var target = {};
         Object.preventExtensions(target);
         var p = new Proxy(target, {});
         Reflect.set(p, 'prop', 1) === false",
    );
}
