use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::runtime::{
    get_iterator, iterator_close_error, iterator_step_value, ArgView, BindingCell, Completion,
    Context, Descriptor, FunctionData, FunctionEnvironment, InternalMethods, JsError, JsObject,
    ObjectKind, ObjectRef, PropertyKey, RegExpCreate, Runtime, Value,
};

use super::parser::{
    parse_script, AssignOp, BinaryOp, BindingPattern, Expr, ForInit, LogicalOp, ObjectPropertyKind,
    Stmt, SwitchCase,
};

pub fn eval_script(runtime: &mut Runtime, source: &str) -> Completion<Value> {
    let realm = runtime.default_realm();
    let mut cx = Context::new(runtime, realm);
    let global = cx.realm()?.global_object;
    let mut env = Env::global(global);
    env.insert_local("this".to_owned(), Value::Object(global));
    let statements = parse_script(source)?;
    if has_use_strict_directive(&statements) {
        env.strict = true;
    }
    instantiate_function_declarations(&mut cx, &mut env, &statements)?;
    let mut last = Value::Undefined;
    for stmt in statements {
        match eval_stmt(&mut cx, &mut env, stmt)? {
            Flow::Normal(value) => last = value,
            Flow::Break | Flow::Continue => {
                return Err(JsError::syntax("break/continue outside loop"));
            }
            Flow::Return(value) => return Ok(value),
            Flow::Throw(value) => {
                return Err(JsError::throw_value(value));
            }
        }
    }
    Ok(last)
}

#[derive(Clone)]
struct Env {
    bindings: Rc<RefCell<HashMap<String, BindingCell>>>,
    global: Option<ObjectRef>,
    global_names: Rc<RefCell<HashSet<String>>>,
    local_names: Rc<RefCell<HashSet<String>>>,
    shadow_names: Rc<RefCell<HashSet<String>>>,
    with_objects: Rc<RefCell<Vec<ObjectRef>>>,
    new_target: Value,
    is_global_frame: bool,
    strict: bool,
}

impl Env {
    fn global(global: ObjectRef) -> Self {
        Self {
            bindings: Rc::new(RefCell::new(HashMap::new())),
            global: Some(global),
            global_names: Rc::new(RefCell::new(HashSet::new())),
            local_names: Rc::new(RefCell::new(HashSet::new())),
            shadow_names: Rc::new(RefCell::new(HashSet::new())),
            with_objects: Rc::new(RefCell::new(Vec::new())),
            new_target: Value::Undefined,
            is_global_frame: true,
            strict: false,
        }
    }

    fn function_frame(captured: FunctionEnvironment) -> Self {
        Self {
            bindings: Rc::new(RefCell::new(captured.bindings.borrow().clone())),
            global: captured.global,
            global_names: captured.global_names,
            local_names: Rc::new(RefCell::new(HashSet::new())),
            shadow_names: Rc::new(RefCell::new(HashSet::new())),
            with_objects: Rc::new(RefCell::new(Vec::new())),
            new_target: Value::Undefined,
            is_global_frame: false,
            strict: captured.strict,
        }
    }

