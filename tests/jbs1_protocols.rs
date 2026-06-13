use jbs::runtime::{
    ArgView, Brand, BuiltinFn, FunctionData, InternalMethods, IntrinsicId, JsObject,
    LengthOfArrayLike, ReceiverView, ToIntegerOrInfinity, ToLength,
};
use jbs::{Completion, Context, Descriptor, ObjectRef, PropertyKey, Runtime, Value};

fn builtin(runtime: &mut Runtime, name: &str, function: BuiltinFn) -> ObjectRef {
    let realm = runtime.default_realm();
    let function_proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::FunctionPrototype);
    runtime.heap_mut().allocate(JsObject::function(
        function_proto,
        FunctionData::builtin(name, 0, function),
    ))
}

#[test]
fn arg_view_to_property_key_handles_primitives() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let mut cx = Context::new(&mut runtime, realm);

    assert_eq!(
        ArgView::new(Value::Null).to_property_key(&mut cx).unwrap(),
        PropertyKey::from("null")
    );
    assert_eq!(
        ArgView::new(Value::Undefined)
            .to_property_key(&mut cx)
            .unwrap(),
        PropertyKey::from("undefined")
    );
    assert_eq!(
        ArgView::new(Value::Number(-0.0))
            .to_property_key(&mut cx)
            .unwrap(),
        PropertyKey::from("0")
    );
    assert_eq!(
        ArgView::new(Value::Number(1e21))
            .to_property_key(&mut cx)
            .unwrap(),
        PropertyKey::from("1e+21")
    );
    assert_eq!(
        ArgView::new(Value::Number(1e-7))
            .to_property_key(&mut cx)
            .unwrap(),
        PropertyKey::from("1e-7")
    );
    assert_eq!(
        ArgView::new(Value::Boolean(true))
            .to_property_key(&mut cx)
            .unwrap(),
        PropertyKey::from("true")
    );
}

#[test]
fn array_objects_convert_to_property_keys_through_join() {
    let mut runtime = Runtime::new();
    let result = runtime
        .eval_script(
            "var obj = {}; Object.defineProperty(obj, [1, 2], {}); obj.hasOwnProperty('1,2');",
        )
        .unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn arg_view_to_object_boxes_primitives_as_ordinary_objects() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let string_proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::StringPrototype)
        .unwrap();
    let mut cx = Context::new(&mut runtime, realm);

    let object = ArgView::new(Value::String("x".to_owned()))
        .to_object(&mut cx)
        .unwrap();

    assert_eq!(
        object.get_prototype_of(&mut cx).unwrap(),
        Some(string_proto)
    );
    assert!(cx
        .heap()
        .get(object)
        .unwrap()
        .has_brand(Brand::PrimitiveWrapper));
}

#[test]
fn receiver_view_checks_internal_slot_brands() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let array = runtime.eval_script("[1, 2, 3]").unwrap();
    let mut cx = Context::new(&mut runtime, realm);

    let object = ReceiverView::new(array)
        .require_brand(&mut cx, Brand::Array)
        .unwrap();
    assert!(cx.heap().get(object).unwrap().has_brand(Brand::Array));

    assert!(ReceiverView::new(Value::Number(1.0))
        .require_brand(&mut cx, Brand::Array)
        .is_err());
}

#[test]
fn abstract_operations_are_shared_entry_points() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let array = runtime.eval_script("[1, 2, 3]").unwrap();
    let mut cx = Context::new(&mut runtime, realm);

    assert_eq!(
        ToIntegerOrInfinity(&mut cx, Value::Number(3.9)).unwrap(),
        3.0
    );
    assert_eq!(ToLength(&mut cx, Value::Number(-1.0)).unwrap(), 0);

    let Value::Object(object) = array else {
        panic!("array literal should produce object");
    };
    assert_eq!(LengthOfArrayLike(&mut cx, object).unwrap(), 3);
}

