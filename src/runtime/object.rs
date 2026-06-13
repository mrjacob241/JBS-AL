use std::collections::HashMap;

use super::descriptor::{validate_and_apply, ToPropertyDescriptor};
use super::{
    ArgView, Completion, Context, Descriptor, FromPropertyDescriptor, FunctionData, JsError,
    LengthOfArrayLike, PropertyKey, SameValue, Value,
};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct ObjectRef(pub(crate) usize);

impl ObjectRef {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub enum ObjectKind {
    Ordinary,
    Array,
    Arguments,
    Function(FunctionData),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionEntry {
    pub key: Value,
    pub value: Value,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CollectionKind {
    Map,
    Set,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CollectionIteratorKind {
    Key,
    Value,
    KeyAndValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InternalSlot {
    PrimitiveValue(Value),
    Array,
    Function,
    BooleanData,
    NumberData,
    StringData,
    SymbolData,
    ErrorData,
    IteratorData {
        target: Value,
        kind: String,
        index: u32,
    },
    MapData {
        entries: Vec<CollectionEntry>,
    },
    SetData {
        entries: Vec<CollectionEntry>,
    },
    WeakMapData {
        entries: Vec<CollectionEntry>,
    },
    WeakSetData {
        entries: Vec<CollectionEntry>,
    },
    CollectionIteratorData {
        target: Value,
        collection_kind: CollectionKind,
        iteration_kind: CollectionIteratorKind,
        index: u32,
    },
    ProxyData {
        target: Option<ObjectRef>,
        handler: Option<ObjectRef>,
        callable: bool,
        constructible: bool,
    },
    ConcatIteratorData {
        iterables: Vec<Value>,
        outer_index: usize,
        active_iterator: Option<Value>,
        active_next_method: Option<Value>,
    },
    RegExpData {
        source: String,
        flags: String,
    },
    DateValue,
    ArrayBufferData,
    TypedArrayData,
    PromiseState,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Brand {
    Array,
    Function,
    PrimitiveWrapper,
    Iterator,
    Map,
    Set,
    WeakMap,
    WeakSet,
    RegExp,
    Date,
    ArrayBuffer,
    TypedArray,
    Promise,
    Error,
}

#[derive(Clone, Debug)]
pub struct JsObject {
    pub prototype: Option<ObjectRef>,
    pub extensible: bool,
    pub sealed: bool,
    pub frozen: bool,
    pub primitive_value: Option<Value>,
    pub internal_slots: Vec<InternalSlot>,
    pub properties: HashMap<PropertyKey, Descriptor>,
    pub property_order: Vec<PropertyKey>,
    pub kind: ObjectKind,
}

impl JsObject {
    pub fn ordinary(prototype: Option<ObjectRef>) -> Self {
        Self {
            prototype,
            extensible: true,
            sealed: false,
            frozen: false,
            primitive_value: None,
            internal_slots: Vec::new(),
            properties: HashMap::new(),
            property_order: Vec::new(),
            kind: ObjectKind::Ordinary,
        }
    }

    pub fn function(prototype: Option<ObjectRef>, data: FunctionData) -> Self {
        Self {
            prototype,
            extensible: true,
            sealed: false,
            frozen: false,
            primitive_value: None,
            internal_slots: vec![InternalSlot::Function],
            properties: HashMap::new(),
            property_order: Vec::new(),
            kind: ObjectKind::Function(data),
        }
    }

    pub fn array(prototype: Option<ObjectRef>) -> Self {
        Self {
            prototype,
            extensible: true,
            sealed: false,
            frozen: false,
            primitive_value: None,
            internal_slots: vec![InternalSlot::Array],
            properties: HashMap::new(),
            property_order: Vec::new(),
            kind: ObjectKind::Array,
        }
    }

    pub fn arguments(prototype: Option<ObjectRef>) -> Self {
        Self {
            prototype,
            extensible: true,
            sealed: false,
            frozen: false,
            primitive_value: None,
            internal_slots: Vec::new(),
            properties: HashMap::new(),
            property_order: Vec::new(),
            kind: ObjectKind::Arguments,
        }
    }

    pub fn has_brand(&self, brand: Brand) -> bool {
        match brand {
            Brand::Array => {
                matches!(self.kind, ObjectKind::Array)
                    || self.internal_slots.contains(&InternalSlot::Array)
            }
            Brand::Function => {
                matches!(self.kind, ObjectKind::Function(_))
                    || self.internal_slots.contains(&InternalSlot::Function)
            }
            Brand::PrimitiveWrapper => {
                self.primitive_value.is_some()
                    || self
                        .internal_slots
                        .iter()
                        .any(|slot| matches!(slot, InternalSlot::PrimitiveValue(_)))
            }
            Brand::Iterator => self
                .internal_slots
                .iter()
                .any(|slot| matches!(slot, InternalSlot::IteratorData { .. })),
            Brand::Map => self
                .internal_slots
                .iter()
                .any(|slot| matches!(slot, InternalSlot::MapData { .. })),
            Brand::Set => self
                .internal_slots
                .iter()
                .any(|slot| matches!(slot, InternalSlot::SetData { .. })),
            Brand::WeakMap => self
                .internal_slots
                .iter()
                .any(|slot| matches!(slot, InternalSlot::WeakMapData { .. })),
            Brand::WeakSet => self
                .internal_slots
                .iter()
                .any(|slot| matches!(slot, InternalSlot::WeakSetData { .. })),
            Brand::RegExp => self
                .internal_slots
                .iter()
                .any(|slot| matches!(slot, InternalSlot::RegExpData { .. })),
            Brand::Date => self.internal_slots.contains(&InternalSlot::DateValue),
            Brand::ArrayBuffer => self.internal_slots.contains(&InternalSlot::ArrayBufferData),
            Brand::TypedArray => self.internal_slots.contains(&InternalSlot::TypedArrayData),
            Brand::Promise => self.internal_slots.contains(&InternalSlot::PromiseState),
            Brand::Error => self.internal_slots.contains(&InternalSlot::ErrorData),
        }
    }

    pub fn add_slot(&mut self, slot: InternalSlot) {
        if !self.internal_slots.contains(&slot) {
            self.internal_slots.push(slot);
        }
    }
}

pub trait InternalMethods {
    fn get_own_property(
        &self,
        cx: &mut Context,
        key: &PropertyKey,
    ) -> Completion<Option<Descriptor>>;
    fn define_own_property(
        &self,
        cx: &mut Context,
        key: PropertyKey,
        desc: Descriptor,
    ) -> Completion<bool>;
    fn has_property(&self, cx: &mut Context, key: &PropertyKey) -> Completion<bool>;
    fn get(&self, cx: &mut Context, key: &PropertyKey, receiver: Value) -> Completion<Value>;
    fn set(
        &self,
        cx: &mut Context,
        key: PropertyKey,
        value: Value,
        receiver: Value,
    ) -> Completion<bool>;
    fn delete(&self, cx: &mut Context, key: &PropertyKey) -> Completion<bool>;
    fn own_property_keys(&self, cx: &mut Context) -> Completion<Vec<PropertyKey>>;
    fn get_prototype_of(&self, cx: &mut Context) -> Completion<Option<ObjectRef>>;
    fn set_prototype_of(&self, cx: &mut Context, proto: Option<ObjectRef>) -> Completion<bool>;
    fn prevent_extensions(&self, cx: &mut Context) -> Completion<bool>;
    fn is_extensible(&self, cx: &mut Context) -> Completion<bool>;
    fn define_own_property_or_throw(
        &self,
        cx: &mut Context,
        key: PropertyKey,
        desc: Descriptor,
    ) -> Completion<()>;
}

impl InternalMethods for ObjectRef {
    fn get_own_property(
        &self,
        cx: &mut Context,
        key: &PropertyKey,
    ) -> Completion<Option<Descriptor>> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "getOwnPropertyDescriptor")? {
                let result = cx.call_mut(
                    trap,
                    Value::Object(handler),
                    &[Value::Object(target), property_key_to_value(key.clone())],
                )?;
                let target_desc = target.get_own_property(cx, key)?;
                let extensible = target.is_extensible(cx)?;
                if matches!(result, Value::Undefined) {
                    if let Some(desc) = target_desc {
                        if !desc.configurable() || !extensible {
                            return Err(JsError::type_error(
                                "proxy getOwnPropertyDescriptor trap cannot hide target property",
                            ));
                        }
                    }
                    return Ok(None);
                }
                let Value::Object(_) = result else {
                    return Err(JsError::type_error(
                        "proxy getOwnPropertyDescriptor trap must return object or undefined",
                    ));
                };
                let trap_desc = ToPropertyDescriptor(cx, result)?;
                validate_proxy_get_own_property_descriptor_invariants(
                    &target_desc,
                    extensible,
                    &trap_desc,
                )?;
                return Ok(Some(complete_property_descriptor(trap_desc)));
            }
            return target.get_own_property(cx, key);
        }
        Ok(cx.heap().get(*self)?.properties.get(key).cloned())
    }

    fn define_own_property(
        &self,
        cx: &mut Context,
        key: PropertyKey,
        desc: Descriptor,
    ) -> Completion<bool> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "defineProperty")? {
                let desc_obj = FromPropertyDescriptor(cx, Some(desc.clone()))?;
                let result = cx.call_mut(
                    trap,
                    Value::Object(handler),
                    &[
                        Value::Object(target),
                        property_key_to_value(key.clone()),
                        desc_obj,
                    ],
                )?;
                if !to_boolean(result) {
                    return Ok(false);
                }
                validate_proxy_define_invariants(cx, target, &key, &desc)?;
                return Ok(true);
            }
            return target.define_own_property(cx, key, desc);
        }
        if is_array_length_key(cx, *self, &key)? {
            return array_set_length(cx, *self, desc);
        }
        ordinary_define_own_property(cx, *self, key, desc, true)
    }

