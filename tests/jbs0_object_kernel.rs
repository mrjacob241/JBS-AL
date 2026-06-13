use jbs::runtime::{BuiltinFn, FunctionData, InternalMethods, IntrinsicId, JsObject};
use jbs::{
    Completion, Context, Descriptor, ErrorKind, ObjectRef, PropertyKey, Runtime, SameValue,
    SameValueZero, Value,
};

fn object(runtime: &mut Runtime, proto: Option<ObjectRef>) -> ObjectRef {
    runtime.heap_mut().allocate(JsObject::ordinary(proto))
}

fn builtin(runtime: &mut Runtime, name: &str, function: BuiltinFn) -> ObjectRef {
    runtime.heap_mut().allocate(JsObject::function(
        None,
        FunctionData::builtin(name, 0, function),
    ))
}

#[test]
fn same_value_preserves_nan_and_signed_zero_rules() {
    assert!(SameValue(
        &Value::Number(f64::NAN),
        &Value::Number(f64::NAN)
    ));
    assert!(!SameValue(&Value::Number(0.0), &Value::Number(-0.0)));
    assert!(SameValueZero(&Value::Number(0.0), &Value::Number(-0.0)));
}

#[test]
fn descriptor_rejects_data_and_accessor_shape() {
    let desc = Descriptor {
        value: Some(Value::Number(1.0)),
        writable: Some(true),
        get: Some(Value::Undefined),
        set: None,
        enumerable: Some(false),
        configurable: Some(false),
    };

    let error = desc.validate_shape().unwrap_err();
    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn non_configurable_non_writable_property_cannot_change_value() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let target = object(&mut runtime, None);
    let mut cx = Context::new(&mut runtime, realm);

    assert!(target
        .define_own_property(
            &mut cx,
            PropertyKey::from("x"),
            Descriptor::data(Value::Number(1.0), false, false, false),
        )
        .unwrap());

    assert!(!target
        .define_own_property(
            &mut cx,
            PropertyKey::from("x"),
            Descriptor::data(Value::Number(2.0), false, false, false),
        )
        .unwrap());

    assert_eq!(
        target
            .get(&mut cx, &PropertyKey::from("x"), Value::Object(target))
            .unwrap(),
        Value::Number(1.0)
    );
}

#[test]
fn prototype_lookup_uses_internal_get() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let proto = object(&mut runtime, None);
    let child = object(&mut runtime, Some(proto));
    let mut cx = Context::new(&mut runtime, realm);

    proto
        .define_own_property(
            &mut cx,
            PropertyKey::from("inherited"),
            Descriptor::data(Value::String("ok".to_owned()), true, true, true),
        )
        .unwrap();

    assert_eq!(
        child
            .get(
                &mut cx,
                &PropertyKey::from("inherited"),
                Value::Object(child)
            )
            .unwrap(),
        Value::String("ok".to_owned())
    );
}

#[test]
fn accessor_getter_is_called_by_internal_get() {
    fn getter(_cx: &mut Context, _this: Value, _args: &[Value]) -> Completion<Value> {
        Ok(Value::String("from getter".to_owned()))
    }

    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let target = object(&mut runtime, None);
    let getter = builtin(&mut runtime, "get x", getter);
    let mut cx = Context::new(&mut runtime, realm);

    target
        .define_own_property(
            &mut cx,
            PropertyKey::from("x"),
            Descriptor::accessor(Some(Value::Object(getter)), None, true, true),
        )
        .unwrap();

    assert_eq!(
        target
            .get(&mut cx, &PropertyKey::from("x"), Value::Object(target))
            .unwrap(),
        Value::String("from getter".to_owned())
    );
}

#[test]
fn inherited_accessor_setter_is_called_with_receiver() {
    fn setter(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
        let Value::Object(receiver) = this else {
            panic!("setter this should be receiver object");
        };
        receiver
            .define_own_property(
                cx,
                PropertyKey::from("observed"),
                Descriptor::data(args[0].clone(), true, true, true),
            )
            .unwrap();
        Ok(Value::Undefined)
    }

    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let proto = object(&mut runtime, None);
    let child = object(&mut runtime, Some(proto));
    let setter = builtin(&mut runtime, "set x", setter);
    let mut cx = Context::new(&mut runtime, realm);

    proto
        .define_own_property(
            &mut cx,
            PropertyKey::from("x"),
            Descriptor::accessor(None, Some(Value::Object(setter)), true, true),
        )
        .unwrap();

    assert!(child
        .set(
            &mut cx,
            PropertyKey::from("x"),
            Value::Number(7.0),
            Value::Object(child),
        )
        .unwrap());

    assert_eq!(
        child
            .get(
                &mut cx,
                &PropertyKey::from("observed"),
                Value::Object(child)
            )
            .unwrap(),
        Value::Number(7.0)
    );
}

#[test]
fn non_extensible_object_rejects_new_own_property() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let target = object(&mut runtime, None);
    let mut cx = Context::new(&mut runtime, realm);

    assert!(target.prevent_extensions(&mut cx).unwrap());
    assert!(!target
        .define_own_property(
            &mut cx,
            PropertyKey::from("x"),
            Descriptor::data(Value::Boolean(true), true, true, true),
        )
        .unwrap());
}