#[test]
fn function_prototype_call_rebinds_this() {
    let mut runtime = Runtime::new();
    let result = runtime
        .eval_script("Object.prototype.toString.call(null);")
        .unwrap();
    assert_eq!(result, Value::String("[object Null]".to_owned()));
}

#[test]
fn object_to_property_key_uses_to_string_hint() {
    let mut runtime = Runtime::new();
    fn to_string(_cx: &mut Context, _this: Value, _args: &[Value]) -> Completion<Value> {
        Ok(Value::String("coerced".to_owned()))
    }
    fn value_of(_cx: &mut Context, _this: Value, _args: &[Value]) -> Completion<Value> {
        Ok(Value::String("wrong".to_owned()))
    }

    let realm = runtime.default_realm();
    let object_proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::ObjectPrototype);
    let to_string = builtin(&mut runtime, "toString", to_string);
    let value_of = builtin(&mut runtime, "valueOf", value_of);
    let mut cx = Context::new(&mut runtime, realm);
    let object = cx.heap_mut().allocate(JsObject::ordinary(object_proto));
    object
        .define_own_property_or_throw(
            &mut cx,
            PropertyKey::from("toString"),
            Descriptor::data(Value::Object(to_string), true, true, true),
        )
        .unwrap();
    object
        .define_own_property_or_throw(
            &mut cx,
            PropertyKey::from("valueOf"),
            Descriptor::data(Value::Object(value_of), true, true, true),
        )
        .unwrap();

    let key = ArgView::new(Value::Object(object))
        .to_property_key(&mut cx)
        .unwrap();
    assert_eq!(key, PropertyKey::from("coerced"));
}

#[test]
fn runtime_number_conversion_uses_value_of_hint() {
    let mut runtime = Runtime::new();
    fn value_of(_cx: &mut Context, _this: Value, _args: &[Value]) -> Completion<Value> {
        Ok(Value::Number(3.0))
    }

    let realm = runtime.default_realm();
    let object_proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::ObjectPrototype);
    let value_of = builtin(&mut runtime, "valueOf", value_of);
    let mut cx = Context::new(&mut runtime, realm);
    let object = cx.heap_mut().allocate(JsObject::ordinary(object_proto));
    object
        .define_own_property_or_throw(
            &mut cx,
            PropertyKey::from("valueOf"),
            Descriptor::data(Value::Object(value_of), true, true, true),
        )
        .unwrap();

    assert_eq!(
        ArgView::new(Value::Object(object))
            .to_number(&mut cx)
            .unwrap(),
        3.0
    );
}

#[test]
fn primitive_prototype_methods_accept_primitive_receivers() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let boolean_proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::BooleanPrototype)
        .unwrap();
    let number_proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::NumberPrototype)
        .unwrap();
    let string_proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::StringPrototype)
        .unwrap();
    let mut cx = Context::new(&mut runtime, realm);

    let boolean_to_string = boolean_proto.get(
        &mut cx,
        &PropertyKey::from("toString"),
        Value::Object(boolean_proto),
    );
    let number_to_string = number_proto.get(
        &mut cx,
        &PropertyKey::from("toString"),
        Value::Object(number_proto),
    );
    let string_value_of = string_proto.get(
        &mut cx,
        &PropertyKey::from("valueOf"),
        Value::Object(string_proto),
    );

    assert_eq!(
        cx.call_mut(boolean_to_string.unwrap(), Value::Boolean(true), &[])
            .unwrap(),
        Value::String("true".to_owned())
    );
    assert_eq!(
        cx.call_mut(number_to_string.unwrap(), Value::Number(12.0), &[])
            .unwrap(),
        Value::String("12".to_owned())
    );
    assert_eq!(
        cx.call_mut(
            string_value_of.unwrap(),
            Value::String("ok".to_owned()),
            &[]
        )
        .unwrap(),
        Value::String("ok".to_owned())
    );
}