    fn has_property(&self, cx: &mut Context, key: &PropertyKey) -> Completion<bool> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "has")? {
                let result = cx.call_mut(
                    trap,
                    Value::Object(handler),
                    &[Value::Object(target), property_key_to_value(key.clone())],
                )?;
                let boolean = to_boolean(result);
                if !boolean {
                    validate_proxy_has_invariants(cx, target, key)?;
                }
                return Ok(boolean);
            }
            return target.has_property(cx, key);
        }
        if self.get_own_property(cx, key)?.is_some() {
            return Ok(true);
        }
        if let Some(proto) = self.get_prototype_of(cx)? {
            return proto.has_property(cx, key);
        }
        Ok(false)
    }

    fn get(&self, cx: &mut Context, key: &PropertyKey, receiver: Value) -> Completion<Value> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "get")? {
                let result = cx.call_mut(
                    trap,
                    Value::Object(handler),
                    &[
                        Value::Object(target),
                        property_key_to_value(key.clone()),
                        receiver,
                    ],
                )?;
                validate_proxy_get_invariants(cx, target, key, &result)?;
                return Ok(result);
            }
            return target.get(cx, key, receiver);
        }
        if let Some(desc) = self.get_own_property(cx, key)? {
            if desc.is_data_descriptor() {
                return Ok(desc.value.unwrap_or(Value::Undefined));
            }
            let getter = desc.get.unwrap_or(Value::Undefined);
            if matches!(getter, Value::Undefined) {
                return Ok(Value::Undefined);
            }
            return cx.call_mut(getter, receiver, &[]);
        }
        if let Some(proto) = self.get_prototype_of(cx)? {
            return proto.get(cx, key, receiver);
        }
        Ok(Value::Undefined)
    }

    fn set(
        &self,
        cx: &mut Context,
        key: PropertyKey,
        value: Value,
        receiver: Value,
    ) -> Completion<bool> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "set")? {
                let result = cx.call_mut(
                    trap,
                    Value::Object(handler),
                    &[
                        Value::Object(target),
                        property_key_to_value(key.clone()),
                        value.clone(),
                        receiver,
                    ],
                )?;
                let boolean = to_boolean(result);
                if boolean {
                    validate_proxy_set_invariants(cx, target, &key, &value)?;
                }
                return Ok(boolean);
            }
            return target.set(cx, key, value, receiver);
        }
        ordinary_set(*self, cx, key, value, receiver)
    }

    fn delete(&self, cx: &mut Context, key: &PropertyKey) -> Completion<bool> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "deleteProperty")? {
                let result = cx.call_mut(
                    trap,
                    Value::Object(handler),
                    &[Value::Object(target), property_key_to_value(key.clone())],
                )?;
                let boolean = to_boolean(result);
                if boolean {
                    validate_proxy_delete_invariants(cx, target, key)?;
                }
                return Ok(boolean);
            }
            return target.delete(cx, key);
        }
        let current = self.get_own_property(cx, key)?;
        if let Some(desc) = current {
            if !desc.configurable() {
                return Ok(false);
            }
            let object = cx.heap_mut().get_mut(*self)?;
            object.properties.remove(key);
            object.property_order.retain(|existing| existing != key);
        }
        Ok(true)
    }

    fn own_property_keys(&self, cx: &mut Context) -> Completion<Vec<PropertyKey>> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "ownKeys")? {
                let result = cx.call_mut(trap, Value::Object(handler), &[Value::Object(target)])?;
                let trap_keys = create_property_key_list_from_array_like(cx, result)?;
                validate_proxy_own_keys_invariants(cx, target, &trap_keys)?;
                return Ok(trap_keys);
            }
            return target.own_property_keys(cx);
        }
        let object = cx.heap().get(*self)?;
        let mut indexes = Vec::new();
        let mut strings = Vec::new();
        let mut symbols = Vec::new();

        for key in &object.property_order {
            match key {
                PropertyKey::String(value) => {
                    if let Some(index) = array_index_key(value) {
                        indexes.push((index, key.clone()));
                    } else {
                        strings.push(key.clone());
                    }
                }
                PropertyKey::Symbol(_) => symbols.push(key.clone()),
            }
        }

        indexes.sort_by_key(|(index, _)| *index);
        Ok(indexes
            .into_iter()
            .map(|(_, key)| key)
            .chain(strings)
            .chain(symbols)
            .collect())
    }

    fn get_prototype_of(&self, cx: &mut Context) -> Completion<Option<ObjectRef>> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "getPrototypeOf")? {
                let result = cx.call_mut(trap, Value::Object(handler), &[Value::Object(target)])?;
                let proto = match result {
                    Value::Object(object) => Some(object),
                    Value::Null => None,
                    _ => {
                        return Err(JsError::type_error(
                            "proxy getPrototypeOf trap must return object or null",
                        ))
                    }
                };
                if !target.is_extensible(cx)? && target.get_prototype_of(cx)? != proto {
                    return Err(JsError::type_error(
                        "proxy getPrototypeOf trap cannot report a different non-extensible target prototype",
                    ));
                }
                return Ok(proto);
            }
            return target.get_prototype_of(cx);
        }
        Ok(cx.heap().get(*self)?.prototype)
    }

    fn set_prototype_of(&self, cx: &mut Context, proto: Option<ObjectRef>) -> Completion<bool> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "setPrototypeOf")? {
                let proto_value = proto.map(Value::Object).unwrap_or(Value::Null);
                let result = cx.call_mut(
                    trap,
                    Value::Object(handler),
                    &[Value::Object(target), proto_value],
                )?;
                let boolean = to_boolean(result);
                if boolean && !target.is_extensible(cx)? && target.get_prototype_of(cx)? != proto {
                    return Err(JsError::type_error(
                        "proxy setPrototypeOf trap cannot change non-extensible target prototype",
                    ));
                }
                return Ok(boolean);
            }
            return target.set_prototype_of(cx, proto);
        }
        if self.get_prototype_of(cx)? == proto {
            return Ok(true);
        }
        if !self.is_extensible(cx)? {
            return Ok(false);
        }
        if would_create_prototype_cycle(cx, *self, proto)? {
            return Ok(false);
        }
        cx.heap_mut().get_mut(*self)?.prototype = proto;
        Ok(true)
    }

    fn prevent_extensions(&self, cx: &mut Context) -> Completion<bool> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "preventExtensions")? {
                let result = cx.call_mut(trap, Value::Object(handler), &[Value::Object(target)])?;
                let boolean = to_boolean(result);
                if boolean {
                    validate_proxy_prevent_extensions_invariants(cx, target)?;
                }
                return Ok(boolean);
            }
            return target.prevent_extensions(cx);
        }
        cx.heap_mut().get_mut(*self)?.extensible = false;
        Ok(true)
    }

    fn is_extensible(&self, cx: &mut Context) -> Completion<bool> {
        if let Some((target, handler)) = proxy_parts(cx, *self)? {
            if let Some(trap) = proxy_trap(cx, handler, "isExtensible")? {
                let result = cx.call_mut(trap, Value::Object(handler), &[Value::Object(target)])?;
                let boolean = to_boolean(result);
                if boolean != target.is_extensible(cx)? {
                    return Err(JsError::type_error(
                        "proxy isExtensible trap must report target extensibility",
                    ));
                }
                return Ok(boolean);
            }
            return target.is_extensible(cx);
        }
        Ok(cx.heap().get(*self)?.extensible)
    }

    fn define_own_property_or_throw(
        &self,
        cx: &mut Context,
        key: PropertyKey,
        desc: Descriptor,
    ) -> Completion<()> {
        if self.define_own_property(cx, key.clone(), desc)? {
            Ok(())
        } else {
            Err(super::JsError::type_error(format!(
                "cannot define property {key}"
            )))
        }
    }
}

