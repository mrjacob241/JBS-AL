use super::{
    Brand, Completion, Context, InternalMethods, InternalSlot, JsError, JsObject, ObjectRef,
    PropertyKey, SameValueZero, Value,
};
use num_traits::Zero;

pub const SYMBOL_ITERATOR_ID: u64 = 1;
pub const SYMBOL_TO_STRING_TAG_ID: u64 = 2;
pub const SYMBOL_UNSCOPABLES_ID: u64 = 3;
pub const SYMBOL_REPLACE_ID: u64 = 4;
pub const SYMBOL_SPECIES_ID: u64 = 5;
pub const SYMBOL_TO_PRIMITIVE_ID: u64 = 6;
pub const SYMBOL_IS_CONCAT_SPREADABLE_ID: u64 = 7;
pub const SYMBOL_HAS_INSTANCE_ID: u64 = 8;
pub const SYMBOL_DISPOSE_ID: u64 = 9;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PrimitiveHint {
    Number,
    String,
}

pub struct ArgView {
    value: Value,
}

impl ArgView {
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    pub fn value(&self) -> &Value {
        &self.value
    }

    pub fn to_boolean(&self) -> bool {
        match &self.value {
            Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => false,
            Value::Undefined | Value::Null => false,
            Value::Boolean(value) => *value,
            Value::Number(value) => *value != 0.0 && !value.is_nan(),
            Value::String(value) => !value.is_empty(),
            Value::BigInt(value) => !value.is_zero(),
            Value::Symbol(_) | Value::Object(_) => true,
        }
    }

    pub fn to_object(&self, cx: &mut Context) -> Completion<ObjectRef> {
        match self.value.clone() {
            Value::Object(object) => Ok(object),
            Value::Undefined | Value::Null => Err(JsError::type_error(
                "cannot convert undefined or null to object",
            )),
            primitive => {
                let proto_id = match primitive {
                    Value::Boolean(_) => super::IntrinsicId::BooleanPrototype,
                    Value::Number(_) => super::IntrinsicId::NumberPrototype,
                    Value::BigInt(_) => super::IntrinsicId::BigIntPrototype,
                    Value::String(_) => super::IntrinsicId::StringPrototype,
                    Value::Symbol(_) => super::IntrinsicId::SymbolPrototype,
                    _ => super::IntrinsicId::ObjectPrototype,
                };
                let proto = cx
                    .realm()?
                    .intrinsics
                    .get(proto_id)
                    .or_else(|| {
                        cx.realm()
                            .ok()?
                            .intrinsics
                            .get(super::IntrinsicId::ObjectPrototype)
                    })
                    .ok_or_else(|| {
                        JsError::internal("missing primitive wrapper prototype intrinsic")
                    })?;
                let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
                let object_data = cx.heap_mut().get_mut(object)?;
                object_data.primitive_value = Some(primitive.clone());
                object_data.add_slot(InternalSlot::PrimitiveValue(primitive.clone()));
                if let Value::String(text) = self.value.clone() {
                    for (index, ch) in text.chars().enumerate() {
                        object.define_own_property_or_throw(
                            cx,
                            PropertyKey::array_index(index as u64),
                            super::Descriptor::data(
                                Value::String(ch.to_string()),
                                false,
                                true,
                                false,
                            ),
                        )?;
                    }
                    object.define_own_property_or_throw(
                        cx,
                        PropertyKey::from("length"),
                        super::Descriptor::data(
                            Value::Number(text.chars().count() as f64),
                            false,
                            false,
                            false,
                        ),
                    )?;
                }
                Ok(object)
            }
        }
    }

    pub fn to_primitive(&self, cx: &mut Context) -> Completion<Value> {
        self.to_primitive_with_hint(cx, PrimitiveHint::Number)
    }

    fn to_primitive_with_hint(&self, cx: &mut Context, hint: PrimitiveHint) -> Completion<Value> {
        let Value::Object(object) = self.value.clone() else {
            return Ok(self.value.clone());
        };

        let exotic_to_primitive = object.get(
            cx,
            &PropertyKey::Symbol(SYMBOL_TO_PRIMITIVE_ID),
            Value::Object(object),
        )?;
        if !matches!(exotic_to_primitive, Value::Undefined | Value::Null) {
            if !cx.is_callable(&exotic_to_primitive)? {
                return Err(JsError::type_error(
                    "Symbol.toPrimitive method is not callable",
                ));
            }
            let hint_value = match hint {
                PrimitiveHint::String => Value::String("string".to_owned()),
                PrimitiveHint::Number => Value::String("number".to_owned()),
            };
            let result = cx.call_mut(exotic_to_primitive, Value::Object(object), &[hint_value])?;
            if !result.is_object() {
                return Ok(result);
            }
            return Err(JsError::type_error(
                "Symbol.toPrimitive method returned an object",
            ));
        }

        if let Some(primitive) = primitive_wrapper_value(cx, object)? {
            return Ok(primitive);
        }

        let method_names = match hint {
            PrimitiveHint::String => ["toString", "valueOf"],
            PrimitiveHint::Number => ["valueOf", "toString"],
        };
        for name in method_names {
            let method = object.get(cx, &PropertyKey::from(name), Value::Object(object))?;
            if matches!(method, Value::Undefined | Value::Null) {
                continue;
            }
            if !cx.is_callable(&method)? {
                continue;
            }
            let result = cx.call_mut(method, Value::Object(object), &[])?;
            if !result.is_object() {
                return Ok(result);
            }
        }

        Err(JsError::type_error(
            "cannot convert object to primitive value",
        ))
    }

