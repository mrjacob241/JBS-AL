use super::{
    ArgView, Completion, Context, Descriptor, InternalMethods, InternalSlot, IntrinsicId, JsError,
    JsObject, ObjectRef, PropertyKey, Value,
};

#[allow(non_snake_case)]
pub fn ToNumber(cx: &mut Context, value: Value) -> Completion<f64> {
    ArgView::new(value).to_number(cx)
}

#[allow(non_snake_case)]
pub fn ToString(cx: &mut Context, value: Value) -> Completion<String> {
    ArgView::new(value).to_string(cx)
}

#[allow(non_snake_case)]
pub fn ToObject(cx: &mut Context, value: Value) -> Completion<ObjectRef> {
    ArgView::new(value).to_object(cx)
}

#[allow(non_snake_case)]
pub fn ToIntegerOrInfinity(cx: &mut Context, value: Value) -> Completion<f64> {
    let number = ToNumber(cx, value)?;
    if number.is_nan() || number == 0.0 {
        return Ok(0.0);
    }
    if !number.is_finite() {
        return Ok(number);
    }
    Ok(number.signum() * number.abs().floor())
}

#[allow(non_snake_case)]
pub fn ToLength(cx: &mut Context, value: Value) -> Completion<u32> {
    const MAX_SAFE_LENGTH: f64 = 9_007_199_254_740_991.0;
    let integer = ToIntegerOrInfinity(cx, value)?;
    if integer <= 0.0 {
        Ok(0)
    } else {
        Ok(integer.min(MAX_SAFE_LENGTH).min(u32::MAX as f64) as u32)
    }
}

#[allow(non_snake_case)]
pub fn LengthOfArrayLike(cx: &mut Context, object: ObjectRef) -> Completion<u32> {
    let value = object.get(cx, &PropertyKey::from("length"), Value::Object(object))?;
    ToLength(cx, value)
}

#[allow(non_snake_case)]
pub fn CreateDataPropertyOrThrow(
    cx: &mut Context,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
) -> Completion<()> {
    object.define_own_property_or_throw(cx, key, Descriptor::data(value, true, true, true))
}

#[allow(non_snake_case)]
pub fn CreateArrayFromList(cx: &mut Context, values: Vec<Value>) -> Completion<ObjectRef> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::ArrayPrototype)
        .or_else(|| {
            cx.realm()
                .ok()?
                .intrinsics
                .get(IntrinsicId::ObjectPrototype)
        })
        .ok_or_else(|| JsError::internal("missing Array.prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::array(Some(proto)));
    let len = values.len();
    for (index, value) in values.into_iter().enumerate() {
        CreateDataPropertyOrThrow(cx, object, PropertyKey::array_index(index as u64), value)?;
    }
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from("length"),
        Descriptor::data(Value::Number(len as f64), true, false, false),
    )?;
    Ok(object)
}

#[allow(non_snake_case)]
pub fn GetMethod(cx: &mut Context, value: Value, key: PropertyKey) -> Completion<Option<Value>> {
    let object = ToObject(cx, value.clone())?;
    let method = object.get(cx, &key, value)?;
    if matches!(method, Value::Undefined | Value::Null) {
        return Ok(None);
    }
    if !cx.is_callable(&method)? {
        return Err(JsError::type_error("method property is not callable"));
    }
    Ok(Some(method))
}

#[allow(non_snake_case)]
pub fn RegExpCreate(cx: &mut Context, source: String, flags: String) -> Completion<Value> {
    validate_regexp_flags(&flags)?;
    let proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::RegExpPrototype)
        .ok_or_else(|| JsError::internal("missing RegExp.prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    cx.heap_mut()
        .get_mut(object)?
        .add_slot(InternalSlot::RegExpData {
            source: source.clone(),
            flags: flags.clone(),
        });
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from("lastIndex"),
        Descriptor::data(Value::Number(0.0), true, false, false),
    )?;
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from("source"),
        Descriptor::data(Value::String(source), false, false, true),
    )?;
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from("flags"),
        Descriptor::data(Value::String(flags), false, false, true),
    )?;
    Ok(Value::Object(object))
}

pub fn regexp_source_flags(cx: &Context, object: ObjectRef) -> Completion<(String, String)> {
    for slot in &cx.heap().get(object)?.internal_slots {
        if let InternalSlot::RegExpData { source, flags } = slot {
            return Ok((source.clone(), flags.clone()));
        }
    }
    Err(JsError::type_error(
        "receiver does not have required RegExp internal slot",
    ))
}

pub(crate) fn validate_regexp_flags(flags: &str) -> Completion<()> {
    let mut seen = Vec::new();
    for ch in flags.chars() {
        if !"dgimsuvy".contains(ch) {
            return Err(JsError::syntax("invalid regular expression flags"));
        }
        if seen.contains(&ch) {
            return Err(JsError::syntax("duplicate regular expression flag"));
        }
        seen.push(ch);
    }
    Ok(())
}
