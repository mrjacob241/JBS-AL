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
fn top_level_var_and_function_declarations_mirror_to_global_object() {
    assert_eval("var answer = 42; globalThis.answer;", Value::Number(42.0));
    assert_eval(
        "var answer = 1; this.answer = this.answer + 4; answer;",
        Value::Number(5.0),
    );
    assert_true("function visible() { return 7; } globalThis.visible === visible");
}

#[test]
fn closures_share_captured_binding_cells() {
    assert_eval(
        "var hits = 0; function inc() { hits++; } inc(); inc(); hits;",
        Value::Number(2.0),
    );
    assert_eval(
        "function make() { var x = 1; return function () { x++; return x; }; } var f = make(); f() + f();",
        Value::Number(5.0),
    );
}

#[test]
fn constructor_and_setter_callbacks_update_outer_bindings() {
    assert_eval(
        "var len = -1; var hits = 0; function C(length) { len = length; hits++; } Array.of.call(C, 'a', 'b'); len + hits;",
        Value::Number(3.0),
    );
    assert_eval(
        "var value = 0; function Pack() { Object.defineProperty(this, 'length', { set: function (len) { value = len; } }); } Array.of.call(Pack, 1, 2, 3, 4); value;",
        Value::Number(4.0),
    );
}

#[test]
fn arguments_objects_have_arguments_builtin_tag() {
    let mut runtime = Runtime::new();
    let result = runtime
        .eval_script(
            "(function () {
               return Object.prototype.toString.call(arguments);
             })(1, 2);",
        )
        .unwrap();
    assert_eq!(result, Value::String("[object Arguments]".to_owned()));
}
