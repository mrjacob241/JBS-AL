use crate::runtime::abstract_ops::CreateArrayFromList;

use super::{
    Completion, FunctionData, Heap, InternalMethods, InternalSlot, JsError, ObjectKind, ObjectRef,
    PropertyKey, Realm, RealmId, Runtime, Value,
};

pub struct Context<'rt> {
    runtime: &'rt mut Runtime,
    realm: RealmId,
}

impl<'rt> Context<'rt> {
    pub fn new(runtime: &'rt mut Runtime, realm: RealmId) -> Self {
        Self { runtime, realm }
    }

    pub fn realm_id(&self) -> RealmId {
        self.realm
    }

    pub fn heap(&self) -> &Heap {
        &self.runtime.heap
    }

    pub fn heap_mut(&mut self) -> &mut Heap {
        &mut self.runtime.heap
    }

    pub fn realm(&self) -> Completion<&Realm> {
        self.runtime
            .realms
            .get(self.realm.0)
            .ok_or_else(|| JsError::internal(format!("invalid realm {}", self.realm.0)))
    }

    pub fn call(&self, callee: Value, _this_value: Value, _args: &[Value]) -> Completion<Value> {
        let Value::Object(object) = callee else {
            return Err(JsError::type_error("value is not callable"));
        };
        let object_data = self.heap().get(object)?;
        let ObjectKind::Function(FunctionData {
            callable: true,
            builtin: Some(_),
            ..
        }) = &object_data.kind
        else {
            return Err(JsError::type_error("object is not callable"));
        };

        Err(JsError::internal(
            "calling built-ins that mutate context requires Context::call_mut",
        ))
    }

    pub fn call_mut(
        &mut self,
        callee: Value,
        this_value: Value,
        args: &[Value],
    ) -> Completion<Value> {
        let Value::Object(object) = callee else {
            return Err(JsError::type_error("value is not callable"));
        };
        if let Some((target, handler)) = self.proxy_parts(object)? {
            if !self.is_callable(&Value::Object(target))? {
                return Err(JsError::type_error("proxy target is not callable"));
            }
            if let Some(trap) = self.proxy_trap(handler, "apply")? {
                let args_array = CreateArrayFromList(self, args.to_vec())?;
                return self.call_mut(
                    trap,
                    Value::Object(handler),
                    &[Value::Object(target), this_value, Value::Object(args_array)],
                );
            }
            return self.call_mut(Value::Object(target), this_value, args);
        }
        let data = match &self.heap().get(object)?.kind {
            ObjectKind::Function(data) if data.callable => data.clone(),
            _ => return Err(JsError::type_error("object is not callable")),
        };
        if let Some(bound) = data.bound {
            let mut bound_args = bound.args;
            bound_args.extend_from_slice(args);
            return self.call_mut(bound.target, bound.this_value, &bound_args);
        }
        if let Some(function) = data.builtin {
            return function(self, this_value, args);
        }
        if data.script.is_some() {
            return crate::syntax::call_script_function(self, data, this_value, args);
        }
        Err(JsError::type_error("function has no executable body"))
    }

    pub fn construct_mut(&mut self, callee: Value, args: &[Value]) -> Completion<Value> {
        self.construct_mut_with_new_target(callee.clone(), args, callee)
    }