    fn capture(&self) -> FunctionEnvironment {
        FunctionEnvironment {
            bindings: self.bindings.clone(),
            global: self.global,
            global_names: self.global_names.clone(),
            strict: self.strict,
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        self.bindings
            .borrow()
            .get(name)
            .map(|cell| cell.borrow().clone())
    }

    fn insert_local(&self, name: String, value: Value) {
        self.local_names.borrow_mut().insert(name.clone());
        let existing = self.bindings.borrow().get(&name).cloned();
        if let Some(cell) = existing {
            *cell.borrow_mut() = value;
        } else {
            self.bindings
                .borrow_mut()
                .insert(name, Rc::new(RefCell::new(value)));
        }
    }

    fn declare_local(&self, name: String, value: Value) {
        if self.local_names.borrow().contains(&name) {
            self.insert_local(name, value);
        } else {
            self.local_names.borrow_mut().insert(name.clone());
            self.bindings
                .borrow_mut()
                .insert(name, Rc::new(RefCell::new(value)));
        }
    }

    fn declare_var(&self, cx: &mut Context, name: String, value: Value) -> Completion<()> {
        if self.is_global_frame {
            self.insert_local(name.clone(), value.clone());
            self.global_names.borrow_mut().insert(name.clone());
            self.define_or_update_global(cx, &name, value)?;
        } else if self.local_names.borrow().contains(&name) {
            self.insert_local(name, value);
        } else {
            self.local_names.borrow_mut().insert(name.clone());
            self.bindings
                .borrow_mut()
                .insert(name, Rc::new(RefCell::new(value)));
        }
        Ok(())
    }

    fn set_identifier(&self, cx: &mut Context, name: String, value: Value) -> Completion<()> {
        if !self.is_shadow_name(&name) {
            if let Some(object) = self.with_binding_object(cx, &name)? {
                object
                    .set(
                        cx,
                        PropertyKey::from(name.as_str()),
                        value,
                        Value::Object(object),
                    )
                    .and_then(|ok| self.require_set_success(ok))?;
                return Ok(());
            }
        }
        self.set_identifier_binding(cx, name, value)
    }

    fn set_identifier_binding(
        &self,
        cx: &mut Context,
        name: String,
        value: Value,
    ) -> Completion<()> {
        let existing = self.bindings.borrow().get(&name).cloned();
        if let Some(cell) = existing {
            *cell.borrow_mut() = value.clone();
        } else {
            self.insert_local(name.clone(), value.clone());
        }
        let should_mirror_global = self.global_names.borrow().contains(&name)
            && !self.is_shadow_name(&name)
            && (self.is_global_frame || !self.is_local_name(&name));
        if should_mirror_global {
            self.define_or_update_global(cx, &name, value)?;
        }
        Ok(())
    }

    fn remove_local(&self, name: &str) {
        self.bindings.borrow_mut().remove(name);
        self.local_names.borrow_mut().remove(name);
    }

    fn delete_identifier(&self, cx: &mut Context, name: &str) -> Completion<bool> {
        if !self.is_shadow_name(name) {
            if let Some(object) = self.with_binding_object(cx, name)? {
                let deleted = object.delete(cx, &PropertyKey::from(name))?;
                return self.require_delete_success(deleted);
            }
        }
        self.remove_local(name);
        Ok(true)
    }

    fn shadow_local(&self, name: String, value: Value) -> (Option<BindingCell>, bool) {
        let had_local_name = self.local_names.borrow().contains(&name);
        let previous = self
            .bindings
            .borrow_mut()
            .insert(name.clone(), Rc::new(RefCell::new(value)));
        self.local_names.borrow_mut().insert(name.clone());
        self.shadow_names.borrow_mut().insert(name.clone());
        (previous, had_local_name)
    }

    fn restore_shadow(&self, name: String, previous: Option<BindingCell>, had_local_name: bool) {
        if let Some(previous) = previous {
            self.bindings.borrow_mut().insert(name.clone(), previous);
        } else {
            self.bindings.borrow_mut().remove(&name);
        }
        if !had_local_name {
            self.local_names.borrow_mut().remove(&name);
        }
        self.shadow_names.borrow_mut().remove(&name);
    }

    fn define_or_update_global(
        &self,
        cx: &mut Context,
        name: &str,
        value: Value,
    ) -> Completion<()> {
        let Some(global) = self.global else {
            return Ok(());
        };
        let key = PropertyKey::from(name);
        if global.has_property(cx, &key)? {
            if global.set(cx, key.clone(), value.clone(), Value::Object(global))? {
                return Ok(());
            }
        }
        global.define_own_property_or_throw(cx, key, Descriptor::data(value, true, true, false))
    }

    fn is_global_name(&self, name: &str) -> bool {
        self.global_names.borrow().contains(name)
    }

    fn is_local_name(&self, name: &str) -> bool {
        self.local_names.borrow().contains(name)
    }

    fn is_shadow_name(&self, name: &str) -> bool {
        self.shadow_names.borrow().contains(name)
    }

    fn global_object(&self) -> Option<ObjectRef> {
        self.global
    }

    fn push_with_object(&self, object: ObjectRef) {
        self.with_objects.borrow_mut().push(object);
    }

    fn pop_with_object(&self) {
        self.with_objects.borrow_mut().pop();
    }

    fn has_with_object(&self) -> bool {
        !self.with_objects.borrow().is_empty()
    }

    fn require_set_success(&self, ok: bool) -> Completion<bool> {
        if !ok && self.strict {
            return Err(JsError::type_error("strict assignment failed"));
        }
        Ok(ok)
    }

    fn require_delete_success(&self, ok: bool) -> Completion<bool> {
        if !ok && self.strict {
            return Err(JsError::type_error("strict delete failed"));
        }
        Ok(ok)
    }

    fn with_binding_object(&self, cx: &mut Context, name: &str) -> Completion<Option<ObjectRef>> {
        if name == "this" {
            return Ok(None);
        }
        let key = PropertyKey::from(name);
        let objects = self.with_objects.borrow().clone();
        for object in objects.into_iter().rev() {
            if object.has_property(cx, &key)? {
                return Ok(Some(object));
            }
        }
        Ok(None)
    }
}

enum Flow {
    Normal(Value),
    Break,
    Continue,
    Return(Value),
    Throw(Value),
}

enum AssignmentTarget {
    Identifier(String, Option<ObjectRef>),
    Member(ObjectRef, PropertyKey),
    Ignored,
}

fn eval_stmt(cx: &mut Context, env: &mut Env, stmt: Stmt) -> Completion<Flow> {
    match stmt {
        Stmt::Var(binding, expr) => {
            let value = match expr {
                Expr::Function(None, params, body) if binding.identifier_name().is_some() => {
                    let name = binding_identifier(binding.clone())?;
                    create_script_function(cx, env, name.clone(), params, body)?
                }
                other => eval_expr(cx, env, other)?,
            };
            declare_binding(cx, env, binding, value.clone())?;
            Ok(Flow::Normal(value))
        }
        Stmt::Function(_, _, _) => Ok(Flow::Normal(Value::Undefined)),
        Stmt::Block(statements) => eval_block(cx, env, statements),
        Stmt::If(test, then_branch, else_branch) => {
            if to_boolean(eval_expr(cx, env, test)?) {
                eval_stmt(cx, env, *then_branch)
            } else if let Some(else_branch) = else_branch {
                eval_stmt(cx, env, *else_branch)
            } else {
                Ok(Flow::Normal(Value::Undefined))
            }
        }
        Stmt::While(test, body) => {
            let mut last = Value::Undefined;
            while to_boolean(eval_expr(cx, env, test.clone())?) {
                match eval_stmt(cx, env, (*body).clone())? {
                    Flow::Normal(value) => last = value,
                    Flow::Continue => continue,
                    Flow::Break => break,
                    Flow::Return(value) => return Ok(Flow::Return(value)),
                    Flow::Throw(value) => return Ok(Flow::Throw(value)),
                }
            }
            Ok(Flow::Normal(last))
        }
        Stmt::Switch(discriminant, cases) => eval_switch(cx, env, discriminant, cases),
        Stmt::With(object_expr, body) => {
            let object = ArgView::new(eval_expr(cx, env, object_expr)?).to_object(cx)?;
            env.push_with_object(object);
            let result = eval_stmt(cx, env, *body);
            env.pop_with_object();
            result
        }
        Stmt::For(init, test, update, body) => {
            if let Some(init) = init {
                match init {
                    ForInit::Var(binding, expr) => {
                        let value = eval_expr(cx, env, expr)?;
                        declare_binding(cx, env, binding, value)?;
                    }
                    ForInit::Expr(expr) => {
                        eval_expr(cx, env, expr)?;
                    }
                }
            }
            let mut last = Value::Undefined;
            loop {
                if let Some(test) = test.clone() {
                    if !to_boolean(eval_expr(cx, env, test)?) {
                        break;
                    }
                }
                match eval_stmt(cx, env, (*body).clone())? {
                    Flow::Normal(value) => last = value,
                    Flow::Continue => {}
                    Flow::Break => break,
                    Flow::Return(value) => return Ok(Flow::Return(value)),
                    Flow::Throw(value) => return Ok(Flow::Throw(value)),
                }
                if let Some(update) = update.clone() {
                    eval_expr(cx, env, update)?;
                }
            }
            Ok(Flow::Normal(last))
        }
        Stmt::ForIn(name, object_expr, body) => {
            let value = eval_expr(cx, env, object_expr)?;
            if matches!(value, Value::Undefined | Value::Null) {
                return Ok(Flow::Normal(Value::Undefined));
            }
            let object = ArgView::new(value).to_object(cx)?;
            let keys = enumerable_property_names(cx, object)?;
            let mut last = Value::Undefined;
            for key in keys {
                env.declare_var(cx, name.clone(), Value::String(key))?;
                match eval_stmt(cx, env, (*body).clone())? {
                    Flow::Normal(value) => last = value,
                    Flow::Continue => continue,
                    Flow::Break => break,
                    Flow::Return(value) => return Ok(Flow::Return(value)),
                    Flow::Throw(value) => return Ok(Flow::Throw(value)),
                }
            }
            Ok(Flow::Normal(last))
        }
        Stmt::ForOf(name, iterable_expr, body) => {
            let iterable = eval_expr(cx, env, iterable_expr)?;
            let iterator = get_iterator(cx, iterable)?;
            let mut last = Value::Undefined;
            loop {
                let value = match iterator_step_value(cx, &iterator) {
                    Ok(Some(value)) => value,
                    Ok(None) => break,
                    Err(error) => return Err(iterator_close_error(cx, &iterator, error)),
                };
                env.declare_var(cx, name.clone(), value)?;
                match eval_stmt(cx, env, (*body).clone())? {
                    Flow::Normal(value) => last = value,
                    Flow::Continue => continue,
                    Flow::Break => {
                        return match crate::runtime::iterator_close(cx, &iterator) {
                            Ok(()) => Ok(Flow::Normal(last)),
                            Err(error) => Err(error),
                        };
                    }
                    Flow::Return(value) => {
                        return match crate::runtime::iterator_close(cx, &iterator) {
                            Ok(()) => Ok(Flow::Return(value)),
                            Err(error) => Err(error),
                        };
                    }
                    Flow::Throw(value) => {
                        let error = JsError::throw_value(value);
                        return Err(iterator_close_error(cx, &iterator, error));
                    }
                }
            }
            Ok(Flow::Normal(last))
        }
        Stmt::Try(try_body, catch, finally_body) => {
            let outcome = eval_block(cx, env, try_body);
            let mut flow = match outcome {
                Err(error) => match catch {
                    Some((binding, catch_body)) => {
                        let value = js_error_to_value(cx, error)?;
                        run_catch(cx, env, binding, catch_body, value)
                    }
                    None => Err(error),
                },
                Ok(Flow::Throw(value)) => match catch {
                    Some((binding, catch_body)) => run_catch(cx, env, binding, catch_body, value),
                    None => Ok(Flow::Throw(value)),
                },
                Ok(flow) => Ok(flow),
            };
            if let Some(finally_body) = finally_body {
                match eval_block(cx, env, finally_body)? {
                    Flow::Normal(_) => {}
                    finally_flow => flow = Ok(finally_flow),
                }
            }
            flow
        }
        Stmt::Break => Ok(Flow::Break),
        Stmt::Continue => Ok(Flow::Continue),
        Stmt::Return(expr) => {
            let value = if let Some(expr) = expr {
                eval_expr(cx, env, expr)?
            } else {
                Value::Undefined
            };
            Ok(Flow::Return(value))
        }
        Stmt::Throw(expr) => {
            let value = eval_expr(cx, env, expr)?;
            Ok(Flow::Throw(value))
        }
        Stmt::Expr(expr) => eval_expr(cx, env, expr).map(Flow::Normal),
    }
}

fn eval_switch(
    cx: &mut Context,
    env: &mut Env,
    discriminant: Expr,
    cases: Vec<SwitchCase>,
) -> Completion<Flow> {
    let discriminant = eval_expr(cx, env, discriminant)?;
    let mut default_index = None;
    let mut start_index = None;

    for (index, case) in cases.iter().enumerate() {
        let Some(test) = case.test.clone() else {
            if default_index.is_none() {
                default_index = Some(index);
            }
            continue;
        };
        let value = eval_expr(cx, env, test)?;
        if strict_equal(&discriminant, &value) {
            start_index = Some(index);
            break;
        }
    }

    let Some(start_index) = start_index.or(default_index) else {
        return Ok(Flow::Normal(Value::Undefined));
    };

    let mut last = Value::Undefined;
    for case in cases.into_iter().skip(start_index) {
        for stmt in case.consequent {
            match eval_stmt(cx, env, stmt)? {
                Flow::Normal(value) => last = value,
                Flow::Break => return Ok(Flow::Normal(last)),
                Flow::Continue => return Ok(Flow::Continue),
                Flow::Return(value) => return Ok(Flow::Return(value)),
                Flow::Throw(value) => return Ok(Flow::Throw(value)),
            }
        }
    }
    Ok(Flow::Normal(last))
}

fn binding_identifier(binding: BindingPattern) -> Completion<String> {
    binding
        .into_identifier()
        .ok_or_else(|| JsError::syntax("binding pattern must be an identifier"))
}

fn declare_binding(
    cx: &mut Context,
    env: &mut Env,
    binding: BindingPattern,
    value: Value,
) -> Completion<()> {
    match binding {
        BindingPattern::Identifier(name) => env.declare_var(cx, name, value),
        BindingPattern::Array(names) => {
            let object = ArgView::new(value).to_object(cx)?;
            for (index, name) in names.into_iter().enumerate() {
                let Some(name) = name else {
                    continue;
                };
                let value = object.get(
                    cx,
                    &PropertyKey::array_index(index as u64),
                    Value::Object(object),
                )?;
                env.declare_var(cx, name, value)?;
            }
            Ok(())
        }
    }
}

fn eval_block(cx: &mut Context, env: &mut Env, statements: Vec<Stmt>) -> Completion<Flow> {
    instantiate_function_declarations(cx, env, &statements)?;
    let previous_strict = env.strict;
    if has_use_strict_directive(&statements) {
        env.strict = true;
    }
    let mut last = Value::Undefined;
    let result = (|| {
        for stmt in statements {
            match eval_stmt(cx, env, stmt)? {
                Flow::Normal(value) => last = value,
                flow => return Ok(flow),
            }
        }
        Ok(Flow::Normal(last))
    })();
    env.strict = previous_strict;
    result
}

fn has_use_strict_directive(statements: &[Stmt]) -> bool {
    for stmt in statements {
        match stmt {
            Stmt::Expr(Expr::String(value)) if value == "use strict" => return true,
            Stmt::Expr(Expr::String(_)) => continue,
            _ => return false,
        }
    }
    false
}

fn instantiate_function_declarations(
    cx: &mut Context,
    env: &mut Env,
    statements: &[Stmt],
) -> Completion<()> {
    for stmt in statements {
        if let Stmt::Function(name, params, body) = stmt {
            let value =
                create_script_function(cx, env, name.clone(), params.clone(), body.clone())?;
            env.declare_var(cx, name.clone(), value)?;
        }
    }
    Ok(())
}

fn run_catch(
    cx: &mut Context,
    env: &mut Env,
    binding: Option<String>,
    catch_body: Vec<Stmt>,
    value: Value,
) -> Completion<Flow> {
    let shadow = binding
        .as_ref()
        .map(|name| env.shadow_local(name.clone(), value));
    let result = eval_block(cx, env, catch_body);
    if let Some(name) = binding {
        let (previous, had_local_name) = shadow.expect("catch binding shadow should exist");
        env.restore_shadow(name, previous, had_local_name);
    }
    result
}

fn eval_expr(cx: &mut Context, env: &mut Env, expr: Expr) -> Completion<Value> {
    match expr {
        Expr::Identifier(name) => resolve_identifier(cx, env, &name),
        Expr::Number(value) => Ok(Value::Number(value)),
        Expr::BigInt(value) => Ok(Value::BigInt(value)),
        Expr::String(value) => Ok(Value::String(value)),
        Expr::RegExp(pattern, flags) => RegExpCreate(cx, pattern, flags),
        Expr::Boolean(value) => Ok(Value::Boolean(value)),
        Expr::Null => Ok(Value::Null),
        Expr::Undefined => Ok(Value::Undefined),
        Expr::ArrayHole => Ok(Value::Undefined),
        Expr::Object(properties) => {
            let proto = cx
                .realm()?
                .intrinsics
                .get(crate::runtime::IntrinsicId::ObjectPrototype)
                .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
            let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
            for property in properties {
                if let ObjectPropertyKind::Spread(expr) = property.kind {
                    let source = eval_expr(cx, env, expr)?;
                    if matches!(source, Value::Undefined | Value::Null) {
                        continue;
                    }
                    let source = ArgView::new(source).to_object(cx)?;
                    for key in source.own_property_keys(cx)? {
                        let Some(source_desc) = source.get_own_property(cx, &key)? else {
                            continue;
                        };
                        if source_desc.enumerable() {
                            let value = source.get(cx, &key, Value::Object(source))?;
                            object.define_own_property_or_throw(
                                cx,
                                key,
                                Descriptor::data(value, true, true, true),
                            )?;
                        }
                    }
                    continue;
                }
                let key = ArgView::new(eval_expr(cx, env, property.key)?).to_property_key(cx)?;
                let desc = match property.kind {
                    ObjectPropertyKind::Data(expr) => {
                        Descriptor::data(eval_expr(cx, env, expr)?, true, true, true)
                    }
                    ObjectPropertyKind::Get(expr) => {
                        Descriptor::accessor(Some(eval_expr(cx, env, expr)?), None, true, true)
                    }
                    ObjectPropertyKind::Set(expr) => {
                        Descriptor::accessor(None, Some(eval_expr(cx, env, expr)?), true, true)
                    }
                    ObjectPropertyKind::Spread(_) => unreachable!("spread handled before key eval"),
                };
                object.define_own_property_or_throw(cx, key, desc)?;
            }
            Ok(Value::Object(object))
        }
        Expr::Function(name, params, body) => {
            create_script_function(cx, env, name.unwrap_or_default(), params, body)
        }
        Expr::NewTarget => Ok(env.new_target.clone()),
        Expr::Array(elements) => {
            let proto = cx
                .realm()?
                .intrinsics
                .get(crate::runtime::IntrinsicId::ArrayPrototype)
                .ok_or_else(|| JsError::internal("missing Array.prototype intrinsic"))?;
            let object = cx.heap_mut().allocate(JsObject::array(Some(proto)));
            let len = elements.len();
            for (index, expr) in elements.into_iter().enumerate() {
                if matches!(expr, Expr::ArrayHole) {
                    continue;
                }
                let value = eval_expr(cx, env, expr)?;
                object.define_own_property_or_throw(
                    cx,
                    PropertyKey::array_index(index as u64),
                    Descriptor::data(value, true, true, true),
                )?;
            }
            object.define_own_property_or_throw(
                cx,
                PropertyKey::from("length"),
                Descriptor::data(Value::Number(len as f64), true, false, false),
            )?;
            Ok(Value::Object(object))
        }
        Expr::Member(base, key) => {
            let base = eval_expr(cx, env, *base)?;
            let object = ArgView::new(base.clone()).to_object(cx)?;
            let key = ArgView::new(eval_expr(cx, env, *key)?).to_property_key(cx)?;
            object.get(cx, &key, base)
        }
        Expr::Call(callee, args) => {
            let (callee_value, this_value) = eval_callee(cx, env, *callee)?;
            let mut values = Vec::new();
            for arg in args {
                values.push(eval_expr(cx, env, arg)?);
            }
            call_value(cx, env, callee_value, this_value, &values)
        }
        Expr::New(callee, args) => {
            let callee = eval_expr(cx, env, *callee)?;
            let mut values = Vec::new();
            for arg in args {
                values.push(eval_expr(cx, env, arg)?);
            }
            construct_value(cx, env, callee, &values)
        }
        Expr::Void(expr) => {
            eval_expr(cx, env, *expr)?;
            Ok(Value::Undefined)
        }
        Expr::TypeOf(expr) => eval_typeof(cx, env, *expr),
        Expr::Delete(expr) => eval_delete(cx, env, *expr),
        Expr::Not(expr) => {
            let value = eval_expr(cx, env, *expr)?;
            Ok(Value::Boolean(!to_boolean(value)))
        }
        Expr::Pos(expr) => {
            let value = eval_expr(cx, env, *expr)?;
            Ok(Value::Number(to_number(cx, value)?))
        }
        Expr::Neg(expr) => {
            let value = eval_expr(cx, env, *expr)?;
            if let Value::BigInt(value) = value {
                return Ok(Value::BigInt(-value));
            }
            Ok(Value::Number(-to_number(cx, value)?))
        }
        Expr::Update(target, increment, prefix) => {
            let old = eval_expr(cx, env, (*target).clone())?;
            let old_number = to_number(cx, old)?;
            let new = Value::Number(old_number + if increment { 1.0 } else { -1.0 });
            assign(cx, env, *target, new.clone())?;
            if prefix {
                Ok(new)
            } else {
                Ok(Value::Number(old_number))
            }
        }
        Expr::Binary(left, op, right) => {
            let left = eval_expr(cx, env, *left)?;
            let right = eval_expr(cx, env, *right)?;
            eval_binary(cx, op, left, right)
        }
        Expr::Logical(left, op, right) => {
            let left = eval_expr(cx, env, *left)?;
            match op {
                LogicalOp::And if !to_boolean(left.clone()) => Ok(left),
                LogicalOp::And => eval_expr(cx, env, *right),
                LogicalOp::Or if to_boolean(left.clone()) => Ok(left),
                LogicalOp::Or => eval_expr(cx, env, *right),
            }
        }
        Expr::Assign(left, op, right) => {
            let value = match op {
                AssignOp::Simple => {
                    let target = eval_assignment_target(cx, env, *left)?;
                    let value = eval_expr(cx, env, *right)?;
                    assign_to_target(cx, env, target, value.clone())?;
                    return Ok(value);
                }
                AssignOp::Binary(op) => {
                    let left_value = eval_expr(cx, env, (*left).clone())?;
                    let right_value = eval_expr(cx, env, *right)?;
                    eval_binary(cx, op, left_value, right_value)?
                }
            };
            assign(cx, env, *left, value.clone())?;
            Ok(value)
        }
        Expr::Equal(left, right, strict, negate) => {
            let left = eval_expr(cx, env, *left)?;
            let right = eval_expr(cx, env, *right)?;
            let equal = if strict {
                strict_equal(&left, &right)
            } else {
                loose_equal(cx, left, right)?
            };
            Ok(Value::Boolean(if negate { !equal } else { equal }))
        }
        Expr::Conditional(test, consequent, alternate) => {
            if to_boolean(eval_expr(cx, env, *test)?) {
                eval_expr(cx, env, *consequent)
            } else {
                eval_expr(cx, env, *alternate)
            }
        }
        Expr::Comma(exprs) => {
            let mut last = Value::Undefined;
            for expr in exprs {
                last = eval_expr(cx, env, expr)?;
            }
            Ok(last)
        }
    }
}

fn strict_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Undefined, Value::Undefined) | (Value::Null, Value::Null) => true,
        (Value::Boolean(left), Value::Boolean(right)) => left == right,
        (Value::String(left), Value::String(right)) => left == right,
        (Value::Symbol(left), Value::Symbol(right)) => left == right,
        (Value::BigInt(left), Value::BigInt(right)) => left == right,
        (Value::Object(left), Value::Object(right)) => left == right,
        (Value::Number(left), Value::Number(right)) => {
            if left.is_nan() || right.is_nan() {
                false
            } else {
                left == right
            }
        }
        _ => false,
    }
}