    pub fn to_property_key(&self, cx: &mut Context) -> Completion<PropertyKey> {
        match self.to_primitive_with_hint(cx, PrimitiveHint::String)? {
            Value::String(value) => Ok(PropertyKey::from(value)),
            Value::Symbol(value) => Ok(PropertyKey::Symbol(value)),
            Value::Number(value) => Ok(PropertyKey::from(number_to_property_string(value))),
            Value::Boolean(value) => Ok(PropertyKey::from(if value { "true" } else { "false" })),
            Value::Undefined => Ok(PropertyKey::from("undefined")),
            Value::Null => Ok(PropertyKey::from("null")),
            Value::BigInt(value) => Ok(PropertyKey::from(value.to_string())),
            Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => Err(
                JsError::type_error("cannot convert internal value to property key"),
            ),
            Value::Object(_) => unreachable!("ToPrimitive returned an object"),
        }
    }

    pub fn to_number(&self, cx: &mut Context) -> Completion<f64> {
        match self.to_primitive(cx)? {
            Value::Number(value) => Ok(value),
            Value::Boolean(true) => Ok(1.0),
            Value::Boolean(false) | Value::Null => Ok(0.0),
            Value::String(value) => Ok(string_to_number(&value)),
            Value::Undefined => Ok(f64::NAN),
            other => Err(JsError::type_error(format!(
                "cannot convert {} to number",
                other.type_name()
            ))),
        }
    }

    pub fn to_string(&self, cx: &mut Context) -> Completion<String> {
        match self.to_primitive_with_hint(cx, PrimitiveHint::String)? {
            Value::Undefined => Ok("undefined".to_owned()),
            Value::Null => Ok("null".to_owned()),
            Value::Boolean(value) => Ok(if value { "true" } else { "false" }.to_owned()),
            Value::String(value) => Ok(value),
            Value::Number(value) => Ok(number_to_property_string(value)),
            Value::BigInt(value) => Ok(value.to_string()),
            other => Err(JsError::type_error(format!(
                "cannot convert {} to string",
                other.type_name()
            ))),
        }
    }
}

fn string_to_number(value: &str) -> f64 {
    let text = value.trim();
    if text.is_empty() {
        return 0.0;
    }
    let (sign, unsigned) = if let Some(rest) = text.strip_prefix('-') {
        (-1.0, rest)
    } else if let Some(rest) = text.strip_prefix('+') {
        (1.0, rest)
    } else {
        (1.0, text)
    };
    if let Some(rest) = unsigned
        .strip_prefix("0b")
        .or_else(|| unsigned.strip_prefix("0B"))
    {
        return i128::from_str_radix(rest, 2)
            .map(|value| sign * value as f64)
            .unwrap_or(f64::NAN);
    }
    if let Some(rest) = unsigned
        .strip_prefix("0o")
        .or_else(|| unsigned.strip_prefix("0O"))
    {
        return i128::from_str_radix(rest, 8)
            .map(|value| sign * value as f64)
            .unwrap_or(f64::NAN);
    }
    if let Some(rest) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        return i128::from_str_radix(rest, 16)
            .map(|value| sign * value as f64)
            .unwrap_or(f64::NAN);
    }
    text.parse::<f64>().unwrap_or(f64::NAN)
}

pub struct ReceiverView {
    value: Value,
}

impl ReceiverView {
    pub fn new(value: Value) -> Self {
        Self { value }
    }

    pub fn generic_object(&self, cx: &mut Context) -> Completion<ObjectRef> {
        ArgView::new(self.value.clone()).to_object(cx)
    }

    pub fn require_brand(&self, cx: &mut Context, brand: Brand) -> Completion<ObjectRef> {
        let object = self.generic_object(cx)?;
        if cx.heap().get(object)?.has_brand(brand) {
            Ok(object)
        } else {
            Err(JsError::type_error(
                "receiver does not have required internal slot",
            ))
        }
    }
}

pub fn primitive_wrapper_value(cx: &Context, object: ObjectRef) -> Completion<Option<Value>> {
    if let Some(value) = cx.heap().get(object)?.primitive_value.clone() {
        return Ok(Some(value));
    }

    let intrinsics = &cx.realm()?.intrinsics;
    if intrinsics.get(super::IntrinsicId::BooleanPrototype) == Some(object) {
        return Ok(Some(Value::Boolean(false)));
    }
    if intrinsics.get(super::IntrinsicId::NumberPrototype) == Some(object) {
        return Ok(Some(Value::Number(0.0)));
    }
    if intrinsics.get(super::IntrinsicId::StringPrototype) == Some(object) {
        return Ok(Some(Value::String(String::new())));
    }
    Ok(None)
}