    pub fn construct_mut_with_new_target(
        &mut self,
        callee: Value,
        args: &[Value],
        new_target: Value,
    ) -> Completion<Value> {
        let Value::Object(object) = callee.clone() else {
            return Err(JsError::type_error("constructor target is not callable"));
        };
        if let Some((target, handler)) = self.proxy_parts(object)? {
            if !self.is_constructor(&Value::Object(target))? {
                return Err(JsError::type_error("proxy target is not a constructor"));
            }
            if let Some(trap) = self.proxy_trap(handler, "construct")? {
                let args_array = CreateArrayFromList(self, args.to_vec())?;
                let result = self.call_mut(
                    trap,
                    Value::Object(handler),
                    &[
                        Value::Object(target),
                        Value::Object(args_array),
                        new_target.clone(),
                    ],
                )?;
                if !matches!(result, Value::Object(_)) {
                    return Err(JsError::type_error(
                        "proxy construct trap must return an object",
                    ));
                }
                return Ok(result);
            }
            return self.construct_mut_with_new_target(Value::Object(target), args, new_target);
        }
        let data = match &self.heap().get(object)?.kind {
            ObjectKind::Function(data) => data.clone(),
            _ => return Err(JsError::type_error("constructor target is not callable")),
        };
        if let Some(bound) = data.bound {
            let mut bound_args = bound.args;
            bound_args.extend_from_slice(args);
            let bound_new_target = if matches!(new_target, Value::Object(target) if target == object)
            {
                bound.target.clone()
            } else {
                new_target
            };
            return self.construct_mut_with_new_target(bound.target, &bound_args, bound_new_target);
        }
        if !data.constructible {
            return Err(JsError::type_error("function is not a constructor"));
        }
        if data.script.is_some() {
            let Value::Object(new_target_object) = new_target else {
                return Err(JsError::type_error("newTarget is not an object"));
            };
            return crate::syntax::construct_script_function(
                self,
                Value::Object(new_target_object),
                new_target_object,
                data,
                args,
            );
        }
        if data.builtin.is_some() {
            let Value::Object(new_target_object) = new_target else {
                return Err(JsError::type_error("newTarget is not an object"));
            };
            return self.call_mut(
                callee.clone(),
                Value::InternalConstructWithNewTarget(new_target_object),
                args,
            );
        }
        Err(JsError::type_error(
            "constructor execution is not implemented for this target in JBS-5",
        ))
    }

    pub fn eval_script(&mut self, source: &str) -> Completion<Value> {
        crate::syntax::eval_script(self.runtime, source)
    }

    pub fn is_callable(&self, value: &Value) -> Completion<bool> {
        let Value::Object(object) = value else {
            return Ok(false);
        };
        if let Some(callable) = self.proxy_callable(*object)? {
            return Ok(callable);
        }
        Ok(matches!(
            self.heap().get(*object)?.kind,
            ObjectKind::Function(FunctionData { callable: true, .. })
        ))
    }

    pub fn is_constructor(&self, value: &Value) -> Completion<bool> {
        let Value::Object(object) = value else {
            return Ok(false);
        };
        if let Some(constructible) = self.proxy_constructible(*object)? {
            return Ok(constructible);
        }
        if let ObjectKind::Function(FunctionData {
            bound: Some(bound), ..
        }) = &self.heap().get(*object)?.kind
        {
            return self.is_constructor(&bound.target);
        }
        Ok(matches!(
            self.heap().get(*object)?.kind,
            ObjectKind::Function(FunctionData {
                constructible: true,
                ..
            })
        ))
    }

    pub fn fresh_symbol(&mut self) -> u64 {
        self.fresh_symbol_with_description(None)
    }

    pub fn fresh_symbol_with_description(&mut self, description: Option<String>) -> u64 {
        let id = self.runtime.next_symbol_id;
        self.runtime.next_symbol_id += 1;
        self.runtime.symbol_descriptions.insert(id, description);
        id
    }

    pub fn symbol_description(&self, symbol: u64) -> Option<&String> {
        self.runtime
            .symbol_descriptions
            .get(&symbol)
            .and_then(|description| description.as_ref())
    }

    fn proxy_parts(&self, object: ObjectRef) -> Completion<Option<(ObjectRef, ObjectRef)>> {
        let object_data = self.heap().get(object)?;
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

    fn proxy_callable(&self, object: ObjectRef) -> Completion<Option<bool>> {
        let object_data = self.heap().get(object)?;
        for slot in &object_data.internal_slots {
            if let InternalSlot::ProxyData { callable, .. } = slot {
                return Ok(Some(*callable));
            }
        }
        Ok(None)
    }

    fn proxy_constructible(&self, object: ObjectRef) -> Completion<Option<bool>> {
        let object_data = self.heap().get(object)?;
        for slot in &object_data.internal_slots {
            if let InternalSlot::ProxyData { constructible, .. } = slot {
                return Ok(Some(*constructible));
            }
        }
        Ok(None)
    }

    fn proxy_trap(&mut self, handler: ObjectRef, name: &str) -> Completion<Option<Value>> {
        let trap = handler.get(self, &PropertyKey::from(name), Value::Object(handler))?;
        if matches!(trap, Value::Undefined | Value::Null) {
            return Ok(None);
        }
        if !self.is_callable(&trap)? {
            return Err(JsError::type_error("proxy trap is not callable"));
        }
        Ok(Some(trap))
    }
}
