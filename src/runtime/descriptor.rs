use super::{
    Completion, Context, InternalMethods, JsError, JsObject, ObjectRef, PropertyKey, SameValue,
    Value,
};
use num_traits::Zero;

#[derive(Clone, Debug)]
pub struct Descriptor {
    pub value: Option<Value>,
    pub writable: Option<bool>,
    pub get: Option<Value>,
    pub set: Option<Value>,
    pub enumerable: Option<bool>,
    pub configurable: Option<bool>,
}

impl Descriptor {
    pub fn empty() -> Self {
        Self {
            value: None,
            writable: None,
            get: None,
            set: None,
            enumerable: None,
            configurable: None,
        }
    }

    pub fn data(value: Value, writable: bool, enumerable: bool, configurable: bool) -> Self {
        Self {
            value: Some(value),
            writable: Some(writable),
            get: None,
            set: None,
            enumerable: Some(enumerable),
            configurable: Some(configurable),
        }
    }

    pub fn accessor(
        get: Option<Value>,
        set: Option<Value>,
        enumerable: bool,
        configurable: bool,
    ) -> Self {
        Self {
            value: None,
            writable: None,
            get,
            set,
            enumerable: Some(enumerable),
            configurable: Some(configurable),
        }
    }

    pub fn is_accessor_descriptor(&self) -> bool {
        self.get.is_some() || self.set.is_some()
    }

    pub fn is_data_descriptor(&self) -> bool {
        self.value.is_some() || self.writable.is_some()
    }

    pub fn is_generic_descriptor(&self) -> bool {
        !self.is_accessor_descriptor() && !self.is_data_descriptor()
    }

    pub fn complete_data(mut self) -> Self {
        self.value.get_or_insert(Value::Undefined);
        self.writable.get_or_insert(false);
        self.enumerable.get_or_insert(false);
        self.configurable.get_or_insert(false);
        self
    }

    pub fn complete_accessor(mut self) -> Self {
        self.get.get_or_insert(Value::Undefined);
        self.set.get_or_insert(Value::Undefined);
        self.enumerable.get_or_insert(false);
        self.configurable.get_or_insert(false);
        self
    }

    pub fn configurable(&self) -> bool {
        self.configurable.unwrap_or(false)
    }

    pub fn enumerable(&self) -> bool {
        self.enumerable.unwrap_or(false)
    }

    pub fn writable(&self) -> bool {
        self.writable.unwrap_or(false)
    }

    pub fn validate_shape(&self) -> Completion<()> {
        if self.is_accessor_descriptor() && self.is_data_descriptor() {
            return Err(JsError::type_error(
                "property descriptor cannot be both data and accessor",
            ));
        }
        Ok(())
    }

    pub fn merged_with_current(&self, current: &Descriptor) -> Descriptor {
        if self.is_accessor_descriptor() {
            return Descriptor {
                value: None,
                writable: None,
                get: self.get.clone().or_else(|| {
                    if current.is_accessor_descriptor() {
                        current.get.clone()
                    } else {
                        Some(Value::Undefined)
                    }
                }),
                set: self.set.clone().or_else(|| {
                    if current.is_accessor_descriptor() {
                        current.set.clone()
                    } else {
                        Some(Value::Undefined)
                    }
                }),
                enumerable: self.enumerable.or(current.enumerable),
                configurable: self.configurable.or(current.configurable),
            };
        }
        if self.is_data_descriptor() {
            return Descriptor {
                value: self.value.clone().or_else(|| {
                    if current.is_data_descriptor() {
                        current.value.clone()
                    } else {
                        Some(Value::Undefined)
                    }
                }),
                writable: self.writable.or_else(|| {
                    if current.is_data_descriptor() {
                        current.writable
                    } else {
                        Some(false)
                    }
                }),
                get: None,
                set: None,
                enumerable: self.enumerable.or(current.enumerable),
                configurable: self.configurable.or(current.configurable),
            };
        }
        Descriptor {
            value: self.value.clone().or_else(|| current.value.clone()),
            writable: self.writable.or(current.writable),
            get: self.get.clone().or_else(|| current.get.clone()),
            set: self.set.clone().or_else(|| current.set.clone()),
            enumerable: self.enumerable.or(current.enumerable),
            configurable: self.configurable.or(current.configurable),
        }
    }
}

