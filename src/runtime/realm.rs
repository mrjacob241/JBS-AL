use std::collections::HashMap;

use super::ObjectRef;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct RealmId(pub(crate) usize);

impl RealmId {
    pub fn index(self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum IntrinsicId {
    ObjectPrototype,
    FunctionPrototype,
    ArrayPrototype,
    IteratorPrototype,
    ArrayIteratorPrototype,
    StringIteratorPrototype,
    MapPrototype,
    SetPrototype,
    WeakMapPrototype,
    WeakSetPrototype,
    MapIteratorPrototype,
    SetIteratorPrototype,
    BooleanPrototype,
    NumberPrototype,
    BigIntPrototype,
    StringPrototype,
    SymbolPrototype,
    RegExpPrototype,
    DatePrototype,
    ErrorPrototype,
    TypeErrorPrototype,
    RangeErrorPrototype,
    ReferenceErrorPrototype,
    SyntaxErrorPrototype,
    EvalErrorPrototype,
    URIErrorPrototype,
    Test262ErrorPrototype,
    AggregateErrorPrototype,
    ObjectConstructor,
    FunctionConstructor,
    ArrayConstructor,
    IteratorConstructor,
    MapConstructor,
    SetConstructor,
    WeakMapConstructor,
    WeakSetConstructor,
    ProxyConstructor,
    BooleanConstructor,
    NumberConstructor,
    BigIntConstructor,
    StringConstructor,
    SymbolConstructor,
    RegExpConstructor,
    DateConstructor,
    ErrorConstructor,
    TypeErrorConstructor,
    RangeErrorConstructor,
    ReferenceErrorConstructor,
    SyntaxErrorConstructor,
    EvalErrorConstructor,
    URIErrorConstructor,
    Test262ErrorConstructor,
    AggregateErrorConstructor,
}

#[derive(Clone, Debug, Default)]
pub struct IntrinsicRegistry {
    objects: HashMap<IntrinsicId, ObjectRef>,
}

impl IntrinsicRegistry {
    pub fn set(&mut self, id: IntrinsicId, object: ObjectRef) {
        self.objects.insert(id, object);
    }

    pub fn get(&self, id: IntrinsicId) -> Option<ObjectRef> {
        self.objects.get(&id).copied()
    }
}

#[derive(Clone, Debug)]
pub struct Realm {
    pub intrinsics: IntrinsicRegistry,
    pub global_object: ObjectRef,
}

impl Realm {
    pub fn new(global_object: ObjectRef) -> Self {
        Self {
            intrinsics: IntrinsicRegistry::default(),
            global_object,
        }
    }
}
