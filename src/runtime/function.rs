use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::syntax::parser::Stmt;

use super::{Completion, Context, ObjectRef, Value};

pub type BuiltinFn = fn(&mut Context, Value, &[Value]) -> Completion<Value>;
pub type BindingCell = Rc<RefCell<Value>>;

#[derive(Clone, Debug)]
pub struct FunctionEnvironment {
    pub bindings: Rc<RefCell<HashMap<String, BindingCell>>>,
    pub global: Option<ObjectRef>,
    pub global_names: Rc<RefCell<HashSet<String>>>,
    pub strict: bool,
}

#[derive(Clone, Debug)]
pub struct FunctionData {
    pub name: String,
    pub length: u32,
    pub callable: bool,
    pub constructible: bool,
    pub builtin: Option<BuiltinFn>,
    pub script: Option<ScriptFunction>,
    pub bound: Option<BoundFunction>,
}

#[derive(Clone, Debug)]
pub struct ScriptFunction {
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub environment: Option<FunctionEnvironment>,
}

#[derive(Clone, Debug)]
pub struct BoundFunction {
    pub target: Value,
    pub this_value: Value,
    pub args: Vec<Value>,
}

impl FunctionData {
    pub fn builtin(name: impl Into<String>, length: u32, function: BuiltinFn) -> Self {
        Self {
            name: name.into(),
            length,
            callable: true,
            constructible: false,
            builtin: Some(function),
            script: None,
            bound: None,
        }
    }

    pub fn script(name: impl Into<String>, params: Vec<String>, body: Vec<Stmt>) -> Self {
        let length = params.len() as u32;
        Self {
            name: name.into(),
            length,
            callable: true,
            constructible: true,
            builtin: None,
            script: Some(ScriptFunction {
                params,
                body,
                environment: None,
            }),
            bound: None,
        }
    }

    pub fn script_with_environment(
        name: impl Into<String>,
        params: Vec<String>,
        body: Vec<Stmt>,
        environment: FunctionEnvironment,
    ) -> Self {
        let mut data = Self::script(name, params, body);
        if let Some(script) = &mut data.script {
            script.environment = Some(environment);
        }
        data
    }

    pub fn bound(
        name: impl Into<String>,
        length: u32,
        target: Value,
        this_value: Value,
        args: Vec<Value>,
    ) -> Self {
        Self {
            name: name.into(),
            length,
            callable: true,
            constructible: false,
            builtin: None,
            script: None,
            bound: Some(BoundFunction {
                target,
                this_value,
                args,
            }),
        }
    }
}