pub fn validate_and_apply(
    current: Option<&Descriptor>,
    extensible: bool,
    desc: &Descriptor,
) -> Completion<bool> {
    desc.validate_shape()?;

    let Some(current) = current else {
        return Ok(extensible);
    };

    if current.configurable() {
        return Ok(true);
    }

    if desc.configurable == Some(true) {
        return Ok(false);
    }

    if let Some(enumerable) = desc.enumerable {
        if enumerable != current.enumerable() {
            return Ok(false);
        }
    }

    let current_is_data = current.is_data_descriptor();
    let desc_is_data = desc.is_data_descriptor();
    let current_is_accessor = current.is_accessor_descriptor();
    let desc_is_accessor = desc.is_accessor_descriptor();

    if !desc.is_generic_descriptor() {
        if current_is_data != desc_is_data || current_is_accessor != desc_is_accessor {
            return Ok(false);
        }
    }

    if current_is_data && desc_is_data && !current.writable() {
        if desc.writable == Some(true) {
            return Ok(false);
        }
        if let Some(value) = &desc.value {
            let current_value = current.value.as_ref().unwrap_or(&Value::Undefined);
            if !SameValue(value, current_value) {
                return Ok(false);
            }
        }
    }

    if current_is_accessor && desc_is_accessor {
        if let Some(get) = &desc.get {
            let current_get = current.get.as_ref().unwrap_or(&Value::Undefined);
            if !SameValue(get, current_get) {
                return Ok(false);
            }
        }
        if let Some(set) = &desc.set {
            let current_set = current.set.as_ref().unwrap_or(&Value::Undefined);
            if !SameValue(set, current_set) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

#[allow(non_snake_case)]
pub fn ToPropertyDescriptor(cx: &mut Context, value: Value) -> Completion<Descriptor> {
    let Value::Object(object) = value else {
        return Err(JsError::type_error("property descriptor must be an object"));
    };

    let mut desc = Descriptor::empty();

    if object.has_property(cx, &PropertyKey::from("enumerable"))? {
        desc.enumerable = Some(to_boolean(object.get(
            cx,
            &PropertyKey::from("enumerable"),
            Value::Object(object),
        )?));
    }
    if object.has_property(cx, &PropertyKey::from("configurable"))? {
        desc.configurable = Some(to_boolean(object.get(
            cx,
            &PropertyKey::from("configurable"),
            Value::Object(object),
        )?));
    }
    if object.has_property(cx, &PropertyKey::from("value"))? {
        desc.value = Some(object.get(cx, &PropertyKey::from("value"), Value::Object(object))?);
    }
    if object.has_property(cx, &PropertyKey::from("writable"))? {
        desc.writable = Some(to_boolean(object.get(
            cx,
            &PropertyKey::from("writable"),
            Value::Object(object),
        )?));
    }
    if object.has_property(cx, &PropertyKey::from("get"))? {
        let getter = object.get(cx, &PropertyKey::from("get"), Value::Object(object))?;
        if !matches!(getter, Value::Undefined) && !cx.is_callable(&getter)? {
            return Err(JsError::type_error(
                "descriptor getter must be callable or undefined",
            ));
        }
        desc.get = Some(getter);
    }
    if object.has_property(cx, &PropertyKey::from("set"))? {
        let setter = object.get(cx, &PropertyKey::from("set"), Value::Object(object))?;
        if !matches!(setter, Value::Undefined) && !cx.is_callable(&setter)? {
            return Err(JsError::type_error(
                "descriptor setter must be callable or undefined",
            ));
        }
        desc.set = Some(setter);
    }

    desc.validate_shape()?;
    Ok(desc)
}

#[allow(non_snake_case)]
pub fn FromPropertyDescriptor(cx: &mut Context, desc: Option<Descriptor>) -> Completion<Value> {
    let Some(desc) = desc else {
        return Ok(Value::Undefined);
    };

    let proto = cx
        .realm()?
        .intrinsics
        .get(super::IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));

    if let Some(value) = desc.value {
        create_data(cx, object, "value", value)?;
    }
    if let Some(writable) = desc.writable {
        create_data(cx, object, "writable", Value::Boolean(writable))?;
    }
    if let Some(get) = desc.get {
        create_data(cx, object, "get", get)?;
    }
    if let Some(set) = desc.set {
        create_data(cx, object, "set", set)?;
    }
    if let Some(enumerable) = desc.enumerable {
        create_data(cx, object, "enumerable", Value::Boolean(enumerable))?;
    }
    if let Some(configurable) = desc.configurable {
        create_data(cx, object, "configurable", Value::Boolean(configurable))?;
    }

    Ok(Value::Object(object))
}

fn create_data(cx: &mut Context, object: ObjectRef, name: &str, value: Value) -> Completion<()> {
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from(name),
        Descriptor::data(value, true, true, true),
    )
}

fn to_boolean(value: Value) -> bool {
    match value {
        Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => false,
        Value::Undefined | Value::Null => false,
        Value::Boolean(value) => value,
        Value::Number(value) => value != 0.0 && !value.is_nan(),
        Value::String(value) => !value.is_empty(),
        Value::BigInt(value) => !value.is_zero(),
        Value::Symbol(_) | Value::Object(_) => true,
    }
}