pub fn proxy_target(cx: &Context, object: ObjectRef) -> Completion<Option<ObjectRef>> {
    let object_data = cx.heap().get(object)?;
    for slot in &object_data.internal_slots {
        if let InternalSlot::ProxyData {
            target, handler, ..
        } = slot
        {
            if target.is_none() || handler.is_none() {
                return Err(JsError::type_error("proxy has been revoked"));
            }
            return Ok(*target);
        }
    }
    Ok(None)
}

fn proxy_parts(cx: &Context, object: ObjectRef) -> Completion<Option<(ObjectRef, ObjectRef)>> {
    let object_data = cx.heap().get(object)?;
    for slot in &object_data.internal_slots {
        if let InternalSlot::ProxyData {
            target, handler, ..
        } = slot
        {
            let (Some(target), Some(handler)) = (target, handler) else {
                return Err(JsError::type_error("proxy has been revoked"));
            };
            return Ok(Some((*target, *handler)));
        }
    }
    Ok(None)
}

fn proxy_trap(cx: &mut Context, handler: ObjectRef, name: &str) -> Completion<Option<Value>> {
    let trap = handler.get(cx, &PropertyKey::from(name), Value::Object(handler))?;
    if matches!(trap, Value::Undefined | Value::Null) {
        return Ok(None);
    }
    if !cx.is_callable(&trap)? {
        return Err(JsError::type_error("proxy trap is not callable"));
    }
    Ok(Some(trap))
}