fn loose_equal(cx: &mut Context, left: Value, right: Value) -> Completion<bool> {
    if strict_equal(&left, &right) {
        return Ok(true);
    }
    if matches!(
        (&left, &right),
        (Value::Null, Value::Undefined) | (Value::Undefined, Value::Null)
    ) {
        return Ok(true);
    }
    match (&left, &right) {
        (Value::Number(_), Value::String(_))
        | (Value::String(_), Value::Number(_))
        | (Value::Boolean(_), _)
        | (_, Value::Boolean(_)) => Ok(to_number(cx, left)? == to_number(cx, right)?),
        _ => Ok(false),
    }
}

fn eval_delete(cx: &mut Context, env: &mut Env, expr: Expr) -> Completion<Value> {
    match expr {
        Expr::Member(base, key) => {
            let base = eval_expr(cx, env, *base)?;
            if matches!(base, Value::Undefined | Value::Null) {
                return Err(JsError::type_error(
                    "cannot delete property of nullish value",
                ));
            }
            let object = ArgView::new(base).to_object(cx)?;
            let key = ArgView::new(eval_expr(cx, env, *key)?).to_property_key(cx)?;
            let deleted = object.delete(cx, &key)?;
            Ok(Value::Boolean(env.require_delete_success(deleted)?))
        }
        Expr::Identifier(name) => Ok(Value::Boolean(env.delete_identifier(cx, &name)?)),
        _ => {
            eval_expr(cx, env, expr)?;
            Ok(Value::Boolean(true))
        }
    }
}

