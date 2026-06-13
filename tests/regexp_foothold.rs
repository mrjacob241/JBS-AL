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
fn single_quoted_strings_share_double_quoted_escape_behavior() {
    assert_eval("'plain';", Value::String("plain".to_owned()));
    assert_eval("'a\"b';", Value::String("a\"b".to_owned()));
    assert_eval("'a\\'b';", Value::String("a'b".to_owned()));
    assert_eval("'a\\nb\\tc';", Value::String("a\nb\tc".to_owned()));
    assert_eval("'a\\\\b' === \"a\\\\b\";", Value::Boolean(true));
}

#[test]
fn regexp_literals_construct_branded_objects_with_shape() {
    assert_eval("/abc/.source;", Value::String("abc".to_owned()));
    assert_eval("/abc/gi.flags;", Value::String("gi".to_owned()));
    assert_eval("/abc/.lastIndex;", Value::Number(0.0));
    assert_eval("/abc/g.toString();", Value::String("/abc/g".to_owned()));
}

#[test]
fn regexp_constructor_uses_same_creation_path() {
    assert_eval("new RegExp('abc').source;", Value::String("abc".to_owned()));
    assert_eval("RegExp('abc', 'i').flags;", Value::String("i".to_owned()));
    assert_eval("RegExp(/abc/).source;", Value::String("abc".to_owned()));
}

#[test]
fn regexp_test_and_exec_use_basic_general_matcher() {
    assert_eval("/abc/.test('xxabcyy');", Value::Boolean(true));
    assert_eval("/abc/.test('xxabx');", Value::Boolean(false));
    assert_eval("/abc/i.test('ABC');", Value::Boolean(true));
    assert_eval("/^abc/.test('abcdef');", Value::Boolean(true));
    assert_eval("/abc$/.test('xxabc');", Value::Boolean(true));
    assert_eval("/a.c/.exec('xxabc')[0];", Value::String("abc".to_owned()));
    assert_eval("/abc/.exec('xxabcyy').index;", Value::Number(2.0));
}