fn property_key_to_value(key: PropertyKey) -> Value {
    match key {
        PropertyKey::String(value) => Value::String(value),
        PropertyKey::Symbol(symbol) => Value::Symbol(symbol),
    }
}

fn to_boolean(value: Value) -> bool {
    match value {
        Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => false,
        Value::Undefined | Value::Null => false,
        Value::Boolean(value) => value,
        Value::Number(value) => value != 0.0 && !value.is_nan(),
        Value::String(value) => !value.is_empty(),
        Value::BigInt(value) => value != 0,
        Value::Symbol(_) | Value::Object(_) => true,
    }
}

fn complete_property_descriptor(desc: Descriptor) -> Descriptor {
    if desc.is_accessor_descriptor() {
        desc.complete_accessor()
    } else {
        desc.complete_data()
    }
}

fn validate_proxy_get_own_property_descriptor_invariants(
    target_desc: &Option<Descriptor>,
    extensible: bool,
    trap_desc: &Descriptor,
) -> Completion<()> {
    if !validate_and_apply(target_desc.as_ref(), extensible, trap_desc)? {
        return Err(JsError::type_error(
            "proxy getOwnPropertyDescriptor trap violates target property invariant",
        ));
    }
    if trap_desc.configurable == Some(false) {
        match target_desc {
            Some(current) if !current.configurable() => {}
            _ => {
                return Err(JsError::type_error(
                    "proxy getOwnPropertyDescriptor trap cannot report non-configurable property",
                ))
            }
        }
    }
    if trap_desc.configurable == Some(false) && trap_desc.writable == Some(false) {
        match target_desc {
            Some(current)
                if !current.configurable() && current.is_data_descriptor() && !current.writable() => {}
            _ => {
                return Err(JsError::type_error(
                    "proxy getOwnPropertyDescriptor trap cannot report non-configurable non-writable property",
                ))
            }
        }
    }
    Ok(())
}