fn eval_instanceof(cx: &mut Context, left: Value, right: Value) -> Completion<Value> {
    let Value::Object(object) = left else {
        return Ok(Value::Boolean(false));
    };
    let Value::Object(constructor) = right else {
        return Err(JsError::type_error(
            "right side of instanceof must be callable",
        ));
    };
    let ObjectKind::Function(data) = cx.heap().get(constructor)?.kind.clone() else {
        return Err(JsError::type_error(
            "right side of instanceof must be callable",
        ));
    };
    if let Some(bound) = data.bound {
        return eval_instanceof(cx, Value::Object(object), bound.target);
    }
    let prototype = constructor.get(
        cx,
        &PropertyKey::from("prototype"),
        Value::Object(constructor),
    )?;
    let Value::Object(target_proto) = prototype else {
        return Err(JsError::type_error("constructor prototype is not object"));
    };
    let mut current = Some(object);
    while let Some(candidate) = current {
        if candidate == target_proto {
            return Ok(Value::Boolean(true));
        }
        current = candidate.get_prototype_of(cx)?;
    }
    Ok(Value::Boolean(false))
}

fn enumerable_property_names(
    cx: &mut Context,
    object: crate::runtime::ObjectRef,
) -> Completion<Vec<String>> {
    let mut names = Vec::new();
    let mut seen = HashSet::new();
    let mut current = Some(object);
    while let Some(candidate) = current {
        for key in candidate.own_property_keys(cx)? {
            let PropertyKey::String(name) = key else {
                continue;
            };
            if !seen.insert(name.clone()) {
                continue;
            }
            if candidate
                .get_own_property(cx, &PropertyKey::from(name.as_str()))?
                .map(|desc| desc.enumerable())
                .unwrap_or(false)
            {
                names.push(name);
            }
        }
        current = candidate.get_prototype_of(cx)?;
    }
    Ok(names)
}