#[test]
fn own_property_keys_follow_ecma_order() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let target = object(&mut runtime, None);
    let mut cx = Context::new(&mut runtime, realm);

    for key in ["b", "1", "a", "0"] {
        target
            .define_own_property(
                &mut cx,
                PropertyKey::from(key),
                Descriptor::data(Value::String(key.to_owned()), true, true, true),
            )
            .unwrap();
    }

    let keys = target.own_property_keys(&mut cx).unwrap();
    assert_eq!(
        keys,
        vec![
            PropertyKey::from("0"),
            PropertyKey::from("1"),
            PropertyKey::from("b"),
            PropertyKey::from("a"),
        ]
    );
}

#[test]
fn define_property_or_throw_reports_false_as_type_error() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let target = object(&mut runtime, None);
    let mut cx = Context::new(&mut runtime, realm);

    target
        .define_own_property_or_throw(
            &mut cx,
            PropertyKey::from("x"),
            Descriptor::data(Value::Number(1.0), false, false, false),
        )
        .unwrap();

    let error = target
        .define_own_property_or_throw(
            &mut cx,
            PropertyKey::from("x"),
            Descriptor::data(Value::Number(2.0), false, false, false),
        )
        .unwrap_err();

    assert_eq!(error.kind, ErrorKind::Type);
}

#[test]
fn object_constructor_called_with_undefined_creates_ordinary_object() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let object_ctor = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::ObjectConstructor)
        .unwrap();
    let mut cx = Context::new(&mut runtime, realm);

    let result = cx
        .call_mut(
            Value::Object(object_ctor),
            Value::Undefined,
            &[Value::Undefined],
        )
        .unwrap();

    assert!(matches!(result, Value::Object(_)));
}

#[test]
fn set_prototype_rejects_cycles() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let parent = object(&mut runtime, None);
    let child = object(&mut runtime, Some(parent));
    let mut cx = Context::new(&mut runtime, realm);

    assert!(!parent.set_prototype_of(&mut cx, Some(child)).unwrap());
    assert_eq!(parent.get_prototype_of(&mut cx).unwrap(), None);
}

#[test]
fn default_realm_bootstraps_global_and_intrinsic_metadata() {
    let mut runtime = Runtime::new();
    let realm_id = runtime.default_realm();
    let global = runtime.realm(realm_id).unwrap().global_object;
    let object_ctor = runtime
        .realm(realm_id)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::ObjectConstructor)
        .expect("Object constructor intrinsic");

    let mut cx = Context::new(&mut runtime, realm_id);
    assert_eq!(
        global
            .get(
                &mut cx,
                &PropertyKey::from("globalThis"),
                Value::Object(global)
            )
            .unwrap(),
        Value::Object(global)
    );

    let object_name = cx
        .heap()
        .get(object_ctor)
        .unwrap()
        .properties
        .get(&PropertyKey::from("name"))
        .unwrap()
        .value
        .clone()
        .unwrap();

    assert_eq!(object_name, Value::String("Object".to_owned()));
}

fn object_static(cx: &mut Context, name: &str) -> Value {
    let global = cx.realm().unwrap().global_object;
    let object_ctor = global
        .get(cx, &PropertyKey::from("Object"), Value::Object(global))
        .unwrap();
    let Value::Object(object_ctor) = object_ctor else {
        panic!("Object global should be an object");
    };
    object_ctor
        .get(cx, &PropertyKey::from(name), Value::Object(object_ctor))
        .unwrap()
}

fn descriptor_object(
    runtime: &mut Runtime,
    value: Value,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) -> ObjectRef {
    let realm = runtime.default_realm();
    let proto = runtime
        .realm(realm)
        .unwrap()
        .intrinsics
        .get(IntrinsicId::ObjectPrototype)
        .unwrap();
    let desc = object(runtime, Some(proto));
    let mut cx = Context::new(runtime, realm);

    desc.define_own_property_or_throw(
        &mut cx,
        PropertyKey::from("value"),
        Descriptor::data(value, true, true, true),
    )
    .unwrap();
    desc.define_own_property_or_throw(
        &mut cx,
        PropertyKey::from("writable"),
        Descriptor::data(Value::Boolean(writable), true, true, true),
    )
    .unwrap();
    desc.define_own_property_or_throw(
        &mut cx,
        PropertyKey::from("enumerable"),
        Descriptor::data(Value::Boolean(enumerable), true, true, true),
    )
    .unwrap();
    desc.define_own_property_or_throw(
        &mut cx,
        PropertyKey::from("configurable"),
        Descriptor::data(Value::Boolean(configurable), true, true, true),
    )
    .unwrap();

    desc
}