fn validate_proxy_get_invariants(
    cx: &mut Context,
    target: ObjectRef,
    key: &PropertyKey,
    result: &Value,
) -> Completion<()> {
    let Some(desc) = target.get_own_property(cx, key)? else {
        return Ok(());
    };
    if desc.configurable() {
        return Ok(());
    }
    if desc.is_data_descriptor() && !desc.writable() {
        if let Some(value) = desc.value.as_ref() {
            if !SameValue(result, value) {
                return Err(JsError::type_error(
                    "proxy get trap must return target value for non-writable non-configurable property",
                ));
            }
        }
    } else if desc.is_accessor_descriptor()
        && desc
            .get
            .as_ref()
            .is_none_or(|getter| matches!(getter, Value::Undefined))
        && !matches!(result, Value::Undefined)
    {
        return Err(JsError::type_error(
            "proxy get trap must return undefined for non-configurable accessor without getter",
        ));
    }
    Ok(())
}

fn validate_proxy_set_invariants(
    cx: &mut Context,
    target: ObjectRef,
    key: &PropertyKey,
    value: &Value,
) -> Completion<()> {
    let Some(desc) = target.get_own_property(cx, key)? else {
        return Ok(());
    };
    if desc.configurable() {
        return Ok(());
    }
    if desc.is_data_descriptor() && !desc.writable() {
        if let Some(current) = desc.value.as_ref() {
            if !SameValue(value, current) {
                return Err(JsError::type_error(
                    "proxy set trap cannot change non-writable non-configurable property",
                ));
            }
        }
    } else if desc.is_accessor_descriptor()
        && desc
            .set
            .as_ref()
            .is_none_or(|setter| matches!(setter, Value::Undefined))
    {
        return Err(JsError::type_error(
            "proxy set trap cannot set non-configurable accessor without setter",
        ));
    }
    Ok(())
}

fn validate_proxy_has_invariants(
    cx: &mut Context,
    target: ObjectRef,
    key: &PropertyKey,
) -> Completion<()> {
    let Some(desc) = target.get_own_property(cx, key)? else {
        return Ok(());
    };
    if !desc.configurable() {
        return Err(JsError::type_error(
            "proxy has trap cannot hide non-configurable property",
        ));
    }
    if !target.is_extensible(cx)? {
        return Err(JsError::type_error(
            "proxy has trap cannot hide property on non-extensible target",
        ));
    }
    Ok(())
}

fn validate_proxy_delete_invariants(
    cx: &mut Context,
    target: ObjectRef,
    key: &PropertyKey,
) -> Completion<()> {
    let Some(desc) = target.get_own_property(cx, key)? else {
        return Ok(());
    };
    if !desc.configurable() {
        return Err(JsError::type_error(
            "proxy deleteProperty trap cannot delete non-configurable property",
        ));
    }
    if !target.is_extensible(cx)? {
        return Err(JsError::type_error(
            "proxy deleteProperty trap cannot delete property from non-extensible target",
        ));
    }
    Ok(())
}