fn eval_binary(cx: &mut Context, op: BinaryOp, left: Value, right: Value) -> Completion<Value> {
    match op {
        BinaryOp::In => {
            let Value::Object(object) = right else {
                return Err(JsError::type_error("right side of in must be object"));
            };
            let key = ArgView::new(left).to_property_key(cx)?;
            return Ok(Value::Boolean(object.has_property(cx, &key)?));
        }
        BinaryOp::InstanceOf => return eval_instanceof(cx, left, right),
        _ => {}
    }

    if matches!(op, BinaryOp::Add) {
        let left = ArgView::new(left).to_primitive(cx)?;
        let right = ArgView::new(right).to_primitive(cx)?;
        if let (Value::BigInt(left), Value::BigInt(right)) = (&left, &right) {
            return Ok(Value::BigInt(left + right));
        }
        if matches!(left, Value::String(_)) || matches!(right, Value::String(_)) {
            return Ok(Value::String(format!(
                "{}{}",
                ArgView::new(left).to_string(cx)?,
                ArgView::new(right).to_string(cx)?
            )));
        }
        return Ok(Value::Number(
            ArgView::new(left).to_number(cx)? + ArgView::new(right).to_number(cx)?,
        ));
    }

    if matches!(
        op,
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
    ) {
        return eval_relational(cx, op, left, right);
    }

    let left = to_number(cx, left)?;
    let right = to_number(cx, right)?;
    Ok(Value::Number(match op {
        BinaryOp::Add => left + right,
        BinaryOp::Sub => left - right,
        BinaryOp::Mul => left * right,
        BinaryOp::Div => left / right,
        BinaryOp::Mod => left % right,
        BinaryOp::Pow => left.powf(right),
        BinaryOp::Shl => ((left as i32).wrapping_shl((right as u32) & 31)) as f64,
        BinaryOp::Shr => ((left as i32).wrapping_shr((right as u32) & 31)) as f64,
        BinaryOp::UShr => ((left as u32).wrapping_shr((right as u32) & 31)) as f64,
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => unreachable!(),
        BinaryOp::In | BinaryOp::InstanceOf => unreachable!(),
    }))
}