pub fn number_to_property_string(value: f64) -> String {
    if SameValueZero(&Value::Number(value), &Value::Number(-0.0)) {
        return "0".to_owned();
    }
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }
    let abs = value.abs();
    if abs >= 1e21 || (abs != 0.0 && abs < 1e-6) {
        return number_to_exponential_string(value);
    }
    if value.fract() == 0.0 {
        return format!("{value:.0}");
    }
    value.to_string()
}

fn number_to_exponential_string(value: f64) -> String {
    let raw = format!("{value:e}");
    let Some((mantissa, exponent)) = raw.split_once('e') else {
        return raw;
    };
    let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
    let exponent = exponent.parse::<i32>().unwrap_or(0);
    if exponent >= 0 {
        format!("{mantissa}e+{exponent}")
    } else {
        format!("{mantissa}e{exponent}")
    }
}

#[derive(Clone)]
pub struct IteratorRecord {
    pub iterator: Value,
    pub next_method: Value,
}

pub fn get_iterator(cx: &mut Context, value: Value) -> Completion<IteratorRecord> {
    if matches!(value, Value::Undefined | Value::Null) {
        return Err(JsError::type_error("value is not iterable"));
    }
    let object = ArgView::new(value.clone()).to_object(cx)?;
    let method = object.get(cx, &PropertyKey::Symbol(SYMBOL_ITERATOR_ID), value.clone())?;
    if !cx.is_callable(&method)? {
        return Err(JsError::type_error("value is not iterable"));
    }
    let iterator = cx.call_mut(method, value, &[])?;
    let Value::Object(iterator_object) = iterator.clone() else {
        return Err(JsError::type_error("iterator method returned non-object"));
    };
    let next_method = iterator_object.get(cx, &PropertyKey::from("next"), iterator.clone())?;
    if !cx.is_callable(&next_method)? {
        return Err(JsError::type_error("iterator next method is not callable"));
    }
    Ok(IteratorRecord {
        iterator,
        next_method,
    })
}

pub fn iterator_next(cx: &mut Context, record: &IteratorRecord) -> Completion<Value> {
    let result = cx.call_mut(record.next_method.clone(), record.iterator.clone(), &[])?;
    if !result.is_object() {
        return Err(JsError::type_error("iterator result is not an object"));
    }
    Ok(result)
}

pub fn iterator_complete(cx: &mut Context, result: &Value) -> Completion<bool> {
    let Value::Object(object) = result else {
        return Err(JsError::type_error("iterator result is not an object"));
    };
    Ok(ArgView::new(object.get(cx, &PropertyKey::from("done"), result.clone())?).to_boolean())
}

pub fn iterator_value(cx: &mut Context, result: &Value) -> Completion<Value> {
    let Value::Object(object) = result else {
        return Err(JsError::type_error("iterator result is not an object"));
    };
    object.get(cx, &PropertyKey::from("value"), result.clone())
}

pub fn iterator_step_value(cx: &mut Context, record: &IteratorRecord) -> Completion<Option<Value>> {
    let result = iterator_next(cx, record)?;
    if iterator_complete(cx, &result)? {
        return Ok(None);
    }
    Ok(Some(iterator_value(cx, &result)?))
}

pub fn iterator_close(cx: &mut Context, record: &IteratorRecord) -> Completion<()> {
    let Value::Object(iterator) = record.iterator.clone() else {
        return Err(JsError::type_error("iterator record target is not object"));
    };
    iterator_close_object(cx, iterator)
}

pub fn iterator_close_value(cx: &mut Context, iterator: Value) -> Completion<()> {
    let Value::Object(iterator) = iterator else {
        return Err(JsError::type_error("iterator target is not object"));
    };
    iterator_close_object(cx, iterator)
}

fn iterator_close_object(cx: &mut Context, iterator: ObjectRef) -> Completion<()> {
    let return_method = iterator.get(cx, &PropertyKey::from("return"), Value::Object(iterator))?;
    if matches!(return_method, Value::Undefined | Value::Null) {
        return Ok(());
    }
    if !cx.is_callable(&return_method)? {
        return Err(JsError::type_error(
            "iterator return method is not callable",
        ));
    }
    let result = cx.call_mut(return_method, Value::Object(iterator), &[])?;
    if !result.is_object() {
        return Err(JsError::type_error(
            "iterator return method returned non-object",
        ));
    }
    Ok(())
}

pub fn iterator_close_error(cx: &mut Context, record: &IteratorRecord, error: JsError) -> JsError {
    match iterator_close(cx, record) {
        Ok(()) => error,
        Err(close_error) => close_error,
    }
}