fn validate_proxy_prevent_extensions_invariants(
    cx: &mut Context,
    target: ObjectRef,
) -> Completion<()> {
    if target.is_extensible(cx)? {
        return Err(JsError::type_error(
            "proxy preventExtensions trap returned true for extensible target",
        ));
    }
    Ok(())
}

fn create_property_key_list_from_array_like(
    cx: &mut Context,
    value: Value,
) -> Completion<Vec<PropertyKey>> {
    let Value::Object(object) = value else {
        return Err(JsError::type_error(
            "proxy ownKeys trap must return an object",
        ));
    };
    let len = LengthOfArrayLike(cx, object)?;
    let mut keys = Vec::new();
    for index in 0..len {
        let key_value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        let key = match key_value {
            Value::String(value) => PropertyKey::String(value),
            Value::Symbol(symbol) => PropertyKey::Symbol(symbol),
            _ => {
                return Err(JsError::type_error(
                    "proxy ownKeys trap result must contain only strings or symbols",
                ))
            }
        };
        if keys.contains(&key) {
            return Err(JsError::type_error(
                "proxy ownKeys trap result contains duplicate key",
            ));
        }
        keys.push(key);
    }
    Ok(keys)
}

fn validate_proxy_own_keys_invariants(
    cx: &mut Context,
    target: ObjectRef,
    trap_keys: &[PropertyKey],
) -> Completion<()> {
    let target_keys = target.own_property_keys(cx)?;
    let mut configurable_keys = Vec::new();
    let mut non_configurable_keys = Vec::new();
    for key in target_keys {
        match target.get_own_property(cx, &key)? {
            Some(desc) if !desc.configurable() => non_configurable_keys.push(key),
            _ => configurable_keys.push(key),
        }
    }

    let extensible = target.is_extensible(cx)?;
    if extensible && non_configurable_keys.is_empty() {
        return Ok(());
    }

    for key in &non_configurable_keys {
        if !trap_keys.contains(key) {
            return Err(JsError::type_error(
                "proxy ownKeys trap result omitted non-configurable target key",
            ));
        }
    }

    if extensible {
        return Ok(());
    }

    for key in &configurable_keys {
        if !trap_keys.contains(key) {
            return Err(JsError::type_error(
                "proxy ownKeys trap result omitted non-extensible target key",
            ));
        }
    }
    if trap_keys.len() != non_configurable_keys.len() + configurable_keys.len() {
        return Err(JsError::type_error(
            "proxy ownKeys trap result included extra key for non-extensible target",
        ));
    }
    Ok(())
}

fn validate_proxy_define_invariants(
    cx: &mut Context,
    target: ObjectRef,
    key: &PropertyKey,
    desc: &Descriptor,
) -> Completion<()> {
    let current = target.get_own_property(cx, key)?;
    let extensible = target.is_extensible(cx)?;
    if current.is_none() && !extensible {
        return Err(JsError::type_error(
            "proxy defineProperty trap cannot add property to non-extensible target",
        ));
    }
    if !validate_and_apply(current.as_ref(), extensible, desc)? {
        return Err(JsError::type_error(
            "proxy defineProperty trap violates target property invariant",
        ));
    }
    if desc.configurable == Some(false) {
        match current.as_ref() {
            Some(current) if !current.configurable() => {}
            _ => {
                return Err(JsError::type_error(
                    "proxy defineProperty trap cannot create non-configurable property",
                ))
            }
        }
    }
    if desc.writable == Some(false) {
        if let Some(current) = current.as_ref() {
            if current.configurable()
                || !current.is_data_descriptor()
                || current.writable()
                || desc
                    .value
                    .as_ref()
                    .zip(current.value.as_ref())
                    .is_some_and(|(left, right)| !SameValue(left, right))
            {
                return Err(JsError::type_error(
                    "proxy defineProperty trap cannot create non-writable data mismatch",
                ));
            }
        }
    }
    Ok(())
}