fn eval_relational(cx: &mut Context, op: BinaryOp, left: Value, right: Value) -> Completion<Value> {
    let left = ArgView::new(left).to_primitive(cx)?;
    let right = ArgView::new(right).to_primitive(cx)?;
    if let (Value::String(left), Value::String(right)) = (&left, &right) {
        return Ok(Value::Boolean(match op {
            BinaryOp::Lt => left < right,
            BinaryOp::Le => left <= right,
            BinaryOp::Gt => left > right,
            BinaryOp::Ge => left >= right,
            _ => unreachable!(),
        }));
    }
    let left = ArgView::new(left).to_number(cx)?;
    let right = ArgView::new(right).to_number(cx)?;
    if left.is_nan() || right.is_nan() {
        return Ok(Value::Boolean(false));
    }
    Ok(Value::Boolean(match op {
        BinaryOp::Lt => left < right,
        BinaryOp::Le => left <= right,
        BinaryOp::Gt => left > right,
        BinaryOp::Ge => left >= right,
        _ => unreachable!(),
    }))
}

fn eval_typeof(cx: &mut Context, env: &mut Env, expr: Expr) -> Completion<Value> {
    let value = match expr {
        Expr::Identifier(name) => match resolve_identifier(cx, env, &name) {
            Ok(value) => value,
            Err(error) if matches!(error.kind, crate::runtime::ErrorKind::Reference) => {
                Value::Undefined
            }
            Err(error) => return Err(error),
        },
        other => eval_expr(cx, env, other)?,
    };
    let name = match value {
        Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => "undefined",
        Value::Undefined => "undefined",
        Value::Null => "object",
        Value::Boolean(_) => "boolean",
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::BigInt(_) => "bigint",
        Value::Symbol(_) => "symbol",
        Value::Object(_) if cx.is_callable(&value)? => "function",
        Value::Object(_) => "object",
    };
    Ok(Value::String(name.to_owned()))
}

fn call_value(
    cx: &mut Context,
    _env: &mut Env,
    callee: Value,
    this_value: Value,
    args: &[Value],
) -> Completion<Value> {
    cx.call_mut(callee, this_value, args)
}

fn construct_value(
    cx: &mut Context,
    _env: &mut Env,
    callee: Value,
    args: &[Value],
) -> Completion<Value> {
    cx.construct_mut(callee, args)
}

