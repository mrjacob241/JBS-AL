use super::object::ObjectRef;

#[derive(Clone, Debug)]
pub enum Value {
    InternalConstruct,
    InternalConstructWithNewTarget(ObjectRef),
    Undefined,
    Null,
    Boolean(bool),
    String(String),
    Symbol(u64),
    Number(f64),
    BigInt(i128),
    Object(ObjectRef),
}

impl Value {
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => "Internal",
            Value::Undefined => "Undefined",
            Value::Null => "Null",
            Value::Boolean(_) => "Boolean",
            Value::String(_) => "String",
            Value::Symbol(_) => "Symbol",
            Value::Number(_) => "Number",
            Value::BigInt(_) => "BigInt",
            Value::Object(_) => "Object",
        }
    }
}

#[allow(non_snake_case)]
pub fn SameValue(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::InternalConstruct, Value::InternalConstruct) => true,
        (
            Value::InternalConstructWithNewTarget(left),
            Value::InternalConstructWithNewTarget(right),
        ) => left == right,
        (Value::Undefined, Value::Undefined) | (Value::Null, Value::Null) => true,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Symbol(a), Value::Symbol(b)) => a == b,
        (Value::BigInt(a), Value::BigInt(b)) => a == b,
        (Value::Object(a), Value::Object(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => same_value_number(*a, *b),
        _ => false,
    }
}

#[allow(non_snake_case)]
pub fn SameValueZero(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            if a.is_nan() && b.is_nan() {
                true
            } else {
                a == b
            }
        }
        _ => SameValue(left, right),
    }
}

fn same_value_number(left: f64, right: f64) -> bool {
    if left.is_nan() && right.is_nan() {
        return true;
    }
    if left == 0.0 && right == 0.0 {
        return left.to_bits() == right.to_bits();
    }
    left == right
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        SameValue(self, other)
    }
}