fn ordinary_define_own_property(
    cx: &mut Context,
    object_ref: ObjectRef,
    key: PropertyKey,
    desc: Descriptor,
    update_array_indices: bool,
) -> Completion<bool> {
    let current = cx.heap().get(object_ref)?.properties.get(&key).cloned();
    let extensible = cx.heap().get(object_ref)?.extensible;
    if is_array_index_key(cx, object_ref, &key)?
        && !array_can_define_index(cx, object_ref, &key, &current)?
    {
        return Ok(false);
    }
    if !validate_and_apply(current.as_ref(), extensible, &desc)? {
        return Ok(false);
    }

    let final_desc = match current.as_ref() {
        Some(current) => desc.merged_with_current(current),
        None if desc.is_accessor_descriptor() => desc.complete_accessor(),
        None => desc.complete_data(),
    };

    let object = cx.heap_mut().get_mut(object_ref)?;
    let is_new = !object.properties.contains_key(&key);
    if is_new {
        object.property_order.push(key.clone());
    }
    object.properties.insert(key.clone(), final_desc);
    if matches!(object.kind, ObjectKind::Array) {
        if let PropertyKey::String(value) = &key {
            if update_array_indices {
                if let Some(index) = array_index_key(value) {
                    update_array_length_after_index(object, index);
                }
            } else if value == "length" {
                // ArraySetLength performs truncation and rollback itself.
            } else if let Some(index) = array_index_key(value) {
                update_array_length_after_index(object, index);
            }
        }
    }
    Ok(true)
}

fn array_set_length(cx: &mut Context, object: ObjectRef, desc: Descriptor) -> Completion<bool> {
    let Some(value) = desc.value.clone() else {
        return ordinary_define_own_property(cx, object, PropertyKey::from("length"), desc, false);
    };

    let new_len = to_array_length_for_set(cx, &value)?;
    let mut new_desc = desc;
    new_desc.value = Some(Value::Number(new_len as f64));

    let old_len_desc = cx
        .heap()
        .get(object)?
        .properties
        .get(&PropertyKey::from("length"))
        .cloned()
        .unwrap_or_else(|| Descriptor::data(Value::Number(0.0), true, false, false));
    let old_len = old_len_desc
        .value
        .as_ref()
        .and_then(|value| match value {
            Value::Number(value) if *value >= 0.0 => Some(*value as u32),
            _ => None,
        })
        .unwrap_or(0);

    if new_len >= old_len {
        return ordinary_define_own_property(
            cx,
            object,
            PropertyKey::from("length"),
            new_desc,
            false,
        );
    }

    if !old_len_desc.writable() {
        return Ok(false);
    }

    let new_writable = new_desc.writable.unwrap_or(true);
    if new_desc.writable == Some(false) {
        new_desc.writable = Some(true);
    }

    if !ordinary_define_own_property(cx, object, PropertyKey::from("length"), new_desc, false)? {
        return Ok(false);
    }

    for index in array_indices_at_or_above(cx, object, new_len)? {
        let key = PropertyKey::array_index(index as u64);
        if !object.delete(cx, &key)? {
            let restored_len = index + 1;
            let mut restore = Descriptor::empty();
            restore.value = Some(Value::Number(restored_len as f64));
            if !new_writable {
                restore.writable = Some(false);
            }
            ordinary_define_own_property(cx, object, PropertyKey::from("length"), restore, false)?;
            return Ok(false);
        }
    }

    if !new_writable {
        let mut non_writable = Descriptor::empty();
        non_writable.writable = Some(false);
        ordinary_define_own_property(cx, object, PropertyKey::from("length"), non_writable, false)?;
    }

    Ok(true)
}

fn array_indices_at_or_above(
    cx: &Context,
    object: ObjectRef,
    minimum: u32,
) -> Completion<Vec<u32>> {
    let mut indexes = Vec::new();
    for key in &cx.heap().get(object)?.property_order {
        let PropertyKey::String(value) = key else {
            continue;
        };
        if let Some(index) = array_index_key(value) {
            if index >= minimum {
                indexes.push(index);
            }
        }
    }
    indexes.sort_by(|left, right| right.cmp(left));
    Ok(indexes)
}

fn ordinary_set(
    target: ObjectRef,
    cx: &mut Context,
    key: PropertyKey,
    value: Value,
    receiver: Value,
) -> Completion<bool> {
    let own_desc = target.get_own_property(cx, &key)?;
    if own_desc.is_none() {
        if let Some(parent) = target.get_prototype_of(cx)? {
            return parent.set(cx, key, value, receiver);
        }
        return ordinary_set_with_own_descriptor(target, cx, key, value, receiver, None);
    }
    ordinary_set_with_own_descriptor(target, cx, key, value, receiver, own_desc)
}

fn ordinary_set_with_own_descriptor(
    _target: ObjectRef,
    cx: &mut Context,
    key: PropertyKey,
    value: Value,
    receiver: Value,
    own_desc: Option<Descriptor>,
) -> Completion<bool> {
    let desc = own_desc.unwrap_or_else(|| Descriptor::data(Value::Undefined, true, true, true));

    if desc.is_data_descriptor() {
        if !desc.writable() {
            return Ok(false);
        }
        let Value::Object(receiver_ref) = receiver else {
            return Ok(false);
        };
        if let Some(existing) = receiver_ref.get_own_property(cx, &key)? {
            if existing.is_accessor_descriptor() || !existing.writable() {
                return Ok(false);
            }
            let mut value_desc = Descriptor::empty();
            value_desc.value = Some(value);
            return receiver_ref.define_own_property(cx, key, value_desc);
        }
        return receiver_ref.define_own_property(
            cx,
            key,
            Descriptor::data(value, true, true, true),
        );
    }

    let setter = desc.set.unwrap_or(Value::Undefined);
    if matches!(setter, Value::Undefined) {
        return Ok(false);
    }
    cx.call_mut(setter, receiver, &[value])?;
    Ok(true)
}

