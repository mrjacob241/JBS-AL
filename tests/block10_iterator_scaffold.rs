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

#[test]
fn iterator_global_and_prototype_are_exposed() {
    assert_true("typeof Iterator === 'function'");
    assert_true("Iterator.name === 'Iterator'");
    assert_true("Iterator.length === 0");
    assert_true("Iterator.prototype.constructor === Iterator");
    assert_true("Object.getOwnPropertyDescriptor(Iterator, 'prototype').writable === false");
    assert_true("Object.getOwnPropertyDescriptor(globalThis, 'Iterator').enumerable === false");
}

#[test]
fn iterator_prototype_symbol_iterator_returns_receiver() {
    assert_true("typeof Iterator.prototype[Symbol.iterator] === 'function'");
    assert_true("Iterator.prototype[Symbol.iterator].name === '[Symbol.iterator]'");
    assert_true("Iterator.prototype[Symbol.iterator].length === 0");
    assert_true("Iterator.prototype[Symbol.iterator].call(37) === 37");
}

#[test]
fn indexed_iterators_inherit_iterator_prototype_iterator_method() {
    assert_true(
        "var it = [1].values(); Object.getPrototypeOf(Object.getPrototypeOf(it)) === Iterator.prototype",
    );
    assert_true("var it = [1].values(); it[Symbol.iterator]() === it");
    assert_true(
        "var it = [1].values(); Object.getOwnPropertyDescriptor(it, Symbol.iterator) === undefined",
    );
    assert_true("var it = [1].values(); Object.getOwnPropertyDescriptor(it, 'next') === undefined");
}

#[test]
fn indexed_iterators_keep_state_internal_and_report_standard_tags() {
    assert_true(
        "var it = [1].values();
         Object.keys(it).length === 0 &&
         it.__jbs_iterator_target === undefined &&
         it.__jbs_iterator_kind === undefined &&
         it.__jbs_iterator_index === undefined",
    );
    assert_true("Object.prototype.toString.call([].values()) === '[object Array Iterator]'");
    assert_true(
        "Object.prototype.toString.call(''[Symbol.iterator]()) === '[object String Iterator]'",
    );
}

#[test]
fn indexed_iterator_next_requires_internal_iterator_state() {
    assert_true(
        "var next = Object.getPrototypeOf([].values()).next;
         var threw = false;
         try { next.call({}); } catch (e) { threw = e instanceof TypeError; }
         threw",
    );
}

#[test]
fn iterator_global_is_abstract_scaffold() {
    assert_true("!isConstructor(Iterator)");
    assert_true("var threw = false; try { Iterator(); } catch (e) { threw = e instanceof TypeError; } threw");
    assert_true(
        "var threw = false; try { new Iterator(); } catch (e) { threw = e instanceof TypeError; } threw",
    );
}