#[test]
fn object_define_property_and_get_own_property_descriptor_round_trip() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let target = object(&mut runtime, None);
    let desc = descriptor_object(
        &mut runtime,
        Value::String("value".to_owned()),
        false,
        true,
        false,
    );
    let mut cx = Context::new(&mut runtime, realm);

    let define_property = object_static(&mut cx, "defineProperty");
    cx.call_mut(
        define_property,
        Value::Undefined,
        &[
            Value::Object(target),
            Value::String("x".to_owned()),
            Value::Object(desc),
        ],
    )
    .unwrap();

    let get_own_property_descriptor = object_static(&mut cx, "getOwnPropertyDescriptor");
    let result = cx
        .call_mut(
            get_own_property_descriptor,
            Value::Undefined,
            &[Value::Object(target), Value::String("x".to_owned())],
        )
        .unwrap();
    let Value::Object(result) = result else {
        panic!("descriptor result should be object");
    };

    assert_eq!(
        result
            .get(&mut cx, &PropertyKey::from("value"), Value::Object(result))
            .unwrap(),
        Value::String("value".to_owned())
    );
    assert_eq!(
        result
            .get(
                &mut cx,
                &PropertyKey::from("writable"),
                Value::Object(result)
            )
            .unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        result
            .get(
                &mut cx,
                &PropertyKey::from("enumerable"),
                Value::Object(result)
            )
            .unwrap(),
        Value::Boolean(true)
    );
}

#[test]
fn object_create_sets_requested_prototype() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let proto = object(&mut runtime, None);
    let mut cx = Context::new(&mut runtime, realm);

    let create = object_static(&mut cx, "create");
    let result = cx
        .call_mut(create, Value::Undefined, &[Value::Object(proto)])
        .unwrap();
    let Value::Object(result) = result else {
        panic!("Object.create result should be object");
    };

    assert_eq!(result.get_prototype_of(&mut cx).unwrap(), Some(proto));
}

#[test]
fn object_prototype_and_extensibility_statics_work() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let proto = object(&mut runtime, None);
    let target = object(&mut runtime, None);
    let mut cx = Context::new(&mut runtime, realm);

    let set_prototype_of = object_static(&mut cx, "setPrototypeOf");
    let get_prototype_of = object_static(&mut cx, "getPrototypeOf");
    let prevent_extensions = object_static(&mut cx, "preventExtensions");
    let is_extensible = object_static(&mut cx, "isExtensible");

    cx.call_mut(
        set_prototype_of,
        Value::Undefined,
        &[Value::Object(target), Value::Object(proto)],
    )
    .unwrap();
    assert_eq!(
        cx.call_mut(get_prototype_of, Value::Undefined, &[Value::Object(target)])
            .unwrap(),
        Value::Object(proto)
    );
    assert_eq!(
        cx.call_mut(
            is_extensible.clone(),
            Value::Undefined,
            &[Value::Object(target)]
        )
        .unwrap(),
        Value::Boolean(true)
    );
    cx.call_mut(
        prevent_extensions,
        Value::Undefined,
        &[Value::Object(target)],
    )
    .unwrap();
    assert_eq!(
        cx.call_mut(is_extensible, Value::Undefined, &[Value::Object(target)])
            .unwrap(),
        Value::Boolean(false)
    );
}

#[test]
fn object_is_has_own_and_keys_work() {
    let mut runtime = Runtime::new();
    let realm = runtime.default_realm();
    let target = object(&mut runtime, None);
    let mut cx = Context::new(&mut runtime, realm);

    target
        .define_own_property_or_throw(
            &mut cx,
            PropertyKey::from("hidden"),
            Descriptor::data(Value::Number(1.0), true, false, true),
        )
        .unwrap();
    target
        .define_own_property_or_throw(
            &mut cx,
            PropertyKey::from("shown"),
            Descriptor::data(Value::Number(2.0), true, true, true),
        )
        .unwrap();

    let object_is = object_static(&mut cx, "is");
    let has_own = object_static(&mut cx, "hasOwn");
    let keys = object_static(&mut cx, "keys");

    assert_eq!(
        cx.call_mut(
            object_is,
            Value::Undefined,
            &[Value::Number(0.0), Value::Number(-0.0)]
        )
        .unwrap(),
        Value::Boolean(false)
    );
    assert_eq!(
        cx.call_mut(
            has_own,
            Value::Undefined,
            &[Value::Object(target), Value::String("hidden".to_owned())]
        )
        .unwrap(),
        Value::Boolean(true)
    );

    let result = cx
        .call_mut(keys, Value::Undefined, &[Value::Object(target)])
        .unwrap();
    let Value::Object(result) = result else {
        panic!("Object.keys result should be array-like object");
    };
    assert_eq!(
        result
            .get(&mut cx, &PropertyKey::from("0"), Value::Object(result))
            .unwrap(),
        Value::String("shown".to_owned())
    );
    assert_eq!(
        result
            .get(&mut cx, &PropertyKey::from("length"), Value::Object(result))
            .unwrap(),
        Value::Number(1.0)
    );
}

#[test]
fn object_assign_boxes_symbol_targets() {
    let mut runtime = Runtime::new();
    let result = runtime
        .eval_script(
            "var target = Symbol('foo');
             var result = Object.assign(target, { a: 1 });
             typeof result === 'object' && result.a === 1 && result.toString() === 'Symbol(foo)';",
        )
        .unwrap();
    assert_eq!(result, Value::Boolean(true));
}