pub(crate) fn call_script_function(
    cx: &mut Context,
    data: FunctionData,
    this_value: Value,
    args: &[Value],
) -> Completion<Value> {
    let Some(script) = data.script else {
        return Err(JsError::type_error("function has no script body"));
    };
    let captured = script
        .environment
        .unwrap_or_else(empty_function_environment);
    let mut fn_env = Env::function_frame(captured);
    fn_env.new_target = Value::Undefined;
    if has_use_strict_directive(&script.body) {
        fn_env.strict = true;
    }
    let this_value = non_strict_this_value(cx, &fn_env, this_value)?;
    fn_env.declare_local("this".to_owned(), this_value);
    fn_env.declare_local("arguments".to_owned(), create_arguments_object(cx, args)?);
    for (index, param) in script.params.iter().enumerate() {
        fn_env.declare_local(
            param.clone(),
            args.get(index).cloned().unwrap_or(Value::Undefined),
        );
    }
    match eval_block(cx, &mut fn_env, script.body)? {
        Flow::Return(value) => Ok(value),
        Flow::Normal(_) => Ok(Value::Undefined),
        Flow::Break | Flow::Continue => Err(JsError::syntax(
            "break/continue cannot escape function body",
        )),
        Flow::Throw(value) => Err(JsError::throw_value(value)),
    }
}

pub(crate) fn construct_script_function(
    cx: &mut Context,
    callee: Value,
    constructor: crate::runtime::ObjectRef,
    data: FunctionData,
    args: &[Value],
) -> Completion<Value> {
    let Some(script) = data.script else {
        return Err(JsError::type_error("function has no script body"));
    };
    let proto_value = constructor.get(cx, &PropertyKey::from("prototype"), callee.clone())?;
    let proto = match proto_value {
        Value::Object(proto) => Some(proto),
        _ => cx
            .realm()?
            .intrinsics
            .get(crate::runtime::IntrinsicId::ObjectPrototype),
    };
    let instance = cx.heap_mut().allocate(JsObject::ordinary(proto));
    let captured = script
        .environment
        .unwrap_or_else(empty_function_environment);
    let mut fn_env = Env::function_frame(captured);
    fn_env.new_target = callee.clone();
    fn_env.declare_local("this".to_owned(), Value::Object(instance));
    fn_env.declare_local("arguments".to_owned(), create_arguments_object(cx, args)?);
    for (index, param) in script.params.iter().enumerate() {
        fn_env.declare_local(
            param.clone(),
            args.get(index).cloned().unwrap_or(Value::Undefined),
        );
    }
    match eval_block(cx, &mut fn_env, script.body)? {
        Flow::Return(Value::Object(object)) => Ok(Value::Object(object)),
        Flow::Return(_) | Flow::Normal(_) => Ok(Value::Object(instance)),
        Flow::Break | Flow::Continue => Err(JsError::syntax(
            "break/continue cannot escape constructor body",
        )),
        Flow::Throw(value) => Err(JsError::throw_value(value)),
    }
}

fn non_strict_this_value(cx: &mut Context, env: &Env, value: Value) -> Completion<Value> {
    if env.strict {
        return Ok(value);
    }
    match value {
        Value::Undefined | Value::Null => Ok(Value::Object(
            env.global_object().unwrap_or(cx.realm()?.global_object),
        )),
        Value::Boolean(_) | Value::Number(_) | Value::String(_) | Value::Symbol(_) => {
            Ok(Value::Object(ArgView::new(value).to_object(cx)?))
        }
        value => Ok(value),
    }
}

fn create_arguments_object(cx: &mut Context, args: &[Value]) -> Completion<Value> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(crate::runtime::IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::arguments(Some(proto)));
    for (index, value) in args.iter().enumerate() {
        object.define_own_property_or_throw(
            cx,
            PropertyKey::array_index(index as u64),
            Descriptor::data(value.clone(), true, true, true),
        )?;
    }
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from("length"),
        Descriptor::data(Value::Number(args.len() as f64), true, false, true),
    )?;
    Ok(Value::Object(object))
}

fn create_script_function(
    cx: &mut Context,
    env: &Env,
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
) -> Completion<Value> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(crate::runtime::IntrinsicId::FunctionPrototype)
        .ok_or_else(|| JsError::internal("missing Function.prototype intrinsic"))?;
    let function = cx.heap_mut().allocate(JsObject::function(Some(proto), {
        let mut data =
            FunctionData::script_with_environment(name.clone(), params, body, env.capture());
        data.constructible = true;
        data
    }));
    let object_proto = cx
        .realm()?
        .intrinsics
        .get(crate::runtime::IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let prototype_object = cx
        .heap_mut()
        .allocate(JsObject::ordinary(Some(object_proto)));
    prototype_object.define_own_property_or_throw(
        cx,
        PropertyKey::from("constructor"),
        Descriptor::data(Value::Object(function), true, false, true),
    )?;
    define_data(
        cx,
        function,
        "prototype",
        Value::Object(prototype_object),
        true,
    )?;
    define_function_metadata(
        cx,
        function,
        "length",
        Value::Number(function_length(cx, function)? as f64),
    )?;
    define_function_metadata(cx, function, "name", Value::String(name))?;
    Ok(Value::Object(function))
}

fn empty_function_environment() -> FunctionEnvironment {
    FunctionEnvironment {
        bindings: Rc::new(RefCell::new(HashMap::new())),
        global: None,
        global_names: Rc::new(RefCell::new(HashSet::new())),
        strict: false,
    }
}

fn to_number(cx: &mut Context, value: Value) -> Completion<f64> {
    ArgView::new(value).to_number(cx)
}

fn eval_assignment_target(
    cx: &mut Context,
    env: &mut Env,
    left: Expr,
) -> Completion<AssignmentTarget> {
    match left {
        Expr::Identifier(name) => {
            let object = if env.is_shadow_name(&name) {
                None
            } else {
                env.with_binding_object(cx, &name)?
            };
            Ok(AssignmentTarget::Identifier(name, object))
        }
        Expr::Undefined => Ok(AssignmentTarget::Ignored),
        Expr::Member(base, key) => {
            let base = eval_expr(cx, env, *base)?;
            let Value::Object(object) = base else {
                return Err(JsError::type_error("assignment target base must be object"));
            };
            let key = ArgView::new(eval_expr(cx, env, *key)?).to_property_key(cx)?;
            Ok(AssignmentTarget::Member(object, key))
        }
        _ => Err(JsError::syntax("invalid assignment target")),
    }
}