fn would_create_prototype_cycle(
    cx: &mut Context,
    object: ObjectRef,
    mut proto: Option<ObjectRef>,
) -> Completion<bool> {
    while let Some(current) = proto {
        if current == object {
            return Ok(true);
        }
        if is_proxy_object(cx, current)? {
            return Ok(false);
        }
        proto = current.get_prototype_of(cx)?;
    }
    Ok(false)
}

fn is_proxy_object(cx: &Context, object: ObjectRef) -> Completion<bool> {
    Ok(cx
        .heap()
        .get(object)?
        .internal_slots
        .iter()
        .any(|slot| matches!(slot, InternalSlot::ProxyData { .. })))
}

fn array_index_key(value: &str) -> Option<u32> {
    if value.is_empty() {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    let index = value.parse::<u32>().ok()?;
    if index == u32::MAX || index.to_string() != value {
        return None;
    }
    Some(index)
}

fn update_array_length_after_index(object: &mut JsObject, index: u32) {
    let key = PropertyKey::from("length");
    let current = object
        .properties
        .get(&key)
        .and_then(|desc| desc.value.as_ref())
        .and_then(|value| match value {
            Value::Number(value) if *value >= 0.0 => Some(*value as u32),
            _ => None,
        })
        .unwrap_or(0);
    if index >= current {
        if let Some(desc) = object.properties.get_mut(&key) {
            desc.value = Some(Value::Number((index + 1) as f64));
        } else {
            object.properties.insert(
                key.clone(),
                Descriptor::data(Value::Number((index + 1) as f64), true, false, false),
            );
        }
        if !object.property_order.contains(&key) {
            object.property_order.push(key);
        }
    }
}

fn is_array_length_key(cx: &Context, object: ObjectRef, key: &PropertyKey) -> Completion<bool> {
    Ok(matches!(cx.heap().get(object)?.kind, ObjectKind::Array)
        && matches!(key, PropertyKey::String(value) if value == "length"))
}

fn is_array_index_key(cx: &Context, object: ObjectRef, key: &PropertyKey) -> Completion<bool> {
    Ok(matches!(cx.heap().get(object)?.kind, ObjectKind::Array)
        && matches!(key, PropertyKey::String(value) if array_index_key(value).is_some()))
}

fn array_can_define_index(
    cx: &Context,
    object: ObjectRef,
    key: &PropertyKey,
    current: &Option<Descriptor>,
) -> Completion<bool> {
    if current.is_some() {
        return Ok(true);
    }
    let PropertyKey::String(value) = key else {
        return Ok(true);
    };
    let Some(index) = array_index_key(value) else {
        return Ok(true);
    };
    let length_desc = cx
        .heap()
        .get(object)?
        .properties
        .get(&PropertyKey::from("length"))
        .cloned();
    let Some(length_desc) = length_desc else {
        return Ok(true);
    };
    let length = length_desc
        .value
        .as_ref()
        .and_then(|value| match value {
            Value::Number(value) if *value >= 0.0 => Some(*value as u32),
            _ => None,
        })
        .unwrap_or(0);
    Ok(index < length || length_desc.writable())
}

fn to_array_length_for_set(cx: &mut Context, value: &Value) -> Completion<u32> {
    let uint32_number = array_length_number(cx, value)?;
    let number_len = array_length_number(cx, value)?;
    if !number_len.is_finite()
        || number_len < 0.0
        || number_len.fract() != 0.0
        || number_len > u32::MAX as f64
        || uint32_number != number_len
    {
        return Err(JsError::range_error("invalid array length"));
    }
    Ok(number_len as u32)
}

fn array_length_number(cx: &mut Context, value: &Value) -> Completion<f64> {
    let primitive = ArgView::new(value.clone()).to_primitive(cx)?;
    match primitive {
        Value::Number(value) => Ok(value),
        Value::String(ref value) if value.is_empty() => Ok(0.0),
        Value::String(ref value) => Ok(value.parse::<f64>().unwrap_or(f64::NAN)),
        Value::Boolean(true) => Ok(1.0),
        Value::Boolean(false) | Value::Null => Ok(0.0),
        Value::Undefined => Ok(f64::NAN),
        _ => Err(JsError::range_error("invalid array length")),
    }
}