fn assign_to_target(
    cx: &mut Context,
    env: &mut Env,
    target: AssignmentTarget,
    value: Value,
) -> Completion<()> {
    match target {
        AssignmentTarget::Identifier(name, Some(object)) => {
            let ok = object.set(
                cx,
                PropertyKey::from(name.as_str()),
                value,
                Value::Object(object),
            )?;
            env.require_set_success(ok)?;
            Ok(())
        }
        AssignmentTarget::Identifier(name, None) => env.set_identifier_binding(cx, name, value),
        AssignmentTarget::Member(object, key) => {
            let ok = object.set(cx, key, value, Value::Object(object))?;
            env.require_set_success(ok)?;
            Ok(())
        }
        AssignmentTarget::Ignored => Ok(()),
    }
}

fn eval_callee(cx: &mut Context, env: &mut Env, expr: Expr) -> Completion<(Value, Value)> {
    match expr {
        Expr::Member(base, key) => {
            let base = eval_expr(cx, env, *base)?;
            let object = ArgView::new(base.clone()).to_object(cx)?;
            let key = ArgView::new(eval_expr(cx, env, *key)?).to_property_key(cx)?;
            let function = object.get(cx, &key, base.clone())?;
            Ok((function, base))
        }
        Expr::Identifier(name) => {
            if let Some(object) = env.with_binding_object(cx, &name)? {
                let function =
                    object.get(cx, &PropertyKey::from(name.as_str()), Value::Object(object))?;
                Ok((function, Value::Object(object)))
            } else {
                Ok((resolve_identifier(cx, env, &name)?, Value::Undefined))
            }
        }
        other => Ok((eval_expr(cx, env, other)?, Value::Undefined)),
    }
}

fn assign(cx: &mut Context, env: &mut Env, left: Expr, value: Value) -> Completion<()> {
    match left {
        Expr::Identifier(name) => {
            env.set_identifier(cx, name, value)?;
            Ok(())
        }
        Expr::Undefined => Ok(()),
        Expr::Member(base, key) => {
            let base = eval_expr(cx, env, *base)?;
            let Value::Object(object) = base else {
                return Err(JsError::type_error("assignment target base must be object"));
            };
            let key = ArgView::new(eval_expr(cx, env, *key)?).to_property_key(cx)?;
            let ok = object.set(cx, key, value, Value::Object(object))?;
            env.require_set_success(ok)?;
            Ok(())
        }
        _ => Err(JsError::syntax("invalid assignment target")),
    }
}

fn resolve_identifier(cx: &mut Context, env: &Env, name: &str) -> Completion<Value> {
    if env.is_shadow_name(name) {
        if let Some(value) = env.get(name) {
            return Ok(value);
        }
    }
    if env.has_with_object() {
        if let Some(object) = env.with_binding_object(cx, name)? {
            return object.get(cx, &PropertyKey::from(name), Value::Object(object));
        }
    } else if env.is_local_name(name) && !(env.is_global_frame && env.is_global_name(name)) {
        if let Some(value) = env.get(name) {
            return Ok(value);
        }
    }
    if env.is_global_name(name) {
        if let Some(global) = env.global_object() {
            return global.get(cx, &PropertyKey::from(name), Value::Object(global));
        }
    }
    if let Some(value) = env.get(name) {
        return Ok(value.clone());
    }
    let global = cx.realm()?.global_object;
    let value = global.get(cx, &PropertyKey::from(name), Value::Object(global))?;
    if matches!(value, Value::Undefined) && name != "undefined" {
        return Err(JsError::reference(format!("{name} is not defined")));
    }
    Ok(value)
}

fn define_data(
    cx: &mut Context,
    object: crate::runtime::ObjectRef,
    name: &str,
    value: Value,
    writable: bool,
) -> Completion<()> {
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from(name),
        Descriptor::data(value, writable, false, false),
    )
}

fn define_function_metadata(
    cx: &mut Context,
    object: crate::runtime::ObjectRef,
    name: &str,
    value: Value,
) -> Completion<()> {
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from(name),
        Descriptor::data(value, false, false, true),
    )
}

fn function_length(cx: &Context, object: crate::runtime::ObjectRef) -> Completion<u32> {
    let ObjectKind::Function(data) = &cx.heap().get(object)?.kind else {
        return Ok(0);
    };
    Ok(data.length)
}

fn js_error_to_value(cx: &mut Context, error: JsError) -> Completion<Value> {
    if matches!(error.kind, crate::runtime::ErrorKind::Throw) {
        return Ok(error.thrown.unwrap_or(Value::Undefined));
    }
    let (name, proto_id) = match error.kind {
        crate::runtime::ErrorKind::Type => {
            ("TypeError", crate::runtime::IntrinsicId::TypeErrorPrototype)
        }
        crate::runtime::ErrorKind::Range => (
            "RangeError",
            crate::runtime::IntrinsicId::RangeErrorPrototype,
        ),
        crate::runtime::ErrorKind::Reference => (
            "ReferenceError",
            crate::runtime::IntrinsicId::ReferenceErrorPrototype,
        ),
        crate::runtime::ErrorKind::Syntax => (
            "SyntaxError",
            crate::runtime::IntrinsicId::SyntaxErrorPrototype,
        ),
        crate::runtime::ErrorKind::URI => {
            ("URIError", crate::runtime::IntrinsicId::URIErrorPrototype)
        }
        crate::runtime::ErrorKind::Throw => ("Error", crate::runtime::IntrinsicId::ErrorPrototype),
        crate::runtime::ErrorKind::Internal => {
            ("Error", crate::runtime::IntrinsicId::ErrorPrototype)
        }
    };
    let proto = cx
        .realm()?
        .intrinsics
        .get(proto_id)
        .ok_or_else(|| JsError::internal("missing error prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from("name"),
        Descriptor::data(Value::String(name.to_owned()), true, false, true),
    )?;
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from("message"),
        Descriptor::data(Value::String(error.message), true, false, true),
    )?;
    Ok(Value::Object(object))
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
