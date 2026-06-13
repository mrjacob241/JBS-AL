use crate::syntax::parser::{parse_script, Stmt};

use std::collections::{HashMap, HashSet};

use super::{
    get_iterator, iterator_close_error, iterator_step_value, number_to_property_string,
    primitive_wrapper_value, proxy_target, regexp_source_flags, ArgView, Brand, CollectionEntry,
    CollectionIteratorKind, CollectionKind, Completion, Context, CreateArrayFromList,
    CreateDataPropertyOrThrow, Descriptor, FromPropertyDescriptor, FunctionData, Heap,
    InternalMethods, InternalSlot, IntrinsicId, IteratorRecord, JsError, JsObject,
    LengthOfArrayLike, ObjectKind, ObjectRef, PropertyKey, Realm, RealmId, RegExpCreate, SameValue,
    SameValueZero, ToNumber, ToPropertyDescriptor, ToString, Value, SYMBOL_IS_CONCAT_SPREADABLE_ID,
    SYMBOL_ITERATOR_ID, SYMBOL_REPLACE_ID, SYMBOL_SPECIES_ID, SYMBOL_TO_PRIMITIVE_ID,
    SYMBOL_TO_STRING_TAG_ID, SYMBOL_UNSCOPABLES_ID,
};

pub struct Runtime {
    pub(crate) heap: Heap,
    pub(crate) realms: Vec<Realm>,
    default_realm: RealmId,
    pub(crate) next_symbol_id: u64,
    pub(crate) symbol_descriptions: HashMap<u64, Option<String>>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut runtime = Self {
            heap: Heap::new(),
            realms: Vec::new(),
            default_realm: RealmId(0),
            next_symbol_id: 1000,
            symbol_descriptions: HashMap::new(),
        };
        runtime
            .symbol_descriptions
            .insert(SYMBOL_ITERATOR_ID, Some("iterator".to_owned()));
        runtime
            .symbol_descriptions
            .insert(SYMBOL_REPLACE_ID, Some("replace".to_owned()));
        runtime
            .symbol_descriptions
            .insert(SYMBOL_SPECIES_ID, Some("species".to_owned()));
        runtime
            .symbol_descriptions
            .insert(SYMBOL_TO_PRIMITIVE_ID, Some("toPrimitive".to_owned()));
        runtime.symbol_descriptions.insert(
            SYMBOL_IS_CONCAT_SPREADABLE_ID,
            Some("isConcatSpreadable".to_owned()),
        );
        runtime
            .symbol_descriptions
            .insert(SYMBOL_TO_STRING_TAG_ID, Some("toStringTag".to_owned()));
        runtime
            .symbol_descriptions
            .insert(SYMBOL_UNSCOPABLES_ID, Some("unscopables".to_owned()));
        let realm = runtime.create_realm();
        runtime.default_realm = realm;
        runtime
    }

    pub fn default_realm(&self) -> RealmId {
        self.default_realm
    }

    pub fn create_realm(&mut self) -> RealmId {
        let object_proto = self.heap.allocate(JsObject::ordinary(None));
        let function_proto = self.heap.allocate(JsObject::function(
            Some(object_proto),
            FunctionData {
                name: String::new(),
                length: 0,
                callable: false,
                constructible: false,
                builtin: None,
                script: None,
                bound: None,
            },
        ));
        let iterator_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let array_iterator_proto = self.heap.allocate(JsObject::ordinary(Some(iterator_proto)));
        let string_iterator_proto = self.heap.allocate(JsObject::ordinary(Some(iterator_proto)));
        let array_proto = self.heap.allocate(JsObject::array(Some(object_proto)));
        let map_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let set_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let weak_map_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let weak_set_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let map_iterator_proto = self.heap.allocate(JsObject::ordinary(Some(iterator_proto)));
        let set_iterator_proto = self.heap.allocate(JsObject::ordinary(Some(iterator_proto)));
        let boolean_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let number_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let bigint_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let string_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let symbol_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let regexp_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let date_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let error_proto = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let type_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let range_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let reference_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let syntax_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let eval_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let uri_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let test262_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let aggregate_error_proto = self.heap.allocate(JsObject::ordinary(Some(error_proto)));
        let global = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let math_obj = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let json_obj = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let reflect_obj = self.heap.allocate(JsObject::ordinary(Some(object_proto)));
        let object_ctor =
            self.create_builtin_function("Object", 1, Some(function_proto), object_constructor);
        let function_ctor =
            self.create_builtin_function("Function", 1, Some(function_proto), function_constructor);
        let array_ctor =
            self.create_builtin_function("Array", 1, Some(function_proto), array_constructor);
        self.mark_constructible(array_ctor);
        let iterator_ctor =
            self.create_builtin_function("Iterator", 0, Some(function_proto), iterator_constructor);
        let map_ctor =
            self.create_builtin_function("Map", 0, Some(function_proto), map_constructor);
        self.mark_constructible(map_ctor);
        let set_ctor =
            self.create_builtin_function("Set", 0, Some(function_proto), set_constructor);
        self.mark_constructible(set_ctor);
        let weak_map_ctor =
            self.create_builtin_function("WeakMap", 0, Some(function_proto), weak_map_constructor);
        self.mark_constructible(weak_map_ctor);
        let weak_set_ctor =
            self.create_builtin_function("WeakSet", 0, Some(function_proto), weak_set_constructor);
        self.mark_constructible(weak_set_ctor);
        let proxy_ctor =
            self.create_builtin_function("Proxy", 2, Some(function_proto), proxy_constructor);
        self.mark_constructible(proxy_ctor);
        let boolean_ctor =
            self.create_builtin_function("Boolean", 1, Some(function_proto), boolean_constructor);
        self.mark_constructible(boolean_ctor);
        let number_ctor =
            self.create_builtin_function("Number", 1, Some(function_proto), number_constructor);
        self.mark_constructible(number_ctor);
        let bigint_ctor =
            self.create_builtin_function("BigInt", 1, Some(function_proto), bigint_constructor);
        self.install_bigint_statics(bigint_ctor, function_proto);
        let string_ctor =
            self.create_builtin_function("String", 1, Some(function_proto), string_constructor);
        self.mark_constructible(string_ctor);
        let symbol_ctor =
            self.create_builtin_function("Symbol", 0, Some(function_proto), symbol_constructor);
        let regexp_ctor =
            self.create_builtin_function("RegExp", 2, Some(function_proto), regexp_constructor);
        self.mark_constructible(regexp_ctor);
        let date_ctor =
            self.create_builtin_function("Date", 7, Some(function_proto), date_constructor);
        self.mark_constructible(date_ctor);
        let error_ctor =
            self.create_builtin_function("Error", 1, Some(function_proto), error_constructor);
        self.mark_constructible(error_ctor);
        let type_error_ctor = self.create_builtin_function(
            "TypeError",
            1,
            Some(function_proto),
            type_error_constructor,
        );
        self.mark_constructible(type_error_ctor);
        let range_error_ctor = self.create_builtin_function(
            "RangeError",
            1,
            Some(function_proto),
            range_error_constructor,
        );
        self.mark_constructible(range_error_ctor);
        let reference_error_ctor = self.create_builtin_function(
            "ReferenceError",
            1,
            Some(function_proto),
            reference_error_constructor,
        );
        self.mark_constructible(reference_error_ctor);
        let syntax_error_ctor = self.create_builtin_function(
            "SyntaxError",
            1,
            Some(function_proto),
            syntax_error_constructor,
        );
        self.mark_constructible(syntax_error_ctor);
        let eval_error_ctor = self.create_builtin_function(
            "EvalError",
            1,
            Some(function_proto),
            eval_error_constructor,
        );
        self.mark_constructible(eval_error_ctor);
        let uri_error_ctor = self.create_builtin_function(
            "URIError",
            1,
            Some(function_proto),
            uri_error_constructor,
        );
        self.mark_constructible(uri_error_ctor);
        let test262_error_ctor = self.create_builtin_function(
            "Test262Error",
            1,
            Some(function_proto),
            test262_error_constructor,
        );
        self.mark_constructible(test262_error_ctor);
        let aggregate_error_ctor = self.create_builtin_function(
            "AggregateError",
            2,
            Some(function_proto),
            aggregate_error_constructor,
        );
        self.mark_constructible(aggregate_error_ctor);
        self.mark_constructible(object_ctor);
        self.mark_constructible(function_ctor);

        let mut realm = Realm::new(global);
        realm
            .intrinsics
            .set(IntrinsicId::ObjectPrototype, object_proto);
        realm
            .intrinsics
            .set(IntrinsicId::FunctionPrototype, function_proto);
        realm
            .intrinsics
            .set(IntrinsicId::ArrayPrototype, array_proto);
        realm
            .intrinsics
            .set(IntrinsicId::IteratorPrototype, iterator_proto);
        realm
            .intrinsics
            .set(IntrinsicId::ArrayIteratorPrototype, array_iterator_proto);
        realm
            .intrinsics
            .set(IntrinsicId::StringIteratorPrototype, string_iterator_proto);
        realm.intrinsics.set(IntrinsicId::MapPrototype, map_proto);
        realm.intrinsics.set(IntrinsicId::SetPrototype, set_proto);
        realm
            .intrinsics
            .set(IntrinsicId::WeakMapPrototype, weak_map_proto);
        realm
            .intrinsics
            .set(IntrinsicId::WeakSetPrototype, weak_set_proto);
        realm
            .intrinsics
            .set(IntrinsicId::MapIteratorPrototype, map_iterator_proto);
        realm
            .intrinsics
            .set(IntrinsicId::SetIteratorPrototype, set_iterator_proto);
        realm
            .intrinsics
            .set(IntrinsicId::BooleanPrototype, boolean_proto);
        realm
            .intrinsics
            .set(IntrinsicId::NumberPrototype, number_proto);
        realm
            .intrinsics
            .set(IntrinsicId::BigIntPrototype, bigint_proto);
        realm
            .intrinsics
            .set(IntrinsicId::StringPrototype, string_proto);
        realm
            .intrinsics
            .set(IntrinsicId::SymbolPrototype, symbol_proto);
        realm
            .intrinsics
            .set(IntrinsicId::RegExpPrototype, regexp_proto);
        realm.intrinsics.set(IntrinsicId::DatePrototype, date_proto);
        realm
            .intrinsics
            .set(IntrinsicId::ErrorPrototype, error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::TypeErrorPrototype, type_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::RangeErrorPrototype, range_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::ReferenceErrorPrototype, reference_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::SyntaxErrorPrototype, syntax_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::EvalErrorPrototype, eval_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::URIErrorPrototype, uri_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::Test262ErrorPrototype, test262_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::AggregateErrorPrototype, aggregate_error_proto);
        realm
            .intrinsics
            .set(IntrinsicId::ObjectConstructor, object_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::FunctionConstructor, function_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::ArrayConstructor, array_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::IteratorConstructor, iterator_ctor);
        realm.intrinsics.set(IntrinsicId::MapConstructor, map_ctor);
        realm.intrinsics.set(IntrinsicId::SetConstructor, set_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::WeakMapConstructor, weak_map_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::WeakSetConstructor, weak_set_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::ProxyConstructor, proxy_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::BooleanConstructor, boolean_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::NumberConstructor, number_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::BigIntConstructor, bigint_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::StringConstructor, string_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::SymbolConstructor, symbol_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::RegExpConstructor, regexp_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::DateConstructor, date_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::ErrorConstructor, error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::TypeErrorConstructor, type_error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::RangeErrorConstructor, range_error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::ReferenceErrorConstructor, reference_error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::SyntaxErrorConstructor, syntax_error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::EvalErrorConstructor, eval_error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::URIErrorConstructor, uri_error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::Test262ErrorConstructor, test262_error_ctor);
        realm
            .intrinsics
            .set(IntrinsicId::AggregateErrorConstructor, aggregate_error_ctor);

        self.install_object_statics(object_ctor, function_proto);
        self.install_object_prototype(object_proto, function_proto);
        self.install_function_prototype(function_proto);
        self.install_iterator_statics(iterator_ctor, function_proto);
        self.install_iterator_prototype(iterator_proto, function_proto);
        self.install_array_iterator_prototype(array_iterator_proto, function_proto);
        self.install_string_iterator_prototype(string_iterator_proto, function_proto);
        self.install_map_prototype(map_proto, function_proto);
        self.install_set_prototype(set_proto, function_proto);
        self.install_weak_map_prototype(weak_map_proto, function_proto);
        self.install_weak_set_prototype(weak_set_proto, function_proto);
        self.install_map_iterator_prototype(map_iterator_proto, function_proto);
        self.install_set_iterator_prototype(set_iterator_proto, function_proto);
        self.install_array_statics(array_ctor, function_proto);
        self.install_array_prototype(array_proto, function_proto);
        self.install_boolean_prototype(boolean_proto, function_proto);
        self.install_number_statics(number_ctor, function_proto);
        self.install_number_prototype(number_proto, function_proto);
        self.install_bigint_prototype(bigint_proto, bigint_ctor, function_proto);
        self.install_string_statics(string_ctor, function_proto);
        self.install_string_prototype(string_proto, function_proto);
        self.install_symbol_prototype(symbol_proto, function_proto);
        self.install_symbol_statics(symbol_ctor);
        self.install_regexp_prototype(regexp_proto, function_proto);
        self.install_date_statics(date_ctor, function_proto);
        self.install_date_prototype(date_proto, function_proto);
        self.install_math_object(math_obj, function_proto);
        self.install_json_object(json_obj, function_proto);
        self.install_reflect_object(reflect_obj, function_proto);
        self.install_proxy_constructor(proxy_ctor, function_proto);
        self.install_error_statics(error_ctor, function_proto);
        self.install_error_prototype(error_proto, function_proto);
        self.install_error_prototype(type_error_proto, function_proto);
        self.install_error_prototype(range_error_proto, function_proto);
        self.install_error_prototype(reference_error_proto, function_proto);
        self.install_error_prototype(syntax_error_proto, function_proto);
        self.install_error_prototype(eval_error_proto, function_proto);
        self.install_error_prototype(uri_error_proto, function_proto);
        self.install_error_prototype(test262_error_proto, function_proto);
        self.install_error_prototype(aggregate_error_proto, function_proto);
        self.install_test262_helpers(global, object_proto, function_proto);
        define_data(
            &mut self.heap,
            object_ctor,
            "prototype",
            Value::Object(object_proto),
            false,
        );
        define_data(
            &mut self.heap,
            function_ctor,
            "prototype",
            Value::Object(function_proto),
            false,
        );
        define_data(
            &mut self.heap,
            array_ctor,
            "prototype",
            Value::Object(array_proto),
            false,
        );
        define_data(
            &mut self.heap,
            iterator_ctor,
            "prototype",
            Value::Object(iterator_proto),
            false,
        );
        define_data(
            &mut self.heap,
            map_ctor,
            "prototype",
            Value::Object(map_proto),
            false,
        );
        define_data(
            &mut self.heap,
            set_ctor,
            "prototype",
            Value::Object(set_proto),
            false,
        );
        define_data(
            &mut self.heap,
            weak_map_ctor,
            "prototype",
            Value::Object(weak_map_proto),
            false,
        );
        define_data(
            &mut self.heap,
            weak_set_ctor,
            "prototype",
            Value::Object(weak_set_proto),
            false,
        );
        define_data(
            &mut self.heap,
            array_proto,
            "length",
            Value::Number(0.0),
            true,
        );
        define_data(
            &mut self.heap,
            boolean_ctor,
            "prototype",
            Value::Object(boolean_proto),
            false,
        );
        define_data(
            &mut self.heap,
            number_ctor,
            "prototype",
            Value::Object(number_proto),
            false,
        );
        define_data(
            &mut self.heap,
            bigint_ctor,
            "prototype",
            Value::Object(bigint_proto),
            false,
        );
        define_number_constants(&mut self.heap, number_ctor);
        define_data(
            &mut self.heap,
            string_ctor,
            "prototype",
            Value::Object(string_proto),
            false,
        );
        define_data(
            &mut self.heap,
            symbol_ctor,
            "prototype",
            Value::Object(symbol_proto),
            false,
        );
        define_data(
            &mut self.heap,
            regexp_ctor,
            "prototype",
            Value::Object(regexp_proto),
            false,
        );
        define_data(
            &mut self.heap,
            date_ctor,
            "prototype",
            Value::Object(date_proto),
            false,
        );
        define_data_with_attrs(
            &mut self.heap,
            object_proto,
            "constructor",
            Value::Object(object_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            function_proto,
            "constructor",
            Value::Object(function_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            array_proto,
            "constructor",
            Value::Object(array_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            boolean_proto,
            "constructor",
            Value::Object(boolean_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            number_proto,
            "constructor",
            Value::Object(number_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            string_proto,
            "constructor",
            Value::Object(string_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            symbol_proto,
            "constructor",
            Value::Object(symbol_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            iterator_proto,
            "constructor",
            Value::Object(iterator_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            map_proto,
            "constructor",
            Value::Object(map_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            set_proto,
            "constructor",
            Value::Object(set_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            weak_map_proto,
            "constructor",
            Value::Object(weak_map_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            weak_set_proto,
            "constructor",
            Value::Object(weak_set_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            regexp_proto,
            "constructor",
            Value::Object(regexp_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            date_proto,
            "constructor",
            Value::Object(date_ctor),
            true,
            false,
            true,
        );
        define_data(
            &mut self.heap,
            error_ctor,
            "prototype",
            Value::Object(error_proto),
            false,
        );
        define_data_with_attrs(
            &mut self.heap,
            error_proto,
            "constructor",
            Value::Object(error_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            error_proto,
            "name",
            Value::String("Error".to_owned()),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            error_proto,
            "message",
            Value::String(String::new()),
            true,
            false,
            true,
        );

        let native_errors = [
            NativeErrorMetadata {
                name: "TypeError",
                constructor: type_error_ctor,
                prototype: type_error_proto,
            },
            NativeErrorMetadata {
                name: "RangeError",
                constructor: range_error_ctor,
                prototype: range_error_proto,
            },
            NativeErrorMetadata {
                name: "ReferenceError",
                constructor: reference_error_ctor,
                prototype: reference_error_proto,
            },
            NativeErrorMetadata {
                name: "SyntaxError",
                constructor: syntax_error_ctor,
                prototype: syntax_error_proto,
            },
            NativeErrorMetadata {
                name: "EvalError",
                constructor: eval_error_ctor,
                prototype: eval_error_proto,
            },
            NativeErrorMetadata {
                name: "URIError",
                constructor: uri_error_ctor,
                prototype: uri_error_proto,
            },
            NativeErrorMetadata {
                name: "Test262Error",
                constructor: test262_error_ctor,
                prototype: test262_error_proto,
            },
            NativeErrorMetadata {
                name: "AggregateError",
                constructor: aggregate_error_ctor,
                prototype: aggregate_error_proto,
            },
        ];
        define_native_error_properties(&mut self.heap, error_ctor, &native_errors);

        define_data(&mut self.heap, global, "undefined", Value::Undefined, false);
        define_data(
            &mut self.heap,
            global,
            "NaN",
            Value::Number(f64::NAN),
            false,
        );
        define_data(
            &mut self.heap,
            global,
            "Infinity",
            Value::Number(f64::INFINITY),
            false,
        );
        define_data(
            &mut self.heap,
            global,
            "globalThis",
            Value::Object(global),
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Object",
            Value::Object(object_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Function",
            Value::Object(function_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Array",
            Value::Object(array_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Iterator",
            Value::Object(iterator_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Map",
            Value::Object(map_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Set",
            Value::Object(set_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "WeakMap",
            Value::Object(weak_map_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "WeakSet",
            Value::Object(weak_set_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Proxy",
            Value::Object(proxy_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Boolean",
            Value::Object(boolean_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Number",
            Value::Object(number_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "BigInt",
            Value::Object(bigint_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "String",
            Value::Object(string_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Symbol",
            Value::Object(symbol_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "RegExp",
            Value::Object(regexp_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Date",
            Value::Object(date_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Math",
            Value::Object(math_obj),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "JSON",
            Value::Object(json_obj),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Reflect",
            Value::Object(reflect_obj),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "Error",
            Value::Object(error_ctor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "TypeError",
            Value::Object(type_error_ctor),
            true,
            false,
            true,
        );
        for (name, ctor) in [
            ("RangeError", range_error_ctor),
            ("ReferenceError", reference_error_ctor),
            ("SyntaxError", syntax_error_ctor),
            ("EvalError", eval_error_ctor),
            ("URIError", uri_error_ctor),
            ("Test262Error", test262_error_ctor),
            ("AggregateError", aggregate_error_ctor),
        ] {
            define_data_with_attrs(
                &mut self.heap,
                global,
                name,
                Value::Object(ctor),
                true,
                false,
                true,
            );
        }
        self.install_global_functions(global, function_proto);
        self.install_number_parse_aliases(number_ctor, global);

        let id = RealmId(self.realms.len());
        self.realms.push(realm);
        id
    }

    pub fn realm(&self, realm: RealmId) -> Option<&Realm> {
        self.realms.get(realm.0)
    }

    pub fn heap(&self) -> &Heap {
        &self.heap
    }

    pub fn heap_mut(&mut self) -> &mut Heap {
        &mut self.heap
    }

    pub fn eval_script(&mut self, source: &str) -> Completion<Value> {
        crate::syntax::eval_script(self, source)
    }

    fn create_builtin_function(
        &mut self,
        name: &str,
        length: u32,
        prototype: Option<ObjectRef>,
        body: super::BuiltinFn,
    ) -> ObjectRef {
        let function = self.heap.allocate(JsObject::function(
            prototype,
            FunctionData::builtin(name, length, body),
        ));

        define_data_with_attrs(
            &mut self.heap,
            function,
            "length",
            Value::Number(length as f64),
            false,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            function,
            "name",
            Value::String(name.to_owned()),
            false,
            false,
            true,
        );
        function
    }

    fn mark_constructible(&mut self, function: ObjectRef) {
        if let Ok(object) = self.heap.get_mut(function) {
            if let super::ObjectKind::Function(data) = &mut object.kind {
                data.constructible = true;
            }
        }
    }

    fn install_object_statics(&mut self, object_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("create", 2, object_create),
            ("defineProperty", 3, object_define_property),
            (
                "getOwnPropertyDescriptor",
                2,
                object_get_own_property_descriptor,
            ),
            ("getPrototypeOf", 1, object_get_prototype_of),
            ("setPrototypeOf", 2, object_set_prototype_of),
            ("preventExtensions", 1, object_prevent_extensions),
            ("isExtensible", 1, object_is_extensible),
            ("hasOwn", 2, object_has_own),
            ("is", 2, object_is),
            ("keys", 1, object_keys),
            ("values", 1, object_values),
            ("entries", 1, object_entries),
            ("fromEntries", 1, object_from_entries),
            ("groupBy", 2, object_group_by),
            ("defineProperties", 2, object_define_properties),
            (
                "getOwnPropertyDescriptors",
                1,
                object_get_own_property_descriptors,
            ),
            ("getOwnPropertyNames", 1, object_get_own_property_names),
            ("getOwnPropertySymbols", 1, object_get_own_property_symbols),
            ("assign", 2, object_assign),
            ("freeze", 1, object_freeze),
            ("seal", 1, object_seal),
            ("isFrozen", 1, object_is_frozen),
            ("isSealed", 1, object_is_sealed),
        ];

        for (name, length, body) in methods {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                object_ctor,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
        }
    }

    fn install_object_prototype(&mut self, object_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("hasOwnProperty", 1, object_proto_has_own_property),
            (
                "propertyIsEnumerable",
                1,
                object_proto_property_is_enumerable,
            ),
            ("valueOf", 0, object_proto_value_of),
            ("toString", 0, object_proto_to_string),
            ("toLocaleString", 0, object_proto_to_locale_string),
            ("isPrototypeOf", 1, object_proto_is_prototype_of),
            ("__defineGetter__", 2, object_proto_define_getter),
            ("__defineSetter__", 2, object_proto_define_setter),
            ("__lookupGetter__", 1, object_proto_lookup_getter),
            ("__lookupSetter__", 1, object_proto_lookup_setter),
        ];

        for (name, length, body) in methods {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                object_proto,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
        }
        let get_proto = self.create_builtin_function(
            "get __proto__",
            0,
            Some(function_proto),
            object_proto_get_proto,
        );
        let set_proto = self.create_builtin_function(
            "set __proto__",
            1,
            Some(function_proto),
            object_proto_set_proto,
        );
        define_accessor_key_with_attrs(
            &mut self.heap,
            object_proto,
            PropertyKey::from("__proto__"),
            Some(Value::Object(get_proto)),
            Some(Value::Object(set_proto)),
            false,
            true,
        );
    }

    fn install_function_prototype(&mut self, function_proto: ObjectRef) {
        define_data_with_attrs(
            &mut self.heap,
            function_proto,
            "length",
            Value::Number(0.0),
            false,
            false,
            true,
        );
        define_data_with_attrs(
            &mut self.heap,
            function_proto,
            "name",
            Value::String(String::new()),
            false,
            false,
            true,
        );

        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("call", 1, function_proto_call),
            ("apply", 2, function_proto_apply),
            ("bind", 1, function_proto_bind),
        ];

        for (name, length, body) in methods {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                function_proto,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
        }
    }

    fn install_iterator_prototype(&mut self, iterator_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("drop", 1, iterator_proto_drop),
            ("every", 1, iterator_proto_every),
            ("filter", 1, iterator_proto_filter),
            ("find", 1, iterator_proto_find),
            ("flatMap", 1, iterator_proto_flat_map),
            ("forEach", 1, iterator_proto_for_each),
            ("map", 1, iterator_proto_map),
            ("reduce", 1, iterator_proto_reduce),
            ("some", 1, iterator_proto_some),
            ("take", 1, iterator_proto_take),
            ("toArray", 0, iterator_proto_to_array),
        ];
        self.install_methods(iterator_proto, function_proto, methods);
        let iterator_method = self.create_builtin_function(
            "[Symbol.iterator]",
            0,
            Some(function_proto),
            iterator_self,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            iterator_proto,
            PropertyKey::Symbol(SYMBOL_ITERATOR_ID),
            Value::Object(iterator_method),
            true,
            false,
            true,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            iterator_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Iterator".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_iterator_statics(&mut self, iterator_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] =
            &[("concat", 0, iterator_concat), ("from", 1, iterator_from)];
        self.install_methods(iterator_ctor, function_proto, methods);
    }

    fn install_array_iterator_prototype(
        &mut self,
        array_iterator_proto: ObjectRef,
        function_proto: ObjectRef,
    ) {
        let next =
            self.create_builtin_function("next", 0, Some(function_proto), indexed_iterator_next);
        define_data_with_attrs(
            &mut self.heap,
            array_iterator_proto,
            "next",
            Value::Object(next),
            true,
            false,
            true,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            array_iterator_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Array Iterator".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_string_iterator_prototype(
        &mut self,
        string_iterator_proto: ObjectRef,
        function_proto: ObjectRef,
    ) {
        let next =
            self.create_builtin_function("next", 0, Some(function_proto), indexed_iterator_next);
        define_data_with_attrs(
            &mut self.heap,
            string_iterator_proto,
            "next",
            Value::Object(next),
            true,
            false,
            true,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            string_iterator_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("String Iterator".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_map_prototype(&mut self, map_proto: ObjectRef, function_proto: ObjectRef) {
        let size_getter =
            self.create_builtin_function("get size", 0, Some(function_proto), map_proto_size);
        define_accessor_key_with_attrs(
            &mut self.heap,
            map_proto,
            PropertyKey::from("size"),
            Some(Value::Object(size_getter)),
            None,
            false,
            true,
        );
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("get", 1, map_proto_get),
            ("set", 2, map_proto_set),
            ("has", 1, map_proto_has),
            ("delete", 1, map_proto_delete),
            ("clear", 0, map_proto_clear),
            ("keys", 0, map_proto_keys),
            ("values", 0, map_proto_values),
            ("entries", 0, map_proto_entries),
            ("forEach", 1, map_proto_for_each),
        ];
        self.install_methods(map_proto, function_proto, methods);
        if let Some(entries) = self
            .heap
            .get(map_proto)
            .ok()
            .and_then(|object| {
                object
                    .properties
                    .get(&PropertyKey::from("entries"))
                    .cloned()
            })
            .and_then(|desc| desc.value)
        {
            define_data_key_with_attrs(
                &mut self.heap,
                map_proto,
                PropertyKey::Symbol(SYMBOL_ITERATOR_ID),
                entries,
                true,
                false,
                true,
            );
        }
        define_data_key_with_attrs(
            &mut self.heap,
            map_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Map".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_set_prototype(&mut self, set_proto: ObjectRef, function_proto: ObjectRef) {
        let size_getter =
            self.create_builtin_function("get size", 0, Some(function_proto), set_proto_size);
        define_accessor_key_with_attrs(
            &mut self.heap,
            set_proto,
            PropertyKey::from("size"),
            Some(Value::Object(size_getter)),
            None,
            false,
            true,
        );
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("add", 1, set_proto_add),
            ("has", 1, set_proto_has),
            ("delete", 1, set_proto_delete),
            ("clear", 0, set_proto_clear),
            ("keys", 0, set_proto_values),
            ("values", 0, set_proto_values),
            ("entries", 0, set_proto_entries),
            ("forEach", 1, set_proto_for_each),
        ];
        self.install_methods(set_proto, function_proto, methods);
        if let Some(values) = self
            .heap
            .get(set_proto)
            .ok()
            .and_then(|object| object.properties.get(&PropertyKey::from("values")).cloned())
            .and_then(|desc| desc.value)
        {
            define_data_key_with_attrs(
                &mut self.heap,
                set_proto,
                PropertyKey::Symbol(SYMBOL_ITERATOR_ID),
                values,
                true,
                false,
                true,
            );
        }
        define_data_key_with_attrs(
            &mut self.heap,
            set_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Set".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_weak_map_prototype(&mut self, weak_map_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("get", 1, weak_map_proto_get),
            ("set", 2, weak_map_proto_set),
            ("has", 1, weak_map_proto_has),
            ("delete", 1, weak_map_proto_delete),
        ];
        self.install_methods(weak_map_proto, function_proto, methods);
        define_data_key_with_attrs(
            &mut self.heap,
            weak_map_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("WeakMap".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_weak_set_prototype(&mut self, weak_set_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("add", 1, weak_set_proto_add),
            ("has", 1, weak_set_proto_has),
            ("delete", 1, weak_set_proto_delete),
        ];
        self.install_methods(weak_set_proto, function_proto, methods);
        define_data_key_with_attrs(
            &mut self.heap,
            weak_set_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("WeakSet".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_map_iterator_prototype(
        &mut self,
        map_iterator_proto: ObjectRef,
        function_proto: ObjectRef,
    ) {
        let next =
            self.create_builtin_function("next", 0, Some(function_proto), collection_iterator_next);
        define_data_with_attrs(
            &mut self.heap,
            map_iterator_proto,
            "next",
            Value::Object(next),
            true,
            false,
            true,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            map_iterator_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Map Iterator".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_set_iterator_prototype(
        &mut self,
        set_iterator_proto: ObjectRef,
        function_proto: ObjectRef,
    ) {
        let next =
            self.create_builtin_function("next", 0, Some(function_proto), collection_iterator_next);
        define_data_with_attrs(
            &mut self.heap,
            set_iterator_proto,
            "next",
            Value::Object(next),
            true,
            false,
            true,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            set_iterator_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Set Iterator".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_array_statics(&mut self, array_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("from", 1, array_from),
            ("isArray", 1, array_is_array),
            ("of", 0, array_of),
        ];

        for (name, length, body) in methods {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                array_ctor,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
        }
    }

    fn install_array_prototype(&mut self, array_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("toString", 0, array_proto_to_string),
            ("toLocaleString", 0, array_proto_to_locale_string),
            ("join", 1, array_proto_join),
            ("at", 1, array_proto_at),
            ("push", 1, array_proto_push),
            ("pop", 0, array_proto_pop),
            ("shift", 0, array_proto_shift),
            ("unshift", 1, array_proto_unshift),
            ("reverse", 0, array_proto_reverse),
            ("sort", 1, array_proto_sort),
            ("slice", 2, array_proto_slice),
            ("splice", 2, array_proto_splice),
            ("toReversed", 0, array_proto_to_reversed),
            ("toSorted", 1, array_proto_to_sorted),
            ("toSpliced", 2, array_proto_to_spliced),
            ("with", 2, array_proto_with),
            ("flat", 0, array_proto_flat),
            ("flatMap", 1, array_proto_flat_map),
            ("copyWithin", 2, array_proto_copy_within),
            ("fill", 1, array_proto_fill),
            ("values", 0, array_proto_values),
            ("keys", 0, array_proto_keys),
            ("entries", 0, array_proto_entries),
            ("map", 1, array_proto_map),
            ("filter", 1, array_proto_filter),
            ("forEach", 1, array_proto_for_each),
            ("find", 1, array_proto_find),
            ("findIndex", 1, array_proto_find_index),
            ("findLast", 1, array_proto_find_last),
            ("findLastIndex", 1, array_proto_find_last_index),
            ("every", 1, array_proto_every),
            ("some", 1, array_proto_some),
            ("includes", 1, array_proto_includes),
            ("indexOf", 1, array_proto_index_of),
            ("lastIndexOf", 1, array_proto_last_index_of),
            ("reduce", 1, array_proto_reduce),
            ("reduceRight", 1, array_proto_reduce_right),
        ];

        let mut values_function = None;
        for (name, length, body) in methods {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                array_proto,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
            if *name == "values" {
                values_function = Some(Value::Object(function));
            }
        }
        if let Some(values_function) = values_function {
            define_data_key_with_attrs(
                &mut self.heap,
                array_proto,
                PropertyKey::Symbol(SYMBOL_ITERATOR_ID),
                values_function,
                true,
                false,
                true,
            );
        }
        let unscopables = self.heap.allocate(JsObject::ordinary(None));
        for name in [
            "at",
            "copyWithin",
            "entries",
            "fill",
            "find",
            "findIndex",
            "findLast",
            "findLastIndex",
            "flat",
            "flatMap",
            "includes",
            "keys",
            "toReversed",
            "toSorted",
            "toSpliced",
            "values",
        ] {
            define_data_with_attrs(
                &mut self.heap,
                unscopables,
                name,
                Value::Boolean(true),
                true,
                true,
                true,
            );
        }
        define_data_key_with_attrs(
            &mut self.heap,
            array_proto,
            PropertyKey::Symbol(SYMBOL_UNSCOPABLES_ID),
            Value::Object(unscopables),
            false,
            false,
            true,
        );
    }

    fn install_boolean_prototype(&mut self, boolean_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("valueOf", 0, primitive_proto_value_of),
            ("toString", 0, boolean_proto_to_string),
        ];
        self.install_methods(boolean_proto, function_proto, methods);
    }

    fn install_number_prototype(&mut self, number_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("valueOf", 0, primitive_proto_value_of),
            ("toString", 0, number_proto_to_string),
            ("toFixed", 1, number_proto_to_fixed),
            ("toExponential", 1, number_proto_to_exponential),
            ("toPrecision", 1, number_proto_to_precision),
        ];
        self.install_methods(number_proto, function_proto, methods);
    }

    fn install_number_statics(&mut self, number_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("isFinite", 1, number_is_finite),
            ("isInteger", 1, number_is_integer),
            ("isNaN", 1, number_is_nan),
            ("isSafeInteger", 1, number_is_safe_integer),
            ("parseFloat", 1, global_parse_float),
            ("parseInt", 2, global_parse_int),
        ];
        self.install_methods(number_ctor, function_proto, methods);
    }

    fn install_bigint_statics(&mut self, bigint_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("asIntN", 2, bigint_as_int_n),
            ("asUintN", 2, bigint_as_uint_n),
        ];
        self.install_methods(bigint_ctor, function_proto, methods);
    }

    fn install_bigint_prototype(
        &mut self,
        bigint_proto: ObjectRef,
        bigint_ctor: ObjectRef,
        function_proto: ObjectRef,
    ) {
        define_data_with_attrs(
            &mut self.heap,
            bigint_proto,
            "constructor",
            Value::Object(bigint_ctor),
            true,
            false,
            true,
        );
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("toString", 0, bigint_proto_to_string),
            ("valueOf", 0, bigint_proto_value_of),
        ];
        self.install_methods(bigint_proto, function_proto, methods);
        define_data_key_with_attrs(
            &mut self.heap,
            bigint_proto,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("BigInt".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_string_prototype(&mut self, string_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("valueOf", 0, primitive_proto_value_of),
            ("toString", 0, string_proto_to_string),
            ("at", 1, string_proto_at),
            ("charAt", 1, string_proto_char_at),
            ("charCodeAt", 1, string_proto_char_code_at),
            ("codePointAt", 1, string_proto_code_point_at),
            ("concat", 1, string_proto_concat),
            ("endsWith", 1, string_proto_ends_with),
            ("includes", 1, string_proto_includes),
            ("indexOf", 1, string_proto_index_of),
            ("lastIndexOf", 1, string_proto_last_index_of),
            ("localeCompare", 1, string_proto_locale_compare),
            ("normalize", 0, string_proto_normalize),
            ("padEnd", 1, string_proto_pad_end),
            ("padStart", 1, string_proto_pad_start),
            ("repeat", 1, string_proto_repeat),
            ("slice", 2, string_proto_slice),
            ("split", 2, string_proto_split),
            ("startsWith", 1, string_proto_starts_with),
            ("substring", 2, string_proto_substring),
            ("toLowerCase", 0, string_proto_to_lower_case),
            ("toLocaleLowerCase", 0, string_proto_to_lower_case),
            ("toUpperCase", 0, string_proto_to_upper_case),
            ("toLocaleUpperCase", 0, string_proto_to_upper_case),
            ("isWellFormed", 0, string_proto_is_well_formed),
            ("trim", 0, string_proto_trim),
            ("trimEnd", 0, string_proto_trim_end),
            ("trimStart", 0, string_proto_trim_start),
            ("toWellFormed", 0, string_proto_to_well_formed),
        ];
        self.install_methods(string_proto, function_proto, methods);
        let iterator = self.create_builtin_function(
            "[Symbol.iterator]",
            0,
            Some(function_proto),
            string_iterator,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            string_proto,
            PropertyKey::Symbol(SYMBOL_ITERATOR_ID),
            Value::Object(iterator),
            true,
            false,
            true,
        );
    }

    fn install_string_statics(&mut self, string_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("fromCharCode", 1, string_from_char_code),
            ("fromCodePoint", 1, string_from_code_point),
            ("raw", 1, string_raw),
        ];
        self.install_methods(string_ctor, function_proto, methods);
    }

    fn install_symbol_statics(&mut self, symbol_ctor: ObjectRef) {
        for (name, id) in [
            ("iterator", SYMBOL_ITERATOR_ID),
            ("replace", SYMBOL_REPLACE_ID),
            ("species", SYMBOL_SPECIES_ID),
            ("isConcatSpreadable", SYMBOL_IS_CONCAT_SPREADABLE_ID),
            ("toPrimitive", SYMBOL_TO_PRIMITIVE_ID),
            ("toStringTag", SYMBOL_TO_STRING_TAG_ID),
            ("unscopables", SYMBOL_UNSCOPABLES_ID),
        ] {
            define_data_with_attrs(
                &mut self.heap,
                symbol_ctor,
                name,
                Value::Symbol(id),
                false,
                false,
                false,
            );
        }
    }

    fn install_symbol_prototype(&mut self, symbol_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("valueOf", 0, primitive_proto_value_of),
            ("toString", 0, symbol_proto_to_string),
        ];
        self.install_methods(symbol_proto, function_proto, methods);
    }

    fn install_regexp_prototype(&mut self, regexp_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("exec", 1, regexp_proto_exec),
            ("test", 1, regexp_proto_test),
            ("toString", 0, regexp_proto_to_string),
        ];
        self.install_methods(regexp_proto, function_proto, methods);
        let symbol_replace = self.create_builtin_function(
            "[Symbol.replace]",
            2,
            Some(function_proto),
            regexp_proto_symbol_replace,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            regexp_proto,
            PropertyKey::Symbol(SYMBOL_REPLACE_ID),
            Value::Object(symbol_replace),
            true,
            false,
            true,
        );
        for (name, getter_body) in [
            ("global", regexp_proto_get_global as super::BuiltinFn),
            ("ignoreCase", regexp_proto_get_ignore_case),
            ("multiline", regexp_proto_get_multiline),
            ("dotAll", regexp_proto_get_dot_all),
            ("unicode", regexp_proto_get_unicode),
            ("sticky", regexp_proto_get_sticky),
            ("hasIndices", regexp_proto_get_has_indices),
        ] {
            let getter_name = format!("get {name}");
            let getter =
                self.create_builtin_function(&getter_name, 0, Some(function_proto), getter_body);
            define_accessor_key_with_attrs(
                &mut self.heap,
                regexp_proto,
                PropertyKey::from(name),
                Some(Value::Object(getter)),
                None,
                false,
                true,
            );
        }
    }

    fn install_date_prototype(&mut self, date_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("toString", 0, date_proto_to_string),
            ("toDateString", 0, date_proto_to_date_string),
            ("toTimeString", 0, date_proto_to_time_string),
            ("toUTCString", 0, date_proto_to_utc_string),
            ("toISOString", 0, date_proto_to_iso_string),
            ("toLocaleString", 0, date_proto_to_string),
            ("toLocaleDateString", 0, date_proto_to_string),
            ("toLocaleTimeString", 0, date_proto_to_string),
            ("toJSON", 1, date_proto_to_json),
            ("valueOf", 0, date_proto_value_of),
            ("getTime", 0, date_proto_value_of),
            ("getTimezoneOffset", 0, date_proto_get_timezone_offset),
            ("getFullYear", 0, date_proto_get_utc_full_year),
            ("getUTCFullYear", 0, date_proto_get_utc_full_year),
            ("getMonth", 0, date_proto_get_utc_month),
            ("getUTCMonth", 0, date_proto_get_utc_month),
            ("getDate", 0, date_proto_get_utc_date),
            ("getUTCDate", 0, date_proto_get_utc_date),
            ("getDay", 0, date_proto_get_utc_day),
            ("getUTCDay", 0, date_proto_get_utc_day),
            ("getHours", 0, date_proto_get_utc_hours),
            ("getUTCHours", 0, date_proto_get_utc_hours),
            ("getMinutes", 0, date_proto_get_utc_minutes),
            ("getUTCMinutes", 0, date_proto_get_utc_minutes),
            ("getSeconds", 0, date_proto_get_utc_seconds),
            ("getUTCSeconds", 0, date_proto_get_utc_seconds),
            ("getMilliseconds", 0, date_proto_get_utc_milliseconds),
            ("getUTCMilliseconds", 0, date_proto_get_utc_milliseconds),
            ("setTime", 1, date_proto_set_time),
            ("setMilliseconds", 1, date_proto_set_utc_milliseconds),
            ("setUTCMilliseconds", 1, date_proto_set_utc_milliseconds),
            ("setSeconds", 2, date_proto_set_utc_seconds),
            ("setUTCSeconds", 2, date_proto_set_utc_seconds),
            ("setMinutes", 3, date_proto_set_utc_minutes),
            ("setUTCMinutes", 3, date_proto_set_utc_minutes),
            ("setHours", 4, date_proto_set_utc_hours),
            ("setUTCHours", 4, date_proto_set_utc_hours),
            ("setDate", 1, date_proto_set_utc_date),
            ("setUTCDate", 1, date_proto_set_utc_date),
            ("setMonth", 2, date_proto_set_utc_month),
            ("setUTCMonth", 2, date_proto_set_utc_month),
            ("setFullYear", 3, date_proto_set_utc_full_year),
            ("setUTCFullYear", 3, date_proto_set_utc_full_year),
        ];
        self.install_methods(date_proto, function_proto, methods);
        let to_primitive = self.create_builtin_function(
            "[Symbol.toPrimitive]",
            1,
            Some(function_proto),
            date_proto_symbol_to_primitive,
        );
        define_data_key_with_attrs(
            &mut self.heap,
            date_proto,
            PropertyKey::Symbol(SYMBOL_TO_PRIMITIVE_ID),
            Value::Object(to_primitive),
            false,
            false,
            true,
        );
    }

    fn install_date_statics(&mut self, date_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("now", 0, date_now),
            ("parse", 1, date_parse),
            ("UTC", 7, date_utc),
        ];
        self.install_methods(date_ctor, function_proto, methods);
    }

    fn install_math_object(&mut self, math_obj: ObjectRef, function_proto: ObjectRef) {
        for (name, value) in [
            ("E", std::f64::consts::E),
            ("LN10", std::f64::consts::LN_10),
            ("LN2", std::f64::consts::LN_2),
            ("LOG10E", std::f64::consts::LOG10_E),
            ("LOG2E", std::f64::consts::LOG2_E),
            ("PI", std::f64::consts::PI),
            ("SQRT1_2", std::f64::consts::FRAC_1_SQRT_2),
            ("SQRT2", std::f64::consts::SQRT_2),
        ] {
            define_data_with_attrs(
                &mut self.heap,
                math_obj,
                name,
                Value::Number(value),
                false,
                false,
                false,
            );
        }
        define_data_key_with_attrs(
            &mut self.heap,
            math_obj,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Math".to_owned()),
            false,
            false,
            true,
        );
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("abs", 1, math_abs),
            ("acos", 1, math_acos),
            ("acosh", 1, math_acosh),
            ("asin", 1, math_asin),
            ("asinh", 1, math_asinh),
            ("atan", 1, math_atan),
            ("atan2", 2, math_atan2),
            ("atanh", 1, math_atanh),
            ("cbrt", 1, math_cbrt),
            ("ceil", 1, math_ceil),
            ("clz32", 1, math_clz32),
            ("cos", 1, math_cos),
            ("cosh", 1, math_cosh),
            ("exp", 1, math_exp),
            ("expm1", 1, math_expm1),
            ("floor", 1, math_floor),
            ("fround", 1, math_fround),
            ("f16round", 1, math_f16round),
            ("hypot", 2, math_hypot),
            ("imul", 2, math_imul),
            ("log", 1, math_log),
            ("log10", 1, math_log10),
            ("log1p", 1, math_log1p),
            ("log2", 1, math_log2),
            ("max", 2, math_max),
            ("min", 2, math_min),
            ("pow", 2, math_pow),
            ("random", 0, math_random),
            ("round", 1, math_round),
            ("sign", 1, math_sign),
            ("sin", 1, math_sin),
            ("sinh", 1, math_sinh),
            ("sqrt", 1, math_sqrt),
            ("tan", 1, math_tan),
            ("tanh", 1, math_tanh),
            ("trunc", 1, math_trunc),
            ("sumPrecise", 1, math_sum_precise),
        ];
        self.install_methods(math_obj, function_proto, methods);
    }

    fn install_json_object(&mut self, json_obj: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] =
            &[("parse", 2, json_parse), ("stringify", 3, json_stringify)];
        self.install_methods(json_obj, function_proto, methods);
        define_data_key_with_attrs(
            &mut self.heap,
            json_obj,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("JSON".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_reflect_object(&mut self, reflect_obj: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("apply", 3, reflect_apply),
            ("construct", 2, reflect_construct),
            ("defineProperty", 3, reflect_define_property),
            ("deleteProperty", 2, reflect_delete_property),
            ("get", 2, reflect_get),
            (
                "getOwnPropertyDescriptor",
                2,
                reflect_get_own_property_descriptor,
            ),
            ("getPrototypeOf", 1, reflect_get_prototype_of),
            ("has", 2, reflect_has),
            ("isExtensible", 1, reflect_is_extensible),
            ("ownKeys", 1, reflect_own_keys),
            ("preventExtensions", 1, reflect_prevent_extensions),
            ("set", 3, reflect_set),
            ("setPrototypeOf", 2, reflect_set_prototype_of),
        ];
        self.install_methods(reflect_obj, function_proto, methods);
        define_data_key_with_attrs(
            &mut self.heap,
            reflect_obj,
            PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::String("Reflect".to_owned()),
            false,
            false,
            true,
        );
    }

    fn install_proxy_constructor(&mut self, proxy_ctor: ObjectRef, function_proto: ObjectRef) {
        let revocable =
            self.create_builtin_function("revocable", 2, Some(function_proto), proxy_revocable);
        define_data_with_attrs(
            &mut self.heap,
            proxy_ctor,
            "revocable",
            Value::Object(revocable),
            true,
            false,
            true,
        );
    }

    fn install_global_functions(&mut self, global: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("eval", 1, global_eval),
            ("parseInt", 2, global_parse_int),
            ("parseFloat", 1, global_parse_float),
            ("isNaN", 1, global_is_nan),
            ("isFinite", 1, global_is_finite),
            ("decodeURI", 1, global_decode_uri),
            ("decodeURIComponent", 1, global_decode_uri_component),
            ("encodeURI", 1, global_encode_uri),
            ("encodeURIComponent", 1, global_encode_uri_component),
        ];
        self.install_methods(global, function_proto, methods);
    }

    fn install_number_parse_aliases(&mut self, number_ctor: ObjectRef, global: ObjectRef) {
        for name in ["parseInt", "parseFloat"] {
            let Ok(value) = global.get(
                &mut Context::new(self, self.default_realm),
                &PropertyKey::from(name),
                Value::Object(global),
            ) else {
                continue;
            };
            define_data_with_attrs(&mut self.heap, number_ctor, name, value, true, false, true);
        }
    }

    fn install_methods(
        &mut self,
        target: ObjectRef,
        function_proto: ObjectRef,
        methods: &[(&str, u32, super::BuiltinFn)],
    ) {
        for (name, length, body) in methods {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                target,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
        }
    }

    fn install_error_prototype(&mut self, error_proto: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[("toString", 0, error_proto_to_string)];
        self.install_methods(error_proto, function_proto, methods);
    }

    fn install_error_statics(&mut self, error_ctor: ObjectRef, function_proto: ObjectRef) {
        let methods: &[(&str, u32, super::BuiltinFn)] = &[("isError", 1, error_is_error)];
        self.install_methods(error_ctor, function_proto, methods);
    }

    fn install_test262_helpers(
        &mut self,
        global: ObjectRef,
        _object_proto: ObjectRef,
        function_proto: ObjectRef,
    ) {
        let assert = self.heap.allocate(JsObject::function(
            Some(function_proto),
            FunctionData::builtin("assert", 1, test262_assert),
        ));
        let methods: &[(&str, u32, super::BuiltinFn)] = &[
            ("sameValue", 2, test262_assert_same_value),
            ("notSameValue", 2, test262_assert_not_same_value),
            ("compareArray", 2, test262_assert_compare_array),
            ("throws", 2, test262_assert_throws),
        ];
        for (name, length, body) in methods {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                assert,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
        }
        define_data_with_attrs(
            &mut self.heap,
            global,
            "assert",
            Value::Object(assert),
            true,
            false,
            true,
        );

        let verify = self.create_builtin_function(
            "verifyProperty",
            3,
            Some(function_proto),
            test262_verify_property,
        );
        define_data_with_attrs(
            &mut self.heap,
            global,
            "verifyProperty",
            Value::Object(verify),
            true,
            false,
            true,
        );

        let globals: &[(&str, u32, super::BuiltinFn)] = &[
            ("verifyEqualTo", 3, test262_verify_equal_to),
            ("verifyWritable", 2, test262_verify_writable),
            ("verifyNotWritable", 2, test262_verify_not_writable),
            ("verifyEnumerable", 2, test262_verify_enumerable),
            ("verifyNotEnumerable", 2, test262_verify_not_enumerable),
            ("verifyConfigurable", 2, test262_verify_configurable),
            ("verifyNotConfigurable", 2, test262_verify_not_configurable),
            ("verifyPrimordialProperty", 3, test262_verify_property),
            ("isConstructor", 1, test262_is_constructor),
            ("checkSequence", 1, test262_check_sequence),
            (
                "verifyPrimordialCallableProperty",
                3,
                test262_verify_primordial_callable_property,
            ),
        ];
        for (name, length, body) in globals {
            let function = self.create_builtin_function(name, *length, Some(function_proto), *body);
            define_data_with_attrs(
                &mut self.heap,
                global,
                *name,
                Value::Object(function),
                true,
                false,
                true,
            );
        }
    }
}

fn function_constructor(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let (params, body_source) = if args.is_empty() {
        (Vec::new(), String::new())
    } else {
        let mut params = Vec::new();
        for value in &args[..args.len() - 1] {
            params.push(to_string_value(cx, value.clone())?);
        }
        (
            params,
            to_string_value(cx, args.last().cloned().unwrap_or(Value::Undefined))?,
        )
    };
    let params_source = params.join(",");
    let source = format!("function anonymous({params_source}) {{ {body_source} }}");
    let mut statements = parse_script(&source)?;
    let Some(Stmt::Function(_, parsed_params, parsed_body)) = statements.pop() else {
        return Err(JsError::syntax(
            "Function constructor did not produce a function",
        ));
    };
    let length = parsed_params.len() as u32;
    let proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::FunctionPrototype)
        .ok_or_else(|| JsError::internal("missing Function.prototype intrinsic"))?;
    let function = cx.heap_mut().allocate(JsObject::function(
        Some(proto),
        FunctionData::script("anonymous", parsed_params, parsed_body),
    ));
    function.define_own_property_or_throw(
        cx,
        PropertyKey::from("length"),
        Descriptor::data(Value::Number(length as f64), false, false, true),
    )?;
    function.define_own_property_or_throw(
        cx,
        PropertyKey::from("name"),
        Descriptor::data(Value::String("anonymous".to_owned()), false, false, true),
    )?;
    let object_proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let prototype_object = cx
        .heap_mut()
        .allocate(JsObject::ordinary(Some(object_proto)));
    prototype_object.define_own_property_or_throw(
        cx,
        PropertyKey::from("constructor"),
        Descriptor::data(Value::Object(function), true, false, true),
    )?;
    function.define_own_property_or_throw(
        cx,
        PropertyKey::from("prototype"),
        Descriptor::data(Value::Object(prototype_object), true, false, false),
    )?;
    Ok(Value::Object(function))
}

fn iterator_constructor(_cx: &mut Context, _this: Value, _args: &[Value]) -> Completion<Value> {
    Err(JsError::type_error("Iterator cannot be called directly"))
}

fn map_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if !is_construct_call(&this) {
        return Err(JsError::type_error("Map constructor requires 'new'"));
    }
    let map = create_collection_object(cx, IntrinsicId::MapPrototype, CollectionKind::Map)?;
    let iterable = args.first().cloned().unwrap_or(Value::Undefined);
    if matches!(iterable, Value::Undefined | Value::Null) {
        return Ok(Value::Object(map));
    }
    let adder = map.get(cx, &PropertyKey::from("set"), Value::Object(map))?;
    if !cx.is_callable(&adder)? {
        return Err(JsError::type_error("Map adder is not callable"));
    }
    let record = get_iterator(cx, iterable)?;
    loop {
        let entry = match iterator_step_value(cx, &record) {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        let result = call_map_constructor_adder(cx, map, adder.clone(), entry);
        if let Err(error) = result {
            return Err(iterator_close_error(cx, &record, error));
        }
    }
    Ok(Value::Object(map))
}

fn set_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if !is_construct_call(&this) {
        return Err(JsError::type_error("Set constructor requires 'new'"));
    }
    let set = create_collection_object(cx, IntrinsicId::SetPrototype, CollectionKind::Set)?;
    let iterable = args.first().cloned().unwrap_or(Value::Undefined);
    if matches!(iterable, Value::Undefined | Value::Null) {
        return Ok(Value::Object(set));
    }
    let adder = set.get(cx, &PropertyKey::from("add"), Value::Object(set))?;
    if !cx.is_callable(&adder)? {
        return Err(JsError::type_error("Set adder is not callable"));
    }
    let record = get_iterator(cx, iterable)?;
    loop {
        let value = match iterator_step_value(cx, &record) {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        if let Err(error) = cx.call_mut(adder.clone(), Value::Object(set), &[value]) {
            return Err(iterator_close_error(cx, &record, error));
        }
    }
    Ok(Value::Object(set))
}

fn weak_map_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if !is_construct_call(&this) {
        return Err(JsError::type_error("WeakMap constructor requires 'new'"));
    }
    let weak_map = create_weak_collection_object(cx, IntrinsicId::WeakMapPrototype, true)?;
    let iterable = args.first().cloned().unwrap_or(Value::Undefined);
    if matches!(iterable, Value::Undefined | Value::Null) {
        return Ok(Value::Object(weak_map));
    }
    let adder = weak_map.get(cx, &PropertyKey::from("set"), Value::Object(weak_map))?;
    if !cx.is_callable(&adder)? {
        return Err(JsError::type_error("WeakMap adder is not callable"));
    }
    let record = get_iterator(cx, iterable)?;
    loop {
        let entry = match iterator_step_value(cx, &record) {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        let result = call_map_constructor_adder(cx, weak_map, adder.clone(), entry);
        if let Err(error) = result {
            return Err(iterator_close_error(cx, &record, error));
        }
    }
    Ok(Value::Object(weak_map))
}

fn weak_set_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if !is_construct_call(&this) {
        return Err(JsError::type_error("WeakSet constructor requires 'new'"));
    }
    let weak_set = create_weak_collection_object(cx, IntrinsicId::WeakSetPrototype, false)?;
    let iterable = args.first().cloned().unwrap_or(Value::Undefined);
    if matches!(iterable, Value::Undefined | Value::Null) {
        return Ok(Value::Object(weak_set));
    }
    let adder = weak_set.get(cx, &PropertyKey::from("add"), Value::Object(weak_set))?;
    if !cx.is_callable(&adder)? {
        return Err(JsError::type_error("WeakSet adder is not callable"));
    }
    let record = get_iterator(cx, iterable)?;
    loop {
        let value = match iterator_step_value(cx, &record) {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        if let Err(error) = cx.call_mut(adder.clone(), Value::Object(weak_set), &[value]) {
            return Err(iterator_close_error(cx, &record, error));
        }
    }
    Ok(Value::Object(weak_set))
}

fn proxy_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if !is_construct_call(&this) {
        return Err(JsError::type_error("Proxy constructor requires 'new'"));
    }
    let target = require_actual_object_arg(args.first())?;
    let handler = require_actual_object_arg(args.get(1))?;
    Ok(Value::Object(create_proxy_object(cx, target, handler)?))
}

fn proxy_revocable(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let handler = require_actual_object_arg(args.get(1))?;
    let proxy = create_proxy_object(cx, target, handler)?;

    let function_proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::FunctionPrototype)
        .ok_or_else(|| JsError::internal("missing Function.prototype intrinsic"))?;
    let base = cx.heap_mut().allocate(JsObject::function(
        Some(function_proto),
        FunctionData::builtin("Proxy revoker", 0, proxy_revoke),
    ));
    let revoke = cx.heap_mut().allocate(JsObject::function(
        Some(function_proto),
        FunctionData::bound(
            "revoke",
            0,
            Value::Object(base),
            Value::Undefined,
            vec![Value::Object(proxy)],
        ),
    ));
    define_data_with_attrs(
        cx.heap_mut(),
        revoke,
        "length",
        Value::Number(0.0),
        false,
        false,
        true,
    );
    define_data_with_attrs(
        cx.heap_mut(),
        revoke,
        "name",
        Value::String(String::new()),
        false,
        false,
        true,
    );

    let object_proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let result = cx
        .heap_mut()
        .allocate(JsObject::ordinary(Some(object_proto)));
    result.define_own_property_or_throw(
        cx,
        PropertyKey::from("proxy"),
        Descriptor::data(Value::Object(proxy), true, true, true),
    )?;
    result.define_own_property_or_throw(
        cx,
        PropertyKey::from("revoke"),
        Descriptor::data(Value::Object(revoke), true, true, true),
    )?;
    Ok(Value::Object(result))
}

fn proxy_revoke(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let Some(Value::Object(proxy)) = args.first() else {
        return Ok(Value::Undefined);
    };
    let object = cx.heap_mut().get_mut(*proxy)?;
    for slot in &mut object.internal_slots {
        if let InternalSlot::ProxyData {
            target, handler, ..
        } = slot
        {
            *target = None;
            *handler = None;
            break;
        }
    }
    Ok(Value::Undefined)
}

fn create_proxy_object(
    cx: &mut Context,
    target: ObjectRef,
    handler: ObjectRef,
) -> Completion<ObjectRef> {
    let (callable, constructible) = proxy_target_capabilities(cx, target)?;
    let object_proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let proxy = cx
        .heap_mut()
        .allocate(JsObject::ordinary(Some(object_proto)));
    cx.heap_mut()
        .get_mut(proxy)?
        .add_slot(InternalSlot::ProxyData {
            target: Some(target),
            handler: Some(handler),
            callable,
            constructible,
        });
    Ok(proxy)
}

fn is_construct_call(value: &Value) -> bool {
    matches!(
        value,
        Value::InternalConstruct | Value::InternalConstructWithNewTarget(_)
    )
}

fn construct_new_target(value: &Value) -> Option<ObjectRef> {
    match value {
        Value::InternalConstructWithNewTarget(object) => Some(*object),
        _ => None,
    }
}

fn proxy_target_capabilities(cx: &Context, target: ObjectRef) -> Completion<(bool, bool)> {
    let object = cx.heap().get(target)?;
    for slot in &object.internal_slots {
        if let InternalSlot::ProxyData {
            callable,
            constructible,
            ..
        } = slot
        {
            return Ok((*callable, *constructible));
        }
    }
    match &object.kind {
        ObjectKind::Function(data) => Ok((data.callable, data.constructible)),
        _ => Ok((false, false)),
    }
}

fn reflect_apply(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&target)? {
        return Err(JsError::type_error("Reflect.apply target is not callable"));
    }
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    let arguments =
        create_list_from_array_like(cx, args.get(2).cloned().unwrap_or(Value::Undefined))?;
    cx.call_mut(target, this_arg, &arguments)
}

fn reflect_construct(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_constructor(&target)? {
        return Err(JsError::type_error(
            "Reflect.construct target is not a constructor",
        ));
    }
    let arguments =
        create_list_from_array_like(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    let new_target = if let Some(new_target) = args.get(2) {
        if !cx.is_constructor(new_target)? {
            return Err(JsError::type_error(
                "Reflect.construct newTarget is not a constructor",
            ));
        }
        new_target.clone()
    } else {
        target.clone()
    };
    cx.construct_mut_with_new_target(target, &arguments, new_target)
}

fn reflect_define_property(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let desc = ToPropertyDescriptor(cx, args.get(2).cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Boolean(target.define_own_property(cx, key, desc)?))
}

fn reflect_delete_property(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    Ok(Value::Boolean(target.delete(cx, &key)?))
}

fn reflect_get(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let receiver = args.get(2).cloned().unwrap_or(Value::Object(target));
    target.get(cx, &key, receiver)
}

fn reflect_get_own_property_descriptor(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let desc = target.get_own_property(cx, &key)?;
    FromPropertyDescriptor(cx, desc)
}

fn reflect_get_prototype_of(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    match target.get_prototype_of(cx)? {
        Some(proto) => Ok(Value::Object(proto)),
        None => Ok(Value::Null),
    }
}

fn reflect_has(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    Ok(Value::Boolean(target.has_property(cx, &key)?))
}

fn reflect_is_extensible(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    Ok(Value::Boolean(target.is_extensible(cx)?))
}

fn reflect_own_keys(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let keys = target
        .own_property_keys(cx)?
        .into_iter()
        .map(property_key_to_value)
        .collect();
    Ok(Value::Object(CreateArrayFromList(cx, keys)?))
}

fn reflect_prevent_extensions(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    Ok(Value::Boolean(target.prevent_extensions(cx)?))
}

fn reflect_set(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let value = args.get(2).cloned().unwrap_or(Value::Undefined);
    let receiver = args.get(3).cloned().unwrap_or(Value::Object(target));
    Ok(Value::Boolean(target.set(cx, key, value, receiver)?))
}

fn reflect_set_prototype_of(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_actual_object_arg(args.first())?;
    let proto = match args.get(1).cloned().unwrap_or(Value::Undefined) {
        Value::Object(object) => Some(object),
        Value::Null => None,
        _ => {
            return Err(JsError::type_error(
                "Reflect.setPrototypeOf prototype must be object or null",
            ))
        }
    };
    Ok(Value::Boolean(target.set_prototype_of(cx, proto)?))
}

fn property_key_to_value(key: PropertyKey) -> Value {
    match key {
        PropertyKey::String(value) => Value::String(value),
        PropertyKey::Symbol(symbol) => Value::Symbol(symbol),
    }
}

fn json_parse(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let text = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    JsonParser::new(&text).parse(cx)
}

fn json_stringify(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    match json_stringify_value(cx, value, &mut Vec::new())? {
        Some(text) => Ok(Value::String(text)),
        None => Ok(Value::Undefined),
    }
}

fn json_stringify_value(
    cx: &mut Context,
    value: Value,
    stack: &mut Vec<ObjectRef>,
) -> Completion<Option<String>> {
    match value {
        Value::Undefined | Value::Symbol(_) => Ok(None),
        Value::Null => Ok(Some("null".to_owned())),
        Value::Boolean(value) => Ok(Some(if value { "true" } else { "false" }.to_owned())),
        Value::String(value) => Ok(Some(json_quote_string(&value))),
        Value::Number(value) => {
            if value.is_finite() {
                Ok(Some(json_number_to_string(value)))
            } else {
                Ok(Some("null".to_owned()))
            }
        }
        Value::BigInt(_) => Err(JsError::type_error(
            "JSON.stringify cannot serialize BigInt",
        )),
        Value::Object(object) => json_stringify_object(cx, object, stack),
        Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => Ok(None),
    }
}

fn json_stringify_object(
    cx: &mut Context,
    object: ObjectRef,
    stack: &mut Vec<ObjectRef>,
) -> Completion<Option<String>> {
    if stack.contains(&object) {
        return Err(JsError::type_error(
            "JSON.stringify cannot serialize cyclic structures",
        ));
    }
    stack.push(object);
    let is_array = cx.heap().get(object)?.has_brand(Brand::Array);
    let result = if is_array {
        let len = length_of_array_like(cx, object)?;
        let mut parts = Vec::new();
        for index in 0..len {
            let value = object.get(
                cx,
                &PropertyKey::array_index(index as u64),
                Value::Object(object),
            )?;
            parts
                .push(json_stringify_value(cx, value, stack)?.unwrap_or_else(|| "null".to_owned()));
        }
        Some(format!("[{}]", parts.join(",")))
    } else {
        let mut parts = Vec::new();
        for key in object.own_property_keys(cx)? {
            let PropertyKey::String(name) = key.clone() else {
                continue;
            };
            let Some(desc) = object.get_own_property(cx, &key)? else {
                continue;
            };
            if !desc.enumerable() {
                continue;
            }
            let value = object.get(cx, &key, Value::Object(object))?;
            if let Some(serialized) = json_stringify_value(cx, value, stack)? {
                parts.push(format!("{}:{}", json_quote_string(&name), serialized));
            }
        }
        Some(format!("{{{}}}", parts.join(",")))
    };
    stack.pop();
    Ok(result)
}

fn json_quote_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch <= '\u{1f}' => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn json_number_to_string(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let text = value.to_string();
    if let Some(stripped) = text.strip_suffix(".0") {
        stripped.to_owned()
    } else {
        text
    }
}

struct JsonParser<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }

    fn parse(mut self, cx: &mut Context) -> Completion<Value> {
        self.skip_json_whitespace();
        let value = self.parse_value(cx)?;
        self.skip_json_whitespace();
        if !self.is_eof() {
            return Err(JsError::syntax("unexpected trailing JSON input"));
        }
        Ok(value)
    }

    fn parse_value(&mut self, cx: &mut Context) -> Completion<Value> {
        self.skip_json_whitespace();
        match self.peek_char() {
            Some('"') => self.parse_string().map(Value::String),
            Some('{') => self.parse_object(cx),
            Some('[') => self.parse_array(cx),
            Some('t') => {
                self.expect_literal("true")?;
                Ok(Value::Boolean(true))
            }
            Some('f') => {
                self.expect_literal("false")?;
                Ok(Value::Boolean(false))
            }
            Some('n') => {
                self.expect_literal("null")?;
                Ok(Value::Null)
            }
            Some('-' | '0'..='9') => self.parse_number().map(Value::Number),
            _ => Err(JsError::syntax("invalid JSON value")),
        }
    }

    fn parse_object(&mut self, cx: &mut Context) -> Completion<Value> {
        self.expect_char('{')?;
        let proto = cx
            .realm()?
            .intrinsics
            .get(IntrinsicId::ObjectPrototype)
            .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
        let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
        self.skip_json_whitespace();
        if self.consume_char('}') {
            return Ok(Value::Object(object));
        }
        loop {
            self.skip_json_whitespace();
            if self.peek_char() != Some('"') {
                return Err(JsError::syntax("JSON object keys must be strings"));
            }
            let key = self.parse_string()?;
            self.skip_json_whitespace();
            self.expect_char(':')?;
            let value = self.parse_value(cx)?;
            object.define_own_property_or_throw(
                cx,
                PropertyKey::from(key),
                Descriptor::data(value, true, true, true),
            )?;
            self.skip_json_whitespace();
            if self.consume_char('}') {
                break;
            }
            self.expect_char(',')?;
        }
        Ok(Value::Object(object))
    }

    fn parse_array(&mut self, cx: &mut Context) -> Completion<Value> {
        self.expect_char('[')?;
        let mut values = Vec::new();
        self.skip_json_whitespace();
        if self.consume_char(']') {
            return Ok(Value::Object(CreateArrayFromList(cx, values)?));
        }
        loop {
            values.push(self.parse_value(cx)?);
            self.skip_json_whitespace();
            if self.consume_char(']') {
                break;
            }
            self.expect_char(',')?;
        }
        Ok(Value::Object(CreateArrayFromList(cx, values)?))
    }

    fn parse_string(&mut self) -> Completion<String> {
        self.expect_char('"')?;
        let mut out = String::new();
        loop {
            let Some(ch) = self.next_char() else {
                return Err(JsError::syntax("unterminated JSON string"));
            };
            match ch {
                '"' => return Ok(out),
                '\\' => out.push(self.parse_escape()?),
                ch if ch <= '\u{1f}' => {
                    return Err(JsError::syntax("control character in JSON string"))
                }
                ch => out.push(ch),
            }
        }
    }

    fn parse_escape(&mut self) -> Completion<char> {
        let Some(ch) = self.next_char() else {
            return Err(JsError::syntax("unterminated JSON escape"));
        };
        match ch {
            '"' | '\\' | '/' => Ok(ch),
            'b' => Ok('\u{08}'),
            'f' => Ok('\u{0c}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => {
                let mut value = 0u32;
                for _ in 0..4 {
                    let Some(hex) = self.next_char().and_then(|c| c.to_digit(16)) else {
                        return Err(JsError::syntax("invalid JSON unicode escape"));
                    };
                    value = (value << 4) | hex;
                }
                char::from_u32(value).ok_or_else(|| JsError::syntax("invalid JSON unicode escape"))
            }
            _ => Err(JsError::syntax("invalid JSON escape")),
        }
    }

    fn parse_number(&mut self) -> Completion<f64> {
        let start = self.pos;
        self.consume_char('-');
        match self.peek_char() {
            Some('0') => {
                self.next_char();
            }
            Some('1'..='9') => {
                self.next_char();
                while matches!(self.peek_char(), Some('0'..='9')) {
                    self.next_char();
                }
            }
            _ => return Err(JsError::syntax("invalid JSON number")),
        }
        if self.consume_char('.') {
            if !matches!(self.peek_char(), Some('0'..='9')) {
                return Err(JsError::syntax("invalid JSON number"));
            }
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.next_char();
            }
        }
        if matches!(self.peek_char(), Some('e' | 'E')) {
            self.next_char();
            if matches!(self.peek_char(), Some('+' | '-')) {
                self.next_char();
            }
            if !matches!(self.peek_char(), Some('0'..='9')) {
                return Err(JsError::syntax("invalid JSON number"));
            }
            while matches!(self.peek_char(), Some('0'..='9')) {
                self.next_char();
            }
        }
        self.text[start..self.pos]
            .parse::<f64>()
            .map_err(|_| JsError::syntax("invalid JSON number"))
    }

    fn expect_literal(&mut self, literal: &str) -> Completion<()> {
        if self.text[self.pos..].starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(JsError::syntax("invalid JSON literal"))
        }
    }

    fn expect_char(&mut self, expected: char) -> Completion<()> {
        if self.consume_char(expected) {
            Ok(())
        } else {
            Err(JsError::syntax(format!(
                "expected '{expected}' in JSON input"
            )))
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.next_char();
            true
        } else {
            false
        }
    }

    fn skip_json_whitespace(&mut self) {
        while matches!(self.peek_char(), Some('\t' | '\n' | '\r' | ' ')) {
            self.next_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.text[self.pos..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.text.len()
    }
}

fn global_eval(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::String(source) => cx.eval_script(&source),
        value => Ok(value),
    }
}

fn global_parse_int(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let input = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let radix = match args.get(1).cloned().unwrap_or(Value::Undefined) {
        Value::Undefined => 0,
        value => to_int32(to_number_value(cx, value)?),
    };
    Ok(Value::Number(parse_int_like(&input, radix)))
}

fn global_parse_float(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let input = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Number(parse_float_like(&input)))
}

fn global_is_nan(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Boolean(value.is_nan()))
}

fn global_is_finite(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Boolean(value.is_finite()))
}

fn global_decode_uri(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let input = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::String(percent_decode(&input)?))
}

fn global_decode_uri_component(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let input = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::String(percent_decode(&input)?))
}

fn global_encode_uri(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let input = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::String(percent_encode(
        &input,
        ";/?:@&=+$,#-_.!~*'()",
    )))
}

fn global_encode_uri_component(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let input = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::String(percent_encode(&input, "-_.!~*'()")))
}

fn number_is_finite(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Boolean(matches!(
        args.first(),
        Some(Value::Number(value)) if value.is_finite()
    )))
}

fn number_is_integer(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Boolean(matches!(
        args.first(),
        Some(Value::Number(value)) if value.is_finite() && value.fract() == 0.0
    )))
}

fn number_is_nan(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Boolean(matches!(
        args.first(),
        Some(Value::Number(value)) if value.is_nan()
    )))
}

fn number_is_safe_integer(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    Ok(Value::Boolean(matches!(
        args.first(),
        Some(Value::Number(value))
            if value.is_finite() && value.fract() == 0.0 && value.abs() <= MAX_SAFE_INTEGER
    )))
}

fn math_abs(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(
        to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?.abs(),
    ))
}

fn math_acos(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::acos)
}

fn math_acosh(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::acosh)
}

fn math_asin(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::asin)
}

fn math_asinh(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::asinh)
}

fn math_atan(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::atan)
}

fn math_atan2(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let y = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let x = to_number_value(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Number(y.atan2(x)))
}

fn math_atanh(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::atanh)
}

fn math_cbrt(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::cbrt)
}

fn math_ceil(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(
        to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?.ceil(),
    ))
}

fn math_clz32(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let number = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let int = to_uint32(number);
    Ok(Value::Number(int.leading_zeros() as f64))
}

fn math_cos(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::cos)
}

fn math_cosh(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::cosh)
}

fn math_exp(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::exp)
}

fn math_expm1(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::exp_m1)
}

fn math_floor(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(
        to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?.floor(),
    ))
}

fn math_fround(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let number = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Number((number as f32) as f64))
}

fn math_f16round(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let number = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Number(round_to_f16_like(number)))
}

fn math_hypot(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let mut sum = 0.0;
    let mut saw_nan = false;
    for value in args {
        let number = to_number_value(cx, value.clone())?;
        if number.is_infinite() {
            return Ok(Value::Number(f64::INFINITY));
        }
        if number.is_nan() {
            saw_nan = true;
            continue;
        }
        sum += number * number;
    }
    if saw_nan {
        return Ok(Value::Number(f64::NAN));
    }
    Ok(Value::Number(sum.sqrt()))
}

fn math_imul(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let left = to_int32(to_number_value(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
    )?);
    let right = to_int32(to_number_value(
        cx,
        args.get(1).cloned().unwrap_or(Value::Undefined),
    )?);
    Ok(Value::Number(left.wrapping_mul(right) as f64))
}

fn math_log(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::ln)
}

fn math_log10(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::log10)
}

fn math_log1p(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::ln_1p)
}

fn math_log2(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::log2)
}

fn math_max(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    if args.is_empty() {
        return Ok(Value::Number(f64::NEG_INFINITY));
    }
    let mut result = f64::NEG_INFINITY;
    for value in args {
        let number = to_number_value(cx, value.clone())?;
        if number.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        result = result.max(number);
    }
    Ok(Value::Number(result))
}

fn math_min(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    if args.is_empty() {
        return Ok(Value::Number(f64::INFINITY));
    }
    let mut result = f64::INFINITY;
    for value in args {
        let number = to_number_value(cx, value.clone())?;
        if number.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        result = result.min(number);
    }
    Ok(Value::Number(result))
}

fn math_pow(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let base = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let exponent = to_number_value(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Number(base.powf(exponent)))
}

fn math_random(_cx: &mut Context, _this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(0.5))
}

fn math_round(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(
        to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?.round(),
    ))
}

fn math_sign(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let number = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    if number.is_nan() || number == 0.0 {
        Ok(Value::Number(number))
    } else {
        Ok(Value::Number(number.signum()))
    }
}

fn math_sin(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::sin)
}

fn math_sinh(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::sinh)
}

fn math_sqrt(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(
        to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?.sqrt(),
    ))
}

fn math_tan(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::tan)
}

fn math_tanh(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    math_unary(cx, args, f64::tanh)
}

fn math_trunc(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(
        to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?.trunc(),
    ))
}

fn math_sum_precise(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let iterable = args.first().cloned().unwrap_or(Value::Undefined);
    let values = collect_iterable_or_array_like(cx, iterable)?;
    let mut sum = 0.0;
    let mut correction = 0.0;
    for value in values {
        let number = to_number_value(cx, value)?;
        if number.is_nan() {
            return Ok(Value::Number(f64::NAN));
        }
        if number.is_infinite() {
            return Ok(Value::Number(number));
        }
        let adjusted = number - correction;
        let next = sum + adjusted;
        correction = (next - sum) - adjusted;
        sum = next;
    }
    Ok(Value::Number(sum))
}

fn math_unary(cx: &mut Context, args: &[Value], op: fn(f64) -> f64) -> Completion<Value> {
    Ok(Value::Number(op(to_number_value(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
    )?)))
}

fn parse_int_like(input: &str, radix: i32) -> f64 {
    let mut text = trim_ecmascript_whitespace_start(input);
    let sign = if let Some(rest) = text.strip_prefix('-') {
        text = rest;
        -1.0
    } else {
        if let Some(rest) = text.strip_prefix('+') {
            text = rest;
        }
        1.0
    };
    let mut radix = radix;
    if radix != 0 && !(2..=36).contains(&radix) {
        return f64::NAN;
    }
    if radix == 0 {
        radix = if text.starts_with("0x") || text.starts_with("0X") {
            16
        } else {
            10
        };
    }
    if radix == 16 {
        text = text
            .strip_prefix("0x")
            .or_else(|| text.strip_prefix("0X"))
            .unwrap_or(text);
    }
    let mut value = 0.0;
    let mut saw_digit = false;
    for ch in text.chars() {
        let Some(digit) = ch.to_digit(radix as u32) else {
            break;
        };
        saw_digit = true;
        value = value * radix as f64 + digit as f64;
    }
    if saw_digit {
        sign * value
    } else {
        f64::NAN
    }
}

fn parse_float_like(input: &str) -> f64 {
    let text = trim_ecmascript_whitespace_start(input);
    if text.starts_with("Infinity") {
        return f64::INFINITY;
    }
    if text.starts_with("+Infinity") {
        return f64::INFINITY;
    }
    if text.starts_with("-Infinity") {
        return f64::NEG_INFINITY;
    }
    let bytes = text.as_bytes();
    let mut pos = 0;
    if matches!(bytes.get(pos), Some(b'+' | b'-')) {
        pos += 1;
    }
    let digits_start = pos;
    while matches!(bytes.get(pos), Some(b'0'..=b'9')) {
        pos += 1;
    }
    let mut saw_digit = false;
    saw_digit |= pos > digits_start;
    if matches!(bytes.get(pos), Some(b'.')) {
        pos += 1;
        let frac_start = pos;
        while matches!(bytes.get(pos), Some(b'0'..=b'9')) {
            pos += 1;
        }
        saw_digit |= pos > frac_start;
    }
    if !saw_digit {
        return f64::NAN;
    }
    let mantissa_end = pos;
    if matches!(bytes.get(pos), Some(b'e' | b'E')) {
        let mut exp_pos = pos + 1;
        if matches!(bytes.get(exp_pos), Some(b'+' | b'-')) {
            exp_pos += 1;
        }
        let exp_start = exp_pos;
        while matches!(bytes.get(exp_pos), Some(b'0'..=b'9')) {
            exp_pos += 1;
        }
        if exp_pos > exp_start {
            pos = exp_pos;
        } else {
            pos = mantissa_end;
        }
    }
    text[..pos].parse::<f64>().unwrap_or(f64::NAN)
}

fn trim_ecmascript_whitespace_start(input: &str) -> &str {
    input.trim_start_matches(is_ecmascript_whitespace_or_line_terminator)
}

fn is_ecmascript_whitespace_or_line_terminator(ch: char) -> bool {
    if ('\u{2000}'..='\u{200A}').contains(&ch) {
        return true;
    }
    matches!(
        ch,
        '\u{0009}'
            | '\u{000A}'
            | '\u{000B}'
            | '\u{000C}'
            | '\u{000D}'
            | '\u{0020}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
            | '\u{FEFF}'
    )
}

fn to_uint32(number: f64) -> u32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    number.trunc().rem_euclid(4_294_967_296.0) as u32
}

fn to_int32(number: f64) -> i32 {
    let uint32 = to_uint32(number);
    if uint32 >= 2_147_483_648 {
        (uint32 as i64 - 4_294_967_296) as i32
    } else {
        uint32 as i32
    }
}

fn percent_decode(input: &str) -> Completion<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(JsError::uri_error("malformed URI escape"));
            }
            let hi = hex_digit(bytes[index + 1])?;
            let lo = hex_digit(bytes[index + 2])?;
            out.push((hi << 4) | lo);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).map_err(|_| JsError::uri_error("malformed URI sequence"))
}

fn percent_encode(input: &str, extra_unescaped: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || extra_unescaped.contains(ch) {
            out.push(ch);
        } else {
            let mut bytes = [0_u8; 4];
            for byte in ch.encode_utf8(&mut bytes).as_bytes() {
                out.push('%');
                out.push_str(&format!("{byte:02X}"));
            }
        }
    }
    out
}

fn hex_digit(byte: u8) -> Completion<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err(JsError::uri_error("malformed URI escape")),
    }
}

fn round_to_f16_like(number: f64) -> f64 {
    if number == 0.0 || !number.is_finite() {
        return number;
    }
    let sign = number.signum();
    let abs = number.abs();
    if abs > 65504.0 {
        return sign * f64::INFINITY;
    }
    if abs < 0.000000059604644775390625 {
        return sign * 0.0;
    }
    let exponent = abs.log2().floor();
    let step = 2_f64.powf(exponent - 10.0);
    sign * (abs / step).round() * step
}

fn array_constructor(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let values = if args.len() == 1 {
        match args[0] {
            Value::Number(length) => {
                if !length.is_finite()
                    || length < 0.0
                    || length.fract() != 0.0
                    || length > u32::MAX as f64
                {
                    return Err(JsError::range_error("invalid array length"));
                }
                let array = create_array_like_value(cx, Vec::new())?;
                array.define_own_property_or_throw(
                    cx,
                    PropertyKey::from("length"),
                    Descriptor::data(Value::Number(length), true, false, false),
                )?;
                return Ok(Value::Object(array));
            }
            _ => args.to_vec(),
        }
    } else {
        args.to_vec()
    };
    create_array_like(cx, values)
}

fn boolean_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = Value::Boolean(truthy(args.first().cloned().unwrap_or(Value::Undefined)));
    if !is_construct_call(&this) {
        return Ok(value);
    }
    create_primitive_wrapper(cx, IntrinsicId::BooleanPrototype, value)
}

fn number_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = if let Some(value) = args.first() {
        Value::Number(to_number_value(cx, value.clone())?)
    } else {
        Value::Number(0.0)
    };
    if !is_construct_call(&this) {
        return Ok(value);
    }
    create_primitive_wrapper(cx, IntrinsicId::NumberPrototype, value)
}

fn bigint_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if is_construct_call(&this) {
        return Err(JsError::type_error("BigInt is not a constructor"));
    }
    let value = args.first().cloned().unwrap_or(Value::Number(0.0));
    let primitive = ArgView::new(value).to_primitive(cx)?;
    let bigint = match primitive {
        Value::Number(value) => number_to_bigint(value)?,
        value => to_bigint_value(cx, value)?,
    };
    Ok(Value::BigInt(bigint))
}

fn bigint_as_int_n(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let bits = to_index_i128(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let value = to_bigint_value(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::BigInt(bigint_wrap_int(bits, value)))
}

fn bigint_as_uint_n(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let bits = to_index_i128(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let value = to_bigint_value(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::BigInt(bigint_wrap_uint(bits, value)))
}

fn bigint_proto_value_of(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::BigInt(this_bigint_value(cx, this)?))
}

fn bigint_proto_to_string(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_bigint_value(cx, this)?;
    let radix = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::Undefined => 10,
        value => {
            let number = to_number_value(cx, value)?;
            if !number.is_finite() || !(2.0..=36.0).contains(&number.trunc()) {
                return Err(JsError::range_error("BigInt radix is out of range"));
            }
            number.trunc() as u32
        }
    };
    Ok(Value::String(format_bigint_radix(value, radix)))
}

fn this_bigint_value(cx: &mut Context, value: Value) -> Completion<i128> {
    match value {
        Value::BigInt(value) => Ok(value),
        Value::Object(object) => match cx.heap().get(object)?.primitive_value.clone() {
            Some(Value::BigInt(value)) => Ok(value),
            _ => Err(JsError::type_error(
                "BigInt.prototype receiver is not BigInt",
            )),
        },
        _ => Err(JsError::type_error(
            "BigInt.prototype receiver is not BigInt",
        )),
    }
}

fn format_bigint_radix(value: i128, radix: u32) -> String {
    debug_assert!((2..=36).contains(&radix));
    if value == 0 {
        return "0".to_owned();
    }
    let negative = value < 0;
    let mut remaining = value.unsigned_abs();
    let mut digits = Vec::new();
    while remaining > 0 {
        let digit = (remaining % radix as u128) as u8;
        let ch = match digit {
            0..=9 => (b'0' + digit) as char,
            _ => (b'a' + digit - 10) as char,
        };
        digits.push(ch);
        remaining /= radix as u128;
    }
    if negative {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

fn to_bigint_value(cx: &mut Context, value: Value) -> Completion<i128> {
    let primitive = ArgView::new(value).to_primitive(cx)?;
    match primitive {
        Value::BigInt(value) => Ok(value),
        Value::Boolean(true) => Ok(1),
        Value::Boolean(false) => Ok(0),
        Value::String(value) => string_to_bigint(&value),
        Value::Number(_) => Err(JsError::type_error("cannot convert Number to BigInt")),
        other => Err(JsError::type_error(format!(
            "cannot convert {} to BigInt",
            other.type_name()
        ))),
    }
}

fn number_to_bigint(value: f64) -> Completion<i128> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(JsError::range_error(
            "number cannot be converted to BigInt because it is not an integer",
        ));
    }
    Ok(value as i128)
}

fn string_to_bigint(text: &str) -> Completion<i128> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(0);
    }
    if let Some(digits) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        return i128::from_str_radix(digits, 16)
            .map_err(|_| JsError::syntax("invalid BigInt string"));
    }
    if let Some(digits) = trimmed
        .strip_prefix("0o")
        .or_else(|| trimmed.strip_prefix("0O"))
    {
        return i128::from_str_radix(digits, 8)
            .map_err(|_| JsError::syntax("invalid BigInt string"));
    }
    if let Some(digits) = trimmed
        .strip_prefix("0b")
        .or_else(|| trimmed.strip_prefix("0B"))
    {
        return i128::from_str_radix(digits, 2)
            .map_err(|_| JsError::syntax("invalid BigInt string"));
    }
    trimmed
        .parse::<i128>()
        .map_err(|_| JsError::syntax("invalid BigInt string"))
}

fn to_index_i128(cx: &mut Context, value: Value) -> Completion<u32> {
    let number = to_number_value(cx, value)?;
    if number.is_nan() || number == 0.0 {
        return Ok(0);
    }
    if number < 0.0 {
        return Err(JsError::range_error("BigInt bit width is negative"));
    }
    if !number.is_finite() || number > u32::MAX as f64 {
        return Err(JsError::range_error("BigInt bit width is out of range"));
    }
    Ok(number.trunc() as u32)
}

fn bigint_wrap_uint(bits: u32, value: i128) -> i128 {
    if bits == 0 {
        return 0;
    }
    if bits >= 127 {
        return value;
    }
    let modulo = 1_i128 << bits;
    ((value % modulo) + modulo) % modulo
}

fn bigint_wrap_int(bits: u32, value: i128) -> i128 {
    if bits == 0 {
        return 0;
    }
    if bits >= 127 {
        return value;
    }
    let modulo = 1_i128 << bits;
    let mut wrapped = ((value % modulo) + modulo) % modulo;
    let threshold = 1_i128 << (bits - 1);
    if wrapped >= threshold {
        wrapped -= modulo;
    }
    wrapped
}

fn string_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = if let Some(value) = args.first() {
        match value {
            Value::Symbol(symbol) => Value::String(symbol_descriptive_string(cx, *symbol)),
            value => Value::String(to_string_value(cx, value.clone())?),
        }
    } else {
        Value::String(String::new())
    };
    if !is_construct_call(&this) {
        return Ok(value);
    }
    let object = create_primitive_wrapper_ref(cx, IntrinsicId::StringPrototype, value.clone())?;
    if let Value::String(text) = value {
        for (index, ch) in text.chars().enumerate() {
            object.define_own_property_or_throw(
                cx,
                PropertyKey::array_index(index as u64),
                Descriptor::data(Value::String(ch.to_string()), false, true, false),
            )?;
        }
        object.define_own_property_or_throw(
            cx,
            PropertyKey::from("length"),
            Descriptor::data(
                Value::Number(text.chars().count() as f64),
                false,
                false,
                false,
            ),
        )?;
    }
    Ok(Value::Object(object))
}

fn symbol_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if is_construct_call(&this) {
        return Err(JsError::type_error("Symbol is not a constructor"));
    }
    let description = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::Undefined => None,
        value => Some(to_string_value(cx, value)?),
    };
    let symbol = cx.fresh_symbol_with_description(description);
    Ok(Value::Symbol(symbol))
}

fn regexp_constructor(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let pattern = args.first().cloned().unwrap_or(Value::Undefined);
    let flags = args.get(1).cloned().unwrap_or(Value::Undefined);
    if let Value::Object(object) = pattern.clone() {
        if regexp_source_flags(cx, object).is_ok() && matches!(flags, Value::Undefined) {
            return Ok(Value::Object(object));
        }
    }
    let source = if matches!(pattern, Value::Undefined) {
        String::new()
    } else if let Value::Object(object) = pattern.clone() {
        if let Ok((source, _)) = regexp_source_flags(cx, object) {
            source
        } else {
            to_string_value(cx, pattern)?
        }
    } else {
        to_string_value(cx, pattern)?
    };
    let flags = if matches!(flags, Value::Undefined) {
        String::new()
    } else {
        to_string_value(cx, flags)?
    };
    RegExpCreate(cx, source, flags)
}

fn date_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if !is_construct_call(&this) {
        return Ok(Value::String("Invalid Date".to_owned()));
    }
    let time = if args.is_empty() {
        f64::NAN
    } else if args.len() == 1 {
        match args.first().cloned().unwrap_or(Value::Undefined) {
            Value::String(text) => parse_date_string(&text),
            Value::Object(object) if cx.heap().get(object)?.has_brand(super::Brand::Date) => {
                date_this_time_value(cx, Value::Object(object))?
            }
            Value::Object(_) => {
                let primitive = date_to_primitive_default(
                    cx,
                    args.first().cloned().unwrap_or(Value::Undefined),
                )?;
                match primitive {
                    Value::String(text) => parse_date_string(&text),
                    value => time_clip(to_number_value(cx, value)?),
                }
            }
            value => time_clip(to_number_value(cx, value)?),
        }
    } else {
        date_components_to_time(cx, args)?
    };
    let proto = prototype_from_new_target_or_intrinsic(
        cx,
        construct_new_target(&this),
        IntrinsicId::DatePrototype,
    )?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    let object_data = cx.heap_mut().get_mut(object)?;
    object_data.add_slot(InternalSlot::DateValue);
    object_data.primitive_value = Some(Value::Number(time));
    Ok(Value::Object(object))
}

fn date_to_primitive_default(cx: &mut Context, value: Value) -> Completion<Value> {
    let Value::Object(object) = value else {
        return Ok(value);
    };
    let exotic = object.get(
        cx,
        &PropertyKey::Symbol(SYMBOL_TO_PRIMITIVE_ID),
        Value::Object(object),
    )?;
    if !matches!(exotic, Value::Undefined | Value::Null) {
        if !cx.is_callable(&exotic)? {
            return Err(JsError::type_error(
                "Symbol.toPrimitive method is not callable",
            ));
        }
        let result = cx.call_mut(
            exotic,
            Value::Object(object),
            &[Value::String("default".to_owned())],
        )?;
        if !result.is_object() {
            return Ok(result);
        }
        return Err(JsError::type_error(
            "Symbol.toPrimitive method returned an object",
        ));
    }
    for name in ["valueOf", "toString"] {
        let method = object.get(cx, &PropertyKey::from(name), Value::Object(object))?;
        if matches!(method, Value::Undefined | Value::Null) || !cx.is_callable(&method)? {
            continue;
        }
        let result = cx.call_mut(method, Value::Object(object), &[])?;
        if !result.is_object() {
            return Ok(result);
        }
    }
    Err(JsError::type_error("cannot convert object to primitive"))
}

fn date_now(_cx: &mut Context, _this: Value, _args: &[Value]) -> Completion<Value> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as f64)
        .unwrap_or(f64::NAN);
    Ok(Value::Number(time_clip(now)))
}

fn date_parse(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let text = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Number(parse_date_string(&text)))
}

fn date_utc(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(date_components_to_time(cx, args)?))
}

fn date_components_to_time(cx: &mut Context, args: &[Value]) -> Completion<f64> {
    let year = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let month = if let Some(value) = args.get(1) {
        to_number_value(cx, value.clone())?
    } else {
        0.0
    };
    let date = if let Some(value) = args.get(2) {
        to_number_value(cx, value.clone())?
    } else {
        1.0
    };
    let hours = if let Some(value) = args.get(3) {
        to_number_value(cx, value.clone())?
    } else {
        0.0
    };
    let minutes = if let Some(value) = args.get(4) {
        to_number_value(cx, value.clone())?
    } else {
        0.0
    };
    let seconds = if let Some(value) = args.get(5) {
        to_number_value(cx, value.clone())?
    } else {
        0.0
    };
    let millis = if let Some(value) = args.get(6) {
        to_number_value(cx, value.clone())?
    } else {
        0.0
    };

    if [year, month, date, hours, minutes, seconds, millis]
        .iter()
        .any(|value| value.is_nan() || !value.is_finite())
    {
        return Ok(f64::NAN);
    }
    let year_integer = year.trunc();
    let full_year = if (0.0..=99.0).contains(&year_integer) {
        year_integer + 1900.0
    } else {
        year
    };
    Ok(time_clip(make_date(
        make_day(full_year, month, date),
        make_time(hours, minutes, seconds, millis),
    )))
}

fn make_time(hours: f64, minutes: f64, seconds: f64, millis: f64) -> f64 {
    ((hours.trunc() * 3_600_000.0 + minutes.trunc() * 60_000.0) + seconds.trunc() * 1000.0)
        + millis.trunc()
}

fn make_date(day: f64, time: f64) -> f64 {
    day * 86_400_000.0 + time
}

fn make_day(year: f64, month: f64, date: f64) -> f64 {
    if !year.is_finite() || !month.is_finite() || !date.is_finite() {
        return f64::NAN;
    }
    let year = year.trunc();
    let month = month.trunc();
    let date = date.trunc();
    let year = year + (month / 12.0).floor();
    let month = month.rem_euclid(12.0) + 1.0;
    days_from_civil(year as i32, month as u32, 1) as f64 + (date - 1.0)
}

fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) as i64
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096).div_euclid(365);
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2).div_euclid(153);
    let day = doy - (153 * mp + 2).div_euclid(5) + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}

fn time_clip(time: f64) -> f64 {
    if !time.is_finite() || time.abs() > 8_640_000_000_000_000.0 {
        return f64::NAN;
    }
    let clipped = time.trunc();
    if clipped == 0.0 {
        0.0
    } else {
        clipped
    }
}

fn parse_date_string(text: &str) -> f64 {
    let text = text.trim();
    if let Some(value) = parse_formatted_date_string(text) {
        return value;
    }
    let bytes = text.as_bytes();
    let (year_start, year_end, signed_extended) = match bytes.first() {
        Some(b'+' | b'-') => (1, 7, true),
        _ => (0, 4, false),
    };
    let Some(year_digits) = text.get(year_start..year_end) else {
        return f64::NAN;
    };
    if !year_digits.chars().all(|ch| ch.is_ascii_digit()) {
        return f64::NAN;
    }
    let mut year_text = String::new();
    if signed_extended && bytes.first() == Some(&b'-') {
        year_text.push('-');
    }
    year_text.push_str(year_digits);
    let Ok(year) = year_text.parse::<i32>() else {
        return f64::NAN;
    };
    if text.len() == year_end {
        return time_clip(make_date(make_day(year as f64, 0.0, 1.0), 0.0));
    }
    let month_sep = year_end;
    let day_sep = year_end + 3;
    if text.len() >= year_end + 6
        && text.as_bytes().get(month_sep) == Some(&b'-')
        && text.as_bytes().get(day_sep) == Some(&b'-')
    {
        let month = text[year_end + 1..year_end + 3].parse::<i32>().ok();
        let day = text[year_end + 4..year_end + 6].parse::<i32>().ok();
        if let (Some(month), Some(day)) = (month, day) {
            let time_start = year_end + 6;
            let time = if text.len() >= time_start + 10
                && text.as_bytes().get(time_start) == Some(&b'T')
            {
                let hour = text[time_start + 1..time_start + 3]
                    .parse::<f64>()
                    .unwrap_or(f64::NAN);
                let minute = text[time_start + 4..time_start + 6]
                    .parse::<f64>()
                    .unwrap_or(f64::NAN);
                let second = text[time_start + 7..time_start + 9]
                    .parse::<f64>()
                    .unwrap_or(f64::NAN);
                let millis = if text.as_bytes().get(time_start + 9) == Some(&b'.')
                    && text.len() >= time_start + 13
                {
                    text[time_start + 10..time_start + 13]
                        .parse::<f64>()
                        .unwrap_or(f64::NAN)
                } else {
                    0.0
                };
                make_time(hour, minute, second, millis)
            } else {
                0.0
            };
            return time_clip(make_date(
                make_day(year as f64, (month - 1) as f64, day as f64),
                time,
            ));
        }
    }
    f64::NAN
}

fn parse_formatted_date_string(text: &str) -> Option<f64> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() == 6 && parts.get(5) == Some(&"GMT") {
        let date = parts.get(1)?.parse::<f64>().ok()?;
        let month = date_month_number(parts.get(2)?)? as f64 - 1.0;
        let year = parts.get(3)?.parse::<f64>().ok()?;
        let (hour, minute, second) = parse_date_time_parts(parts.get(4)?)?;
        return Some(time_clip(make_date(
            make_day(year, month, date),
            make_time(hour, minute, second, 0.0),
        )));
    }
    if parts.len() >= 6 && parts.get(5).is_some_and(|part| part.starts_with("GMT")) {
        let month = date_month_number(parts.get(1)?)? as f64 - 1.0;
        let date = parts.get(2)?.parse::<f64>().ok()?;
        let year = parts.get(3)?.parse::<f64>().ok()?;
        let (hour, minute, second) = parse_date_time_parts(parts.get(4)?)?;
        return Some(time_clip(make_date(
            make_day(year, month, date),
            make_time(hour, minute, second, 0.0),
        )));
    }
    None
}

fn parse_date_time_parts(text: &str) -> Option<(f64, f64, f64)> {
    let mut parts = text.split(':');
    let hour = parts.next()?.parse::<f64>().ok()?;
    let minute = parts.next()?.parse::<f64>().ok()?;
    let second = parts.next()?.parse::<f64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((hour, minute, second))
}

fn date_month_number(name: &str) -> Option<u32> {
    Some(match name {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    })
}

fn iso_string_from_time(time: f64) -> Completion<String> {
    if time.is_nan() {
        return Err(JsError::range_error("Invalid time value"));
    }
    let day = (time / 86_400_000.0).floor() as i64;
    let time_within_day = time - day as f64 * 86_400_000.0;
    let (year, month, date) = civil_from_days(day);
    let hour = (time_within_day / 3_600_000.0).floor() as u32;
    let minute = ((time_within_day % 3_600_000.0) / 60_000.0).floor() as u32;
    let second = ((time_within_day % 60_000.0) / 1000.0).floor() as u32;
    let millis = (time_within_day % 1000.0).floor() as u32;
    let year = if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else if year < 0 {
        format!("-{:06}", -(year as i64))
    } else {
        format!("+{year:06}")
    };
    Ok(format!(
        "{year}-{month:02}-{date:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z"
    ))
}

fn date_weekday_name(day: u32) -> &'static str {
    match day {
        0 => "Sun",
        1 => "Mon",
        2 => "Tue",
        3 => "Wed",
        4 => "Thu",
        5 => "Fri",
        _ => "Sat",
    }
}

fn date_month_name(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        _ => "Dec",
    }
}

fn date_display_year(year: i32) -> String {
    if year < 0 {
        format!("-{:04}", -(year as i64))
    } else {
        format!("{year:04}")
    }
}

fn date_date_string(time: f64) -> Completion<String> {
    let Some((year, month, date, weekday, _, _, _, _)) = date_utc_parts(time) else {
        return Err(JsError::range_error("Invalid time value"));
    };
    Ok(format!(
        "{} {} {:02} {}",
        date_weekday_name(weekday),
        date_month_name(month),
        date,
        date_display_year(year)
    ))
}

fn date_time_string(time: f64) -> Completion<String> {
    let Some((_, _, _, _, hour, minute, second, _)) = date_utc_parts(time) else {
        return Err(JsError::range_error("Invalid time value"));
    };
    Ok(format!("{hour:02}:{minute:02}:{second:02} GMT+0000"))
}

fn date_to_string(time: f64) -> Completion<String> {
    Ok(format!(
        "{} {}",
        date_date_string(time)?,
        date_time_string(time)?
    ))
}

fn date_utc_string(time: f64) -> Completion<String> {
    let Some((year, month, date, weekday, hour, minute, second, _)) = date_utc_parts(time) else {
        return Err(JsError::range_error("Invalid time value"));
    };
    Ok(format!(
        "{}, {:02} {} {} {hour:02}:{minute:02}:{second:02} GMT",
        date_weekday_name(weekday),
        date,
        date_month_name(month),
        date_display_year(year)
    ))
}

fn create_primitive_wrapper(
    cx: &mut Context,
    proto_id: IntrinsicId,
    primitive: Value,
) -> Completion<Value> {
    create_primitive_wrapper_ref(cx, proto_id, primitive).map(Value::Object)
}

fn create_primitive_wrapper_ref(
    cx: &mut Context,
    proto_id: IntrinsicId,
    primitive: Value,
) -> Completion<ObjectRef> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(proto_id)
        .ok_or_else(|| JsError::internal("missing primitive wrapper prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    let object_data = cx.heap_mut().get_mut(object)?;
    object_data.primitive_value = Some(primitive.clone());
    object_data.add_slot(InternalSlot::PrimitiveValue(primitive));
    Ok(object)
}

fn error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "Error",
        IntrinsicId::ErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn type_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "TypeError",
        IntrinsicId::TypeErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn range_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "RangeError",
        IntrinsicId::RangeErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn reference_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "ReferenceError",
        IntrinsicId::ReferenceErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn syntax_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "SyntaxError",
        IntrinsicId::SyntaxErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn eval_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "EvalError",
        IntrinsicId::EvalErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn uri_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "URIError",
        IntrinsicId::URIErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn test262_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_error_value(
        cx,
        "Test262Error",
        IntrinsicId::Test262ErrorPrototype,
        construct_new_target(&this),
        args.first(),
        args.get(1),
    )
}

fn aggregate_error_constructor(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let error = create_error_value(
        cx,
        "AggregateError",
        IntrinsicId::AggregateErrorPrototype,
        construct_new_target(&this),
        args.get(1),
        args.get(2),
    )?;
    let Value::Object(object) = error.clone() else {
        return Ok(error);
    };
    let errors =
        collect_iterable_or_array_like(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let errors = create_array_like(cx, errors)?;
    define_error_field(cx, object, "errors", errors)?;
    Ok(error)
}

fn create_error_value(
    cx: &mut Context,
    name: &str,
    proto_id: IntrinsicId,
    new_target: Option<ObjectRef>,
    message: Option<&Value>,
    options: Option<&Value>,
) -> Completion<Value> {
    let proto = prototype_from_new_target_or_intrinsic(cx, new_target, proto_id)?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    cx.heap_mut()
        .get_mut(object)?
        .add_slot(InternalSlot::ErrorData);
    define_error_field(cx, object, "name", Value::String(name.to_owned()))?;
    if let Some(value) = message {
        if !matches!(value, Value::Undefined) {
            let message = Value::String(to_string_value(cx, value.clone())?);
            define_error_field(cx, object, "message", message)?;
        }
    }
    install_error_cause(cx, object, options)?;
    Ok(Value::Object(object))
}

fn error_is_error(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let Some(Value::Object(object)) = args.first() else {
        return Ok(Value::Boolean(false));
    };
    Ok(Value::Boolean(
        cx.heap().get(*object)?.has_brand(Brand::Error),
    ))
}

fn prototype_from_new_target_or_intrinsic(
    cx: &mut Context,
    new_target: Option<ObjectRef>,
    intrinsic: IntrinsicId,
) -> Completion<ObjectRef> {
    if let Some(new_target) = new_target {
        let proto = new_target.get(
            cx,
            &PropertyKey::from("prototype"),
            Value::Object(new_target),
        )?;
        if let Value::Object(proto) = proto {
            return Ok(proto);
        }
    }
    cx.realm()?
        .intrinsics
        .get(intrinsic)
        .ok_or_else(|| JsError::internal("missing intrinsic prototype"))
}

fn install_error_cause(
    cx: &mut Context,
    object: ObjectRef,
    options: Option<&Value>,
) -> Completion<()> {
    let Some(Value::Object(options)) = options else {
        return Ok(());
    };
    let cause_key = PropertyKey::from("cause");
    if options.has_property(cx, &cause_key)? {
        let cause = options.get(cx, &cause_key, Value::Object(*options))?;
        define_error_field(cx, object, "cause", cause)?;
    }
    Ok(())
}

fn define_error_field(
    cx: &mut Context,
    object: ObjectRef,
    name: &str,
    value: Value,
) -> Completion<()> {
    object.define_own_property_or_throw(
        cx,
        PropertyKey::from(name),
        Descriptor::data(value, true, false, true),
    )
}

fn object_constructor(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    if let Value::Object(_) = value {
        return Ok(value);
    }
    if matches!(value, Value::Undefined | Value::Null) {
        let proto = cx
            .realm()?
            .intrinsics
            .get(IntrinsicId::ObjectPrototype)
            .ok_or_else(|| super::JsError::internal("missing Object.prototype intrinsic"))?;
        let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
        return Ok(Value::Object(object));
    }
    Ok(Value::Object(ArgView::new(value).to_object(cx)?))
}

fn object_create(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let proto = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::Object(object) => Some(object),
        Value::Null => None,
        _ => {
            return Err(JsError::type_error(
                "Object.create prototype must be object or null",
            ))
        }
    };

    let object = cx.heap_mut().allocate(JsObject::ordinary(proto));
    if let Some(properties) = args.get(1) {
        if !matches!(properties, Value::Undefined) {
            define_properties(cx, object, properties.clone())?;
        }
    }
    Ok(Value::Object(object))
}

fn object_define_property(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_actual_object_arg(args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let desc = ToPropertyDescriptor(cx, args.get(2).cloned().unwrap_or(Value::Undefined))?;
    object.define_own_property_or_throw(cx, key, desc)?;
    Ok(Value::Object(object))
}

fn object_get_own_property_descriptor(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let desc = object.get_own_property(cx, &key)?;
    FromPropertyDescriptor(cx, desc)
}

fn object_get_prototype_of(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(_cx, args.first())?;
    match object.get_prototype_of(_cx)? {
        Some(proto) => Ok(Value::Object(proto)),
        None => Ok(Value::Null),
    }
}

fn object_set_prototype_of(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let proto = match args.get(1).cloned().unwrap_or(Value::Undefined) {
        Value::Object(object) => Some(object),
        Value::Null => None,
        _ => {
            return Err(JsError::type_error(
                "Object.setPrototypeOf prototype must be object or null",
            ))
        }
    };
    if !object.set_prototype_of(cx, proto)? {
        return Err(JsError::type_error("could not set object prototype"));
    }
    Ok(Value::Object(object))
}

fn object_prevent_extensions(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let Value::Object(object) = value else {
        return Ok(value);
    };
    if !object.prevent_extensions(cx)? {
        return Err(JsError::type_error(
            "Object.preventExtensions internal method returned false",
        ));
    }
    Ok(Value::Object(object))
}

fn object_is_extensible(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let Some(value) = args.first() else {
        return Ok(Value::Boolean(false));
    };
    let Value::Object(object) = value else {
        return Ok(Value::Boolean(false));
    };
    Ok(Value::Boolean(object.is_extensible(cx)?))
}

fn object_has_own(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    Ok(Value::Boolean(object.get_own_property(cx, &key)?.is_some()))
}

fn object_is(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let left = args.first().cloned().unwrap_or(Value::Undefined);
    let right = args.get(1).cloned().unwrap_or(Value::Undefined);
    Ok(Value::Boolean(SameValue(&left, &right)))
}

fn object_keys(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let mut values = Vec::new();
    for key in object.own_property_keys(cx)? {
        let Some(desc) = object.get_own_property(cx, &key)? else {
            continue;
        };
        if desc.enumerable() {
            if let PropertyKey::String(name) = key {
                values.push(Value::String(name));
            }
        }
    }
    create_array_like(cx, values)
}

fn object_values(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let mut values = Vec::new();
    for key in object.own_property_keys(cx)? {
        let Some(desc) = object.get_own_property(cx, &key)? else {
            continue;
        };
        if desc.enumerable() && matches!(key, PropertyKey::String(_)) {
            values.push(object.get(cx, &key, Value::Object(object))?);
        }
    }
    create_array_like(cx, values)
}

fn object_entries(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let mut values = Vec::new();
    for key in object.own_property_keys(cx)? {
        let Some(desc) = object.get_own_property(cx, &key)? else {
            continue;
        };
        if desc.enumerable() {
            if let PropertyKey::String(name) = key.clone() {
                let value = object.get(cx, &key, Value::Object(object))?;
                values.push(Value::Object(create_array_like_value(
                    cx,
                    vec![Value::String(name), value],
                )?));
            }
        }
    }
    create_array_like(cx, values)
}

fn object_from_entries(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let entries = args.first().cloned().unwrap_or(Value::Undefined);
    let proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let result = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    let record = get_iterator(cx, entries)?;
    loop {
        let entry = match iterator_step_value(cx, &record) {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        if let Err(error) = add_from_entries_pair(cx, result, entry) {
            return Err(iterator_close_error(cx, &record, error));
        }
    }
    Ok(Value::Object(result))
}

fn object_group_by(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let items = args.first().cloned().unwrap_or(Value::Undefined);
    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Object.groupBy callback is not callable",
        ));
    }
    let result = cx.heap_mut().allocate(JsObject::ordinary(None));
    let record = get_iterator(cx, items)?;
    let mut index = 0_u32;
    loop {
        let value = match iterator_step_value(cx, &record) {
            Ok(Some(value)) => value,
            Ok(None) => break,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        let key_value = match cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value.clone(), Value::Number(index as f64)],
        ) {
            Ok(value) => value,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        let key = match ArgView::new(key_value).to_property_key(cx) {
            Ok(key) => key,
            Err(error) => return Err(iterator_close_error(cx, &record, error)),
        };
        let group = result.get(cx, &key, Value::Object(result))?;
        let group_object = if let Value::Object(group_object) = group {
            group_object
        } else {
            let created = create_array_like_value(cx, Vec::new())?;
            CreateDataPropertyOrThrow(cx, result, key.clone(), Value::Object(created))?;
            created
        };
        let length = length_of_array_like(cx, group_object)?;
        CreateDataPropertyOrThrow(
            cx,
            group_object,
            PropertyKey::array_index(length as u64),
            value,
        )?;
        index = index
            .checked_add(1)
            .ok_or_else(|| JsError::type_error("Object.groupBy index overflow"))?;
    }
    Ok(Value::Object(result))
}

fn add_from_entries_pair(cx: &mut Context, result: ObjectRef, entry: Value) -> Completion<()> {
    let Value::Object(entry_object) = entry else {
        return Err(JsError::type_error(
            "Object.fromEntries entry is not object",
        ));
    };
    let key_value = entry_object.get(
        cx,
        &PropertyKey::array_index(0),
        Value::Object(entry_object),
    )?;
    let value = entry_object.get(
        cx,
        &PropertyKey::array_index(1),
        Value::Object(entry_object),
    )?;
    let key = ArgView::new(key_value).to_property_key(cx)?;
    CreateDataPropertyOrThrow(cx, result, key, value)
}

fn object_define_properties(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_actual_object_arg(args.first())?;
    let properties = args.get(1).cloned().unwrap_or(Value::Undefined);
    define_properties(cx, object, properties)?;
    Ok(Value::Object(object))
}

fn object_get_own_property_names(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let values = object
        .own_property_keys(cx)?
        .into_iter()
        .filter_map(|key| match key {
            PropertyKey::String(value) => Some(Value::String(value)),
            PropertyKey::Symbol(_) => None,
        })
        .collect();
    create_array_like(cx, values)
}

fn object_get_own_property_symbols(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let values = object
        .own_property_keys(cx)?
        .into_iter()
        .filter_map(|key| match key {
            PropertyKey::String(_) => None,
            PropertyKey::Symbol(value) => Some(Value::Symbol(value)),
        })
        .collect();
    create_array_like(cx, values)
}

fn object_get_own_property_descriptors(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let descriptors = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    for key in object.own_property_keys(cx)? {
        let desc = object.get_own_property(cx, &key)?;
        let desc_value = FromPropertyDescriptor(cx, desc)?;
        descriptors.define_own_property_or_throw(
            cx,
            key,
            Descriptor::data(desc_value, true, true, true),
        )?;
    }
    Ok(Value::Object(descriptors))
}

fn object_assign(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let target = require_object_arg(cx, args.first())?;
    for source in args.iter().skip(1) {
        if matches!(source, Value::Undefined | Value::Null) {
            continue;
        }
        let source = ArgView::new(source.clone()).to_object(cx)?;
        for key in source.own_property_keys(cx)? {
            let Some(desc) = source.get_own_property(cx, &key)? else {
                continue;
            };
            if desc.enumerable() {
                let value = source.get(cx, &key, Value::Object(source))?;
                set_property_or_throw(cx, target, key, value)?;
            }
        }
    }
    Ok(Value::Object(target))
}

fn object_freeze(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let Value::Object(object) = value else {
        return Ok(value);
    };
    set_integrity(cx, object, true)?;
    Ok(Value::Object(object))
}

fn object_seal(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let Value::Object(object) = value else {
        return Ok(value);
    };
    set_integrity(cx, object, false)?;
    Ok(Value::Object(object))
}

fn object_is_frozen(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let Some(value) = args.first() else {
        return Ok(Value::Boolean(true));
    };
    let Value::Object(object) = value else {
        return Ok(Value::Boolean(true));
    };
    Ok(Value::Boolean(cx.heap().get(*object)?.frozen))
}

fn object_is_sealed(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let Some(value) = args.first() else {
        return Ok(Value::Boolean(true));
    };
    let Value::Object(object) = value else {
        return Ok(Value::Boolean(true));
    };
    let object_data = cx.heap().get(*object)?;
    Ok(Value::Boolean(object_data.sealed || object_data.frozen))
}

fn object_proto_has_own_property(
    cx: &mut Context,
    this: Value,
    args: &[Value],
) -> Completion<Value> {
    let object = ArgView::new(this).to_object(cx)?;
    let key = to_property_key_arg(cx, args.first())?;
    Ok(Value::Boolean(object.get_own_property(cx, &key)?.is_some()))
}

fn object_proto_property_is_enumerable(
    cx: &mut Context,
    this: Value,
    args: &[Value],
) -> Completion<Value> {
    let object = ArgView::new(this).to_object(cx)?;
    let key = to_property_key_arg(cx, args.first())?;
    Ok(Value::Boolean(
        object
            .get_own_property(cx, &key)?
            .map(|desc| desc.enumerable())
            .unwrap_or(false),
    ))
}

fn object_proto_value_of(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::Object(ArgView::new(this).to_object(cx)?))
}

fn object_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let builtin_tag = match this.clone() {
        Value::InternalConstruct | Value::InternalConstructWithNewTarget(_) => "Object",
        Value::Undefined => "Undefined",
        Value::Null => "Null",
        Value::Boolean(_) => "Boolean",
        Value::String(_) => "String",
        Value::Number(_) => "Number",
        Value::BigInt(_) => "BigInt",
        Value::Symbol(_) => "Symbol",
        Value::Object(object) => match cx.heap().get(object)?.kind {
            super::ObjectKind::Array => "Array",
            super::ObjectKind::Arguments => "Arguments",
            super::ObjectKind::Function(_) => "Function",
            _ if cx.heap().get(object)?.has_brand(super::Brand::Date) => "Date",
            _ if cx.heap().get(object)?.has_brand(super::Brand::Error) => "Error",
            _ => match primitive_wrapper_value(cx, object)? {
                Some(Value::Boolean(_)) => "Boolean",
                Some(Value::Number(_)) => "Number",
                Some(Value::String(_)) => "String",
                Some(Value::Symbol(_)) => "Symbol",
                Some(Value::BigInt(_)) => "BigInt",
                _ => "Object",
            },
        },
    };
    let tag = if let Value::Object(object) = this {
        match object.get(
            cx,
            &PropertyKey::Symbol(SYMBOL_TO_STRING_TAG_ID),
            Value::Object(object),
        )? {
            Value::String(tag) => tag,
            _ => builtin_tag.to_owned(),
        }
    } else {
        builtin_tag.to_owned()
    };
    Ok(Value::String(format!("[object {tag}]")))
}

fn object_proto_to_locale_string(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    let object = ArgView::new(this.clone()).to_object(cx)?;
    let to_string = object.get(cx, &PropertyKey::from("toString"), this.clone())?;
    cx.call_mut(to_string, this, &[])
}

fn object_proto_is_prototype_of(
    cx: &mut Context,
    this: Value,
    args: &[Value],
) -> Completion<Value> {
    let proto = ArgView::new(this).to_object(cx)?;
    let Some(Value::Object(mut object)) = args.first().cloned() else {
        return Ok(Value::Boolean(false));
    };
    while let Some(next) = object.get_prototype_of(cx)? {
        if next == proto {
            return Ok(Value::Boolean(true));
        }
        object = next;
    }
    Ok(Value::Boolean(false))
}

fn object_proto_get_proto(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let object = ArgView::new(this).to_object(cx)?;
    Ok(object
        .get_prototype_of(cx)?
        .map(Value::Object)
        .unwrap_or(Value::Null))
}

fn object_proto_set_proto(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if matches!(this, Value::Undefined | Value::Null) {
        return Err(JsError::type_error(
            "Object.prototype.__proto__ setter receiver is not object coercible",
        ));
    }
    let proto = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::Object(object) => Some(object),
        Value::Null => None,
        _ => return Ok(Value::Undefined),
    };
    let Value::Object(object) = this else {
        return Ok(Value::Undefined);
    };
    if !object.set_prototype_of(cx, proto)? {
        return Err(JsError::type_error("could not set object prototype"));
    }
    Ok(Value::Undefined)
}

fn object_proto_define_getter(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    object_proto_define_accessor(cx, this, args, true)
}

fn object_proto_define_setter(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    object_proto_define_accessor(cx, this, args, false)
}

fn object_proto_define_accessor(
    cx: &mut Context,
    this: Value,
    args: &[Value],
    getter: bool,
) -> Completion<Value> {
    let object = ArgView::new(this).to_object(cx)?;
    let key = to_property_key_arg(cx, args.first())?;
    let callable = args.get(1).cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callable)? {
        return Err(JsError::type_error("accessor must be callable"));
    }
    let desc = if getter {
        Descriptor::accessor(Some(callable), None, true, true)
    } else {
        Descriptor::accessor(None, Some(callable), true, true)
    };
    object.define_own_property_or_throw(cx, key, desc)?;
    Ok(Value::Undefined)
}

fn object_proto_lookup_getter(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    object_proto_lookup_accessor(cx, this, args, true)
}

fn object_proto_lookup_setter(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    object_proto_lookup_accessor(cx, this, args, false)
}

fn object_proto_lookup_accessor(
    cx: &mut Context,
    this: Value,
    args: &[Value],
    getter: bool,
) -> Completion<Value> {
    let key = to_property_key_arg(cx, args.first())?;
    let mut object = Some(ArgView::new(this).to_object(cx)?);
    while let Some(current) = object {
        if let Some(desc) = current.get_own_property(cx, &key)? {
            if desc.is_accessor_descriptor() {
                return Ok(if getter {
                    desc.get.unwrap_or(Value::Undefined)
                } else {
                    desc.set.unwrap_or(Value::Undefined)
                });
            }
            return Ok(Value::Undefined);
        }
        object = current.get_prototype_of(cx)?;
    }
    Ok(Value::Undefined)
}

fn function_proto_call(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let this_arg = args.first().cloned().unwrap_or(Value::Undefined);
    let call_args = if args.len() > 1 { &args[1..] } else { &[] };
    cx.call_mut(this, this_arg, call_args)
}

fn function_proto_apply(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let this_arg = args.first().cloned().unwrap_or(Value::Undefined);
    let apply_args = collect_array_like(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    cx.call_mut(this, this_arg, &apply_args)
}

fn function_proto_bind(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    if !cx.is_callable(&this)? {
        return Err(JsError::type_error(
            "Function.prototype.bind target is not callable",
        ));
    }
    let this_arg = args.first().cloned().unwrap_or(Value::Undefined);
    let bound_args = if args.len() > 1 {
        args[1..].to_vec()
    } else {
        Vec::new()
    };
    let length = bound_function_length(cx, &this, bound_args.len())?;
    let name = format!("bound {}", bound_function_target_name(cx, &this)?);
    let function_proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::FunctionPrototype)
        .ok_or_else(|| JsError::internal("missing Function.prototype intrinsic"))?;
    let function = cx.heap_mut().allocate(JsObject::function(
        Some(function_proto),
        FunctionData::bound(
            name.clone(),
            function_data_length(length),
            this,
            this_arg,
            bound_args,
        ),
    ));
    define_data_with_attrs(
        cx.heap_mut(),
        function,
        "length",
        Value::Number(length),
        false,
        false,
        true,
    );
    define_data_with_attrs(
        cx.heap_mut(),
        function,
        "name",
        Value::String(name),
        false,
        false,
        true,
    );
    Ok(Value::Object(function))
}

fn array_is_array(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let Some(Value::Object(object)) = args.first() else {
        return Ok(Value::Boolean(false));
    };
    Ok(Value::Boolean(is_array_object(cx, *object)?))
}

fn is_array_object(cx: &Context, object: ObjectRef) -> Completion<bool> {
    if let Some(target) = proxy_target(cx, object)? {
        return is_array_object(cx, target);
    }
    Ok(matches!(
        cx.heap().get(object)?.kind,
        super::ObjectKind::Array
    ))
}

fn array_from(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let items = args.first().cloned().unwrap_or(Value::Undefined);
    if matches!(items, Value::Undefined | Value::Null) {
        return Err(JsError::type_error(
            "Array.from requires an array-like or iterable",
        ));
    }
    let mapper = args.get(1).cloned().unwrap_or(Value::Undefined);
    let has_mapper = !matches!(mapper, Value::Undefined);
    if has_mapper && !cx.is_callable(&mapper)? {
        return Err(JsError::type_error("Array.from mapper is not callable"));
    }
    let this_arg = args.get(2).cloned().unwrap_or(Value::Undefined);
    let object = ArgView::new(items.clone()).to_object(cx)?;
    let iterator_method =
        object.get(cx, &PropertyKey::Symbol(SYMBOL_ITERATOR_ID), items.clone())?;

    if !matches!(iterator_method, Value::Undefined | Value::Null) {
        if !cx.is_callable(&iterator_method)? {
            return Err(JsError::type_error("Array.from @@iterator is not callable"));
        }
        let result = create_array_from_target(cx, this.clone(), None)?;
        let record = get_iterator(cx, items)?;
        let mut index = 0usize;
        loop {
            let value = match iterator_step_value(cx, &record) {
                Ok(Some(value)) => value,
                Ok(None) => break,
                Err(error) => return Err(iterator_close_error(cx, &record, error)),
            };
            let mapped = match map_array_from_value(
                cx,
                mapper.clone(),
                has_mapper,
                this_arg.clone(),
                value,
                index,
            ) {
                Ok(mapped) => mapped,
                Err(error) => return Err(iterator_close_error(cx, &record, error)),
            };
            if let Err(error) = CreateDataPropertyOrThrow(
                cx,
                result,
                PropertyKey::array_index(index as u64),
                mapped,
            ) {
                return Err(iterator_close_error(cx, &record, error));
            }
            index += 1;
        }
        if let Err(error) = set_property_or_throw(
            cx,
            result,
            PropertyKey::from("length"),
            Value::Number(index as f64),
        ) {
            return Err(iterator_close_error(cx, &record, error));
        }
        return Ok(Value::Object(result));
    }

    let len = length_of_array_like(cx, object)?;
    let result = create_array_from_target(cx, this, Some(len))?;
    for index in 0..len {
        let value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        let mapped = map_array_from_value(
            cx,
            mapper.clone(),
            has_mapper,
            this_arg.clone(),
            value,
            index as usize,
        )?;
        CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(index as u64), mapped)?;
    }
    set_property_or_throw(
        cx,
        result,
        PropertyKey::from("length"),
        Value::Number(len as f64),
    )?;
    Ok(Value::Object(result))
}

fn map_array_from_value(
    cx: &mut Context,
    mapper: Value,
    has_mapper: bool,
    this_arg: Value,
    value: Value,
    index: usize,
) -> Completion<Value> {
    if has_mapper {
        cx.call_mut(mapper, this_arg, &[value, Value::Number(index as f64)])
    } else {
        Ok(value)
    }
}

fn array_of(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    create_array_from_result(cx, this, args.to_vec())
}

fn create_array_from_result(
    cx: &mut Context,
    constructor: Value,
    values: Vec<Value>,
) -> Completion<Value> {
    let len = values.len();
    if !cx.is_constructor(&constructor)? {
        return create_array_like(cx, values);
    }
    let Value::Object(result) = cx.construct_mut(constructor, &[Value::Number(len as f64)])? else {
        return Err(JsError::type_error(
            "array result constructor returned non-object",
        ));
    };
    for (index, value) in values.into_iter().enumerate() {
        CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(index as u64), value)?;
    }
    if !result.set(
        cx,
        PropertyKey::from("length"),
        Value::Number(len as f64),
        Value::Object(result),
    )? {
        return Err(JsError::type_error("could not set array result length"));
    }
    Ok(Value::Object(result))
}

fn create_array_from_target(
    cx: &mut Context,
    constructor: Value,
    len: Option<u32>,
) -> Completion<ObjectRef> {
    if !cx.is_constructor(&constructor)? {
        return create_empty_array_with_length(cx, len.unwrap_or(0));
    }

    let args = len
        .map(|len| vec![Value::Number(len as f64)])
        .unwrap_or_default();
    let Value::Object(result) = cx.construct_mut(constructor, &args)? else {
        return Err(JsError::type_error(
            "array result constructor returned non-object",
        ));
    };
    Ok(result)
}

fn array_proto_values(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    create_indexed_iterator(cx, this, "value")
}

fn array_proto_keys(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    create_indexed_iterator(cx, this, "key")
}

fn array_proto_entries(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    create_indexed_iterator(cx, this, "entry")
}

fn string_iterator(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let value = primitive_this_value(cx, this)?;
    let Value::String(_) = value else {
        return Err(JsError::type_error(
            "String iterator receiver is not string",
        ));
    };
    create_indexed_iterator(cx, value, "string")
}

fn create_collection_object(
    cx: &mut Context,
    proto_id: IntrinsicId,
    kind: CollectionKind,
) -> Completion<ObjectRef> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(proto_id)
        .ok_or_else(|| JsError::internal("missing collection prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    let slot = match kind {
        CollectionKind::Map => InternalSlot::MapData {
            entries: Vec::new(),
        },
        CollectionKind::Set => InternalSlot::SetData {
            entries: Vec::new(),
        },
    };
    cx.heap_mut().get_mut(object)?.add_slot(slot);
    Ok(object)
}

fn create_weak_collection_object(
    cx: &mut Context,
    proto_id: IntrinsicId,
    is_map: bool,
) -> Completion<ObjectRef> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(proto_id)
        .ok_or_else(|| JsError::internal("missing weak collection prototype intrinsic"))?;
    let object = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    let slot = if is_map {
        InternalSlot::WeakMapData {
            entries: Vec::new(),
        }
    } else {
        InternalSlot::WeakSetData {
            entries: Vec::new(),
        }
    };
    cx.heap_mut().get_mut(object)?.add_slot(slot);
    Ok(object)
}

fn can_be_held_weakly(value: &Value) -> bool {
    matches!(value, Value::Object(_) | Value::Symbol(_))
}

fn call_map_constructor_adder(
    cx: &mut Context,
    map: ObjectRef,
    adder: Value,
    entry: Value,
) -> Completion<()> {
    let Value::Object(entry_object) = entry else {
        return Err(JsError::type_error("Map constructor entry is not object"));
    };
    let key = entry_object.get(
        cx,
        &PropertyKey::array_index(0),
        Value::Object(entry_object),
    )?;
    let value = entry_object.get(
        cx,
        &PropertyKey::array_index(1),
        Value::Object(entry_object),
    )?;
    cx.call_mut(adder, Value::Object(map), &[key, value])?;
    Ok(())
}

fn weak_collection_entries_snapshot(
    cx: &Context,
    object: ObjectRef,
    is_map: bool,
) -> Completion<Vec<CollectionEntry>> {
    let object_data = cx.heap().get(object)?;
    for slot in &object_data.internal_slots {
        match (is_map, slot) {
            (true, InternalSlot::WeakMapData { entries })
            | (false, InternalSlot::WeakSetData { entries }) => return Ok(entries.clone()),
            _ => {}
        }
    }
    Err(JsError::type_error(
        "receiver does not have required weak collection internal slot",
    ))
}

fn with_weak_collection_entries_mut<R>(
    cx: &mut Context,
    object: ObjectRef,
    is_map: bool,
    body: impl FnOnce(&mut Vec<CollectionEntry>) -> R,
) -> Completion<R> {
    let object_data = cx.heap_mut().get_mut(object)?;
    for slot in &mut object_data.internal_slots {
        match (is_map, slot) {
            (true, InternalSlot::WeakMapData { entries })
            | (false, InternalSlot::WeakSetData { entries }) => return Ok(body(entries)),
            _ => {}
        }
    }
    Err(JsError::type_error(
        "receiver does not have required weak collection internal slot",
    ))
}

fn require_weak_collection_receiver(
    cx: &mut Context,
    this: Value,
    is_map: bool,
) -> Completion<ObjectRef> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error(
            "weak collection receiver is not an object",
        ));
    };
    weak_collection_entries_snapshot(cx, object, is_map)?;
    Ok(object)
}

fn weak_map_proto_get(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let weak_map = require_weak_collection_receiver(cx, this, true)?;
    let key = args.first().cloned().unwrap_or(Value::Undefined);
    if !can_be_held_weakly(&key) {
        return Ok(Value::Undefined);
    }
    for entry in weak_collection_entries_snapshot(cx, weak_map, true)? {
        if SameValueZero(&entry.key, &key) {
            return Ok(entry.value);
        }
    }
    Ok(Value::Undefined)
}

fn weak_map_proto_set(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let weak_map = require_weak_collection_receiver(cx, this.clone(), true)?;
    let key = args.first().cloned().unwrap_or(Value::Undefined);
    if !can_be_held_weakly(&key) {
        return Err(JsError::type_error("WeakMap key must be object or symbol"));
    }
    let value = args.get(1).cloned().unwrap_or(Value::Undefined);
    with_weak_collection_entries_mut(cx, weak_map, true, |entries| {
        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| SameValueZero(&entry.key, &key))
        {
            entry.value = value;
        } else {
            entries.push(CollectionEntry { key, value });
        }
    })?;
    Ok(this)
}

fn weak_map_proto_has(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let weak_map = require_weak_collection_receiver(cx, this, true)?;
    let key = args.first().cloned().unwrap_or(Value::Undefined);
    if !can_be_held_weakly(&key) {
        return Ok(Value::Boolean(false));
    }
    Ok(Value::Boolean(
        weak_collection_entries_snapshot(cx, weak_map, true)?
            .iter()
            .any(|entry| SameValueZero(&entry.key, &key)),
    ))
}

fn weak_map_proto_delete(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let weak_map = require_weak_collection_receiver(cx, this, true)?;
    let key = args.first().cloned().unwrap_or(Value::Undefined);
    if !can_be_held_weakly(&key) {
        return Ok(Value::Boolean(false));
    }
    let deleted = with_weak_collection_entries_mut(cx, weak_map, true, |entries| {
        let before = entries.len();
        entries.retain(|entry| !SameValueZero(&entry.key, &key));
        before != entries.len()
    })?;
    Ok(Value::Boolean(deleted))
}

fn weak_set_proto_add(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let weak_set = require_weak_collection_receiver(cx, this.clone(), false)?;
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    if !can_be_held_weakly(&value) {
        return Err(JsError::type_error(
            "WeakSet value must be object or symbol",
        ));
    }
    with_weak_collection_entries_mut(cx, weak_set, false, |entries| {
        if !entries
            .iter()
            .any(|entry| SameValueZero(&entry.key, &value))
        {
            entries.push(CollectionEntry {
                key: value.clone(),
                value,
            });
        }
    })?;
    Ok(this)
}

fn weak_set_proto_has(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let weak_set = require_weak_collection_receiver(cx, this, false)?;
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    if !can_be_held_weakly(&value) {
        return Ok(Value::Boolean(false));
    }
    Ok(Value::Boolean(
        weak_collection_entries_snapshot(cx, weak_set, false)?
            .iter()
            .any(|entry| SameValueZero(&entry.key, &value)),
    ))
}

fn weak_set_proto_delete(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let weak_set = require_weak_collection_receiver(cx, this, false)?;
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    if !can_be_held_weakly(&value) {
        return Ok(Value::Boolean(false));
    }
    let deleted = with_weak_collection_entries_mut(cx, weak_set, false, |entries| {
        let before = entries.len();
        entries.retain(|entry| !SameValueZero(&entry.key, &value));
        before != entries.len()
    })?;
    Ok(Value::Boolean(deleted))
}

fn normalize_collection_key(key: Value) -> Value {
    match key {
        Value::Number(value) if value == 0.0 => Value::Number(0.0),
        other => other,
    }
}

fn collection_entries_snapshot(
    cx: &Context,
    object: ObjectRef,
    kind: CollectionKind,
) -> Completion<Vec<CollectionEntry>> {
    let object_data = cx.heap().get(object)?;
    for slot in &object_data.internal_slots {
        match (kind.clone(), slot) {
            (CollectionKind::Map, InternalSlot::MapData { entries })
            | (CollectionKind::Set, InternalSlot::SetData { entries }) => {
                return Ok(entries.clone())
            }
            _ => {}
        }
    }
    Err(JsError::type_error(
        "receiver does not have required collection internal slot",
    ))
}

fn with_collection_entries_mut<R>(
    cx: &mut Context,
    object: ObjectRef,
    kind: CollectionKind,
    body: impl FnOnce(&mut Vec<CollectionEntry>) -> R,
) -> Completion<R> {
    let object_data = cx.heap_mut().get_mut(object)?;
    for slot in &mut object_data.internal_slots {
        match (kind.clone(), slot) {
            (CollectionKind::Map, InternalSlot::MapData { entries })
            | (CollectionKind::Set, InternalSlot::SetData { entries }) => return Ok(body(entries)),
            _ => {}
        }
    }
    Err(JsError::type_error(
        "receiver does not have required collection internal slot",
    ))
}

fn require_collection_receiver(
    cx: &mut Context,
    this: Value,
    kind: CollectionKind,
) -> Completion<ObjectRef> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error("collection receiver is not an object"));
    };
    collection_entries_snapshot(cx, object, kind)?;
    Ok(object)
}

fn set_map_entry(cx: &mut Context, map: ObjectRef, key: Value, value: Value) {
    let key = normalize_collection_key(key);
    let _ = with_collection_entries_mut(cx, map, CollectionKind::Map, |entries| {
        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| SameValueZero(&entry.key, &key))
        {
            entry.value = value;
        } else {
            entries.push(CollectionEntry { key, value });
        }
    });
}

fn add_set_value(cx: &mut Context, set: ObjectRef, value: Value) {
    let value = normalize_collection_key(value);
    let _ = with_collection_entries_mut(cx, set, CollectionKind::Set, |entries| {
        if !entries
            .iter()
            .any(|entry| SameValueZero(&entry.key, &value))
        {
            entries.push(CollectionEntry {
                key: value.clone(),
                value,
            });
        }
    });
}

fn map_proto_size(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    Ok(Value::Number(
        collection_entries_snapshot(cx, map, CollectionKind::Map)?.len() as f64,
    ))
}

fn set_proto_size(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this, CollectionKind::Set)?;
    Ok(Value::Number(
        collection_entries_snapshot(cx, set, CollectionKind::Set)?.len() as f64,
    ))
}

fn map_proto_get(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    let key = normalize_collection_key(args.first().cloned().unwrap_or(Value::Undefined));
    for entry in collection_entries_snapshot(cx, map, CollectionKind::Map)? {
        if SameValueZero(&entry.key, &key) {
            return Ok(entry.value);
        }
    }
    Ok(Value::Undefined)
}

fn map_proto_set(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this.clone(), CollectionKind::Map)?;
    let key = args.first().cloned().unwrap_or(Value::Undefined);
    let value = args.get(1).cloned().unwrap_or(Value::Undefined);
    set_map_entry(cx, map, key, value);
    Ok(this)
}

fn map_proto_has(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    let key = normalize_collection_key(args.first().cloned().unwrap_or(Value::Undefined));
    Ok(Value::Boolean(
        collection_entries_snapshot(cx, map, CollectionKind::Map)?
            .iter()
            .any(|entry| SameValueZero(&entry.key, &key)),
    ))
}

fn map_proto_delete(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    let key = normalize_collection_key(args.first().cloned().unwrap_or(Value::Undefined));
    let deleted = with_collection_entries_mut(cx, map, CollectionKind::Map, |entries| {
        let before = entries.len();
        entries.retain(|entry| !SameValueZero(&entry.key, &key));
        before != entries.len()
    })?;
    Ok(Value::Boolean(deleted))
}

fn map_proto_clear(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    with_collection_entries_mut(cx, map, CollectionKind::Map, |entries| entries.clear())?;
    Ok(Value::Undefined)
}

fn map_proto_keys(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    create_collection_iterator(
        cx,
        Value::Object(map),
        CollectionKind::Map,
        CollectionIteratorKind::Key,
    )
}

fn map_proto_values(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    create_collection_iterator(
        cx,
        Value::Object(map),
        CollectionKind::Map,
        CollectionIteratorKind::Value,
    )
}

fn map_proto_entries(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this, CollectionKind::Map)?;
    create_collection_iterator(
        cx,
        Value::Object(map),
        CollectionKind::Map,
        CollectionIteratorKind::KeyAndValue,
    )
}

fn map_proto_for_each(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let map = require_collection_receiver(cx, this.clone(), CollectionKind::Map)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Map.prototype.forEach callback is not callable",
        ));
    }
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    for entry in collection_entries_snapshot(cx, map, CollectionKind::Map)? {
        cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[entry.value, entry.key, this.clone()],
        )?;
    }
    Ok(Value::Undefined)
}

fn set_proto_add(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this.clone(), CollectionKind::Set)?;
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    add_set_value(cx, set, value);
    Ok(this)
}

fn set_proto_has(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this, CollectionKind::Set)?;
    let value = normalize_collection_key(args.first().cloned().unwrap_or(Value::Undefined));
    Ok(Value::Boolean(
        collection_entries_snapshot(cx, set, CollectionKind::Set)?
            .iter()
            .any(|entry| SameValueZero(&entry.key, &value)),
    ))
}

fn set_proto_delete(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this, CollectionKind::Set)?;
    let value = normalize_collection_key(args.first().cloned().unwrap_or(Value::Undefined));
    let deleted = with_collection_entries_mut(cx, set, CollectionKind::Set, |entries| {
        let before = entries.len();
        entries.retain(|entry| !SameValueZero(&entry.key, &value));
        before != entries.len()
    })?;
    Ok(Value::Boolean(deleted))
}

fn set_proto_clear(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this, CollectionKind::Set)?;
    with_collection_entries_mut(cx, set, CollectionKind::Set, |entries| entries.clear())?;
    Ok(Value::Undefined)
}

fn set_proto_values(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this, CollectionKind::Set)?;
    create_collection_iterator(
        cx,
        Value::Object(set),
        CollectionKind::Set,
        CollectionIteratorKind::Value,
    )
}

fn set_proto_entries(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this, CollectionKind::Set)?;
    create_collection_iterator(
        cx,
        Value::Object(set),
        CollectionKind::Set,
        CollectionIteratorKind::KeyAndValue,
    )
}

fn set_proto_for_each(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let set = require_collection_receiver(cx, this.clone(), CollectionKind::Set)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Set.prototype.forEach callback is not callable",
        ));
    }
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    for entry in collection_entries_snapshot(cx, set, CollectionKind::Set)? {
        cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[entry.value.clone(), entry.value, this.clone()],
        )?;
    }
    Ok(Value::Undefined)
}

fn create_collection_iterator(
    cx: &mut Context,
    target: Value,
    collection_kind: CollectionKind,
    iteration_kind: CollectionIteratorKind,
) -> Completion<Value> {
    let proto_id = match collection_kind {
        CollectionKind::Map => IntrinsicId::MapIteratorPrototype,
        CollectionKind::Set => IntrinsicId::SetIteratorPrototype,
    };
    let proto = cx
        .realm()?
        .intrinsics
        .get(proto_id)
        .ok_or_else(|| JsError::internal("missing collection iterator prototype intrinsic"))?;
    let iterator = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    cx.heap_mut()
        .get_mut(iterator)?
        .add_slot(InternalSlot::CollectionIteratorData {
            target,
            collection_kind,
            iteration_kind,
            index: 0,
        });
    Ok(Value::Object(iterator))
}

fn collection_iterator_next(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let Value::Object(iterator) = this else {
        return Err(JsError::type_error(
            "collection iterator receiver is not object",
        ));
    };
    let (target, collection_kind, iteration_kind, index) = {
        let object = cx.heap().get(iterator)?;
        let Some(InternalSlot::CollectionIteratorData {
            target,
            collection_kind,
            iteration_kind,
            index,
        }) = object
            .internal_slots
            .iter()
            .find(|slot| matches!(slot, InternalSlot::CollectionIteratorData { .. }))
        else {
            return Err(JsError::type_error(
                "collection iterator receiver is missing state",
            ));
        };
        (
            target.clone(),
            collection_kind.clone(),
            iteration_kind.clone(),
            *index,
        )
    };
    let Value::Object(target_object) = target.clone() else {
        return Err(JsError::type_error(
            "collection iterator target is not object",
        ));
    };
    let entries = collection_entries_snapshot(cx, target_object, collection_kind.clone())?;
    if index as usize >= entries.len() {
        return create_iterator_result(cx, Value::Undefined, true);
    }
    {
        let object = cx.heap_mut().get_mut(iterator)?;
        let Some(InternalSlot::CollectionIteratorData { index, .. }) = object
            .internal_slots
            .iter_mut()
            .find(|slot| matches!(slot, InternalSlot::CollectionIteratorData { .. }))
        else {
            return Err(JsError::type_error(
                "collection iterator receiver is missing state",
            ));
        };
        *index += 1;
    }
    let entry = entries[index as usize].clone();
    let value = match (collection_kind, iteration_kind) {
        (CollectionKind::Map, CollectionIteratorKind::Key) => entry.key,
        (CollectionKind::Map, CollectionIteratorKind::Value) => entry.value,
        (CollectionKind::Map, CollectionIteratorKind::KeyAndValue) => {
            create_array_like(cx, vec![entry.key, entry.value])?
        }
        (CollectionKind::Set, CollectionIteratorKind::Key)
        | (CollectionKind::Set, CollectionIteratorKind::Value) => entry.value,
        (CollectionKind::Set, CollectionIteratorKind::KeyAndValue) => {
            create_array_like(cx, vec![entry.value.clone(), entry.value])?
        }
    };
    create_iterator_result(cx, value, false)
}

fn create_indexed_iterator(cx: &mut Context, target: Value, kind: &str) -> Completion<Value> {
    if !matches!(kind, "string") {
        ArgView::new(target.clone()).to_object(cx)?;
    }
    let intrinsic = if matches!(kind, "string") {
        IntrinsicId::StringIteratorPrototype
    } else {
        IntrinsicId::ArrayIteratorPrototype
    };
    let proto = cx
        .realm()?
        .intrinsics
        .get(intrinsic)
        .ok_or_else(|| JsError::internal("missing indexed iterator prototype intrinsic"))?;
    let iterator = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    cx.heap_mut()
        .get_mut(iterator)?
        .add_slot(InternalSlot::IteratorData {
            target,
            kind: kind.to_owned(),
            index: 0,
        });
    Ok(Value::Object(iterator))
}

fn iterator_self(_cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(this)
}

fn iterator_from(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let Value::Object(object) = value.clone() else {
        let values = iterator_values(cx, value)?;
        return iterator_from_values(cx, values);
    };
    let next = object.get(cx, &PropertyKey::from("next"), Value::Object(object))?;
    if cx.is_callable(&next)? {
        return Ok(Value::Object(object));
    }
    let values = iterator_values(cx, Value::Object(object))?;
    iterator_from_values(cx, values)
}

fn iterator_concat(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let mut iterable_objects = Vec::new();
    for item in args {
        let Value::Object(object) = item.clone() else {
            return Err(JsError::type_error("Iterator.concat item is not object"));
        };
        let method = object.get(cx, &PropertyKey::Symbol(SYMBOL_ITERATOR_ID), item.clone())?;
        if !cx.is_callable(&method)? {
            return Err(JsError::type_error("Iterator.concat item is not iterable"));
        }
        iterable_objects.push(item.clone());
    }

    create_concat_iterator(cx, iterable_objects)
}

fn iterator_proto_to_array(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let values = iterator_values(cx, this)?;
    create_array_like(cx, values)
}

fn iterator_proto_map(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback =
        require_iterator_callback(cx, args.first().cloned().unwrap_or(Value::Undefined), "map")?;
    let mut out = Vec::new();
    for (index, value) in iterator_values(cx, this)?.into_iter().enumerate() {
        out.push(cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value, Value::Number(index as f64)],
        )?);
    }
    iterator_from_values(cx, out)
}

fn iterator_proto_filter(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback = require_iterator_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "filter",
    )?;
    let mut out = Vec::new();
    for (index, value) in iterator_values(cx, this)?.into_iter().enumerate() {
        let keep = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value.clone(), Value::Number(index as f64)],
        )?;
        if truthy(keep) {
            out.push(value);
        }
    }
    iterator_from_values(cx, out)
}

fn iterator_proto_flat_map(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback = require_iterator_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "flatMap",
    )?;
    let mut out = Vec::new();
    for (index, value) in iterator_values(cx, this)?.into_iter().enumerate() {
        let mapped = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value, Value::Number(index as f64)],
        )?;
        out.extend(iterator_values(cx, mapped)?);
    }
    iterator_from_values(cx, out)
}

fn iterator_proto_take(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let limit = iterator_limit(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let out = iterator_values(cx, this)?
        .into_iter()
        .take(limit)
        .collect::<Vec<_>>();
    iterator_from_values(cx, out)
}

fn iterator_proto_drop(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let limit = iterator_limit(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let out = iterator_values(cx, this)?
        .into_iter()
        .skip(limit)
        .collect::<Vec<_>>();
    iterator_from_values(cx, out)
}

fn iterator_proto_every(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback = require_iterator_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "every",
    )?;
    for (index, value) in iterator_values(cx, this)?.into_iter().enumerate() {
        let result = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value, Value::Number(index as f64)],
        )?;
        if !truthy(result) {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

fn iterator_proto_some(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback = require_iterator_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "some",
    )?;
    for (index, value) in iterator_values(cx, this)?.into_iter().enumerate() {
        let result = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value, Value::Number(index as f64)],
        )?;
        if truthy(result) {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

fn iterator_proto_find(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback = require_iterator_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "find",
    )?;
    for (index, value) in iterator_values(cx, this)?.into_iter().enumerate() {
        let result = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value.clone(), Value::Number(index as f64)],
        )?;
        if truthy(result) {
            return Ok(value);
        }
    }
    Ok(Value::Undefined)
}

fn iterator_proto_for_each(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback = require_iterator_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "forEach",
    )?;
    for (index, value) in iterator_values(cx, this)?.into_iter().enumerate() {
        cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[value, Value::Number(index as f64)],
        )?;
    }
    Ok(Value::Undefined)
}

fn iterator_proto_reduce(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let callback = require_iterator_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "reduce",
    )?;
    let values = iterator_values(cx, this)?;
    let mut iter = values.into_iter();
    let (mut accumulator, mut index) = if let Some(initial) = args.get(1).cloned() {
        (initial, 0_usize)
    } else {
        let Some(first) = iter.next() else {
            return Err(JsError::type_error(
                "Iterator.prototype.reduce of empty iterator with no initial value",
            ));
        };
        (first, 1_usize)
    };
    for value in iter {
        accumulator = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[accumulator, value, Value::Number(index as f64)],
        )?;
        index += 1;
    }
    Ok(accumulator)
}

fn iterator_values(cx: &mut Context, value: Value) -> Completion<Vec<Value>> {
    let record = get_iterator_or_iterator_like(cx, value)?;
    let mut out = Vec::new();
    while let Some(value) = iterator_step_value(cx, &record)? {
        out.push(value);
    }
    Ok(out)
}

fn get_iterator_or_iterator_like(cx: &mut Context, value: Value) -> Completion<IteratorRecord> {
    if let Value::Object(object) = value.clone() {
        let next_method = object.get(cx, &PropertyKey::from("next"), Value::Object(object))?;
        if cx.is_callable(&next_method)? {
            return Ok(IteratorRecord {
                iterator: value,
                next_method,
            });
        }
    }
    get_iterator(cx, value)
}

fn iterator_from_values(cx: &mut Context, values: Vec<Value>) -> Completion<Value> {
    let array = create_array_like_value(cx, values)?;
    create_indexed_iterator(cx, Value::Object(array), "value")
}

fn create_concat_iterator(cx: &mut Context, iterables: Vec<Value>) -> Completion<Value> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::IteratorPrototype)
        .ok_or_else(|| JsError::internal("missing Iterator.prototype intrinsic"))?;
    let function_proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::FunctionPrototype)
        .ok_or_else(|| JsError::internal("missing Function.prototype intrinsic"))?;
    let iterator = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    cx.heap_mut()
        .get_mut(iterator)?
        .add_slot(InternalSlot::ConcatIteratorData {
            iterables,
            outer_index: 0,
            active_iterator: None,
            active_next_method: None,
        });
    let next = cx.heap_mut().allocate(JsObject::function(
        Some(function_proto),
        FunctionData::builtin("next", 0, concat_iterator_next),
    ));
    define_data_with_attrs(
        cx.heap_mut(),
        next,
        "length",
        Value::Number(0.0),
        false,
        false,
        true,
    );
    define_data_with_attrs(
        cx.heap_mut(),
        next,
        "name",
        Value::String("next".to_owned()),
        false,
        false,
        true,
    );
    define_data_with_attrs(
        cx.heap_mut(),
        iterator,
        "next",
        Value::Object(next),
        true,
        false,
        true,
    );
    Ok(Value::Object(iterator))
}

fn concat_iterator_next(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let Value::Object(iterator) = this else {
        return Err(JsError::type_error(
            "concat iterator receiver is not object",
        ));
    };
    loop {
        let (iterables, outer_index, active_iterator, active_next_method) = {
            let object = cx.heap().get(iterator)?;
            let Some(InternalSlot::ConcatIteratorData {
                iterables,
                outer_index,
                active_iterator,
                active_next_method,
            }) = object
                .internal_slots
                .iter()
                .find(|slot| matches!(slot, InternalSlot::ConcatIteratorData { .. }))
            else {
                return Err(JsError::type_error(
                    "concat iterator receiver is missing state",
                ));
            };
            (
                iterables.clone(),
                *outer_index,
                active_iterator.clone(),
                active_next_method.clone(),
            )
        };
        if outer_index >= iterables.len() {
            return create_iterator_result(cx, Value::Undefined, true);
        }
        let record =
            if let (Some(iterator), Some(next_method)) = (active_iterator, active_next_method) {
                IteratorRecord {
                    iterator,
                    next_method,
                }
            } else {
                let record = get_iterator(cx, iterables[outer_index].clone())?;
                update_concat_iterator_state(cx, iterator, |slot| {
                    if let InternalSlot::ConcatIteratorData {
                        active_iterator,
                        active_next_method,
                        ..
                    } = slot
                    {
                        *active_iterator = Some(record.iterator.clone());
                        *active_next_method = Some(record.next_method.clone());
                    }
                })?;
                record
            };
        if let Some(value) = iterator_step_value(cx, &record)? {
            return create_iterator_result(cx, value, false);
        }
        update_concat_iterator_state(cx, iterator, |slot| {
            if let InternalSlot::ConcatIteratorData {
                outer_index,
                active_iterator,
                active_next_method,
                ..
            } = slot
            {
                *outer_index += 1;
                *active_iterator = None;
                *active_next_method = None;
            }
        })?;
    }
}

fn update_concat_iterator_state(
    cx: &mut Context,
    iterator: ObjectRef,
    update: impl FnOnce(&mut InternalSlot),
) -> Completion<()> {
    let object = cx.heap_mut().get_mut(iterator)?;
    let Some(slot) = object
        .internal_slots
        .iter_mut()
        .find(|slot| matches!(slot, InternalSlot::ConcatIteratorData { .. }))
    else {
        return Err(JsError::type_error(
            "concat iterator receiver is missing state",
        ));
    };
    update(slot);
    Ok(())
}

fn require_iterator_callback(
    cx: &Context,
    callback: Value,
    method_name: &str,
) -> Completion<Value> {
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(format!(
            "Iterator.prototype.{method_name} callback is not callable"
        )));
    }
    Ok(callback)
}

fn iterator_limit(cx: &mut Context, value: Value) -> Completion<usize> {
    let number = to_number_value(cx, value)?;
    if number.is_nan() || number <= 0.0 {
        return Ok(0);
    }
    if !number.is_finite() {
        return Ok(usize::MAX);
    }
    Ok(number.trunc() as usize)
}

fn indexed_iterator_next(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let Value::Object(iterator) = this else {
        return Err(JsError::type_error("iterator next receiver is not object"));
    };
    let (target, kind, index) = {
        let object = cx.heap().get(iterator)?;
        let Some(InternalSlot::IteratorData {
            target,
            kind,
            index,
        }) = object
            .internal_slots
            .iter()
            .find(|slot| matches!(slot, InternalSlot::IteratorData { .. }))
        else {
            return Err(JsError::type_error("iterator receiver is missing state"));
        };
        (target.clone(), kind.clone(), *index)
    };

    let len = iterator_target_length(cx, target.clone(), &kind)?;
    if index >= len {
        return create_iterator_result(cx, Value::Undefined, true);
    }

    {
        let object = cx.heap_mut().get_mut(iterator)?;
        let Some(InternalSlot::IteratorData { index, .. }) = object
            .internal_slots
            .iter_mut()
            .find(|slot| matches!(slot, InternalSlot::IteratorData { .. }))
        else {
            return Err(JsError::type_error("iterator receiver is missing state"));
        };
        *index += 1;
    }

    let value = match kind.as_str() {
        "key" => Value::Number(index as f64),
        "value" => {
            let object = ArgView::new(target.clone()).to_object(cx)?;
            object.get(
                cx,
                &PropertyKey::array_index(index as u64),
                Value::Object(object),
            )?
        }
        "entry" => {
            let object = ArgView::new(target.clone()).to_object(cx)?;
            let value = object.get(
                cx,
                &PropertyKey::array_index(index as u64),
                Value::Object(object),
            )?;
            create_array_like(cx, vec![Value::Number(index as f64), value])?
        }
        "string" => {
            let Value::String(text) = target else {
                return Err(JsError::type_error("string iterator target is not string"));
            };
            Value::String(
                text.chars()
                    .nth(index as usize)
                    .map(|ch| ch.to_string())
                    .unwrap_or_default(),
            )
        }
        _ => return Err(JsError::type_error("unknown iterator kind")),
    };
    create_iterator_result(cx, value, false)
}

fn iterator_target_length(cx: &mut Context, target: Value, kind: &str) -> Completion<u32> {
    if kind == "string" {
        let Value::String(text) = target else {
            return Err(JsError::type_error("string iterator target is not string"));
        };
        return Ok(text.chars().count() as u32);
    }
    let object = ArgView::new(target).to_object(cx)?;
    length_of_array_like(cx, object)
}

fn create_iterator_result(cx: &mut Context, value: Value, done: bool) -> Completion<Value> {
    let proto = cx
        .realm()?
        .intrinsics
        .get(IntrinsicId::ObjectPrototype)
        .ok_or_else(|| JsError::internal("missing Object.prototype intrinsic"))?;
    let result = cx.heap_mut().allocate(JsObject::ordinary(Some(proto)));
    result.define_own_property_or_throw(
        cx,
        PropertyKey::from("value"),
        Descriptor::data(value, true, true, true),
    )?;
    result.define_own_property_or_throw(
        cx,
        PropertyKey::from("done"),
        Descriptor::data(Value::Boolean(done), true, true, true),
    )?;
    Ok(Value::Object(result))
}

fn array_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this.clone())?;
    let join = object.get(cx, &PropertyKey::from("join"), Value::Object(object))?;
    if cx.is_callable(&join)? {
        cx.call_mut(join, this, &[])
    } else {
        object_proto_to_string(cx, Value::Object(object), &[])
    }
}

fn array_proto_join(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like(cx, object)?;
    let separator = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::Undefined => ",".to_owned(),
        value => to_string_value(cx, value)?,
    };
    let mut parts = Vec::new();
    for index in 0..len {
        let value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        let text = match value {
            Value::Undefined | Value::Null => String::new(),
            value => to_string_value(cx, value)?,
        };
        parts.push(text);
    }
    Ok(Value::String(parts.join(&separator)))
}

fn array_proto_to_locale_string(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like(cx, object)?;
    let mut parts = Vec::new();
    for index in 0..len {
        let value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        let text = match value {
            Value::Undefined | Value::Null => String::new(),
            value => {
                let element = ArgView::new(value.clone()).to_object(cx)?;
                let method =
                    element.get(cx, &PropertyKey::from("toLocaleString"), value.clone())?;
                if !cx.is_callable(&method)? {
                    return Err(JsError::type_error(
                        "element toLocaleString is not callable",
                    ));
                }
                let localized = cx.call_mut(method, value, &[])?;
                to_string_value(cx, localized)?
            }
        };
        parts.push(text);
    }
    Ok(Value::String(parts.join(",")))
}

fn array_proto_at(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like(cx, object)?;
    let index = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let integer = if index.is_nan() { 0.0 } else { index.trunc() };
    let relative = if integer < 0.0 {
        len as i64 + integer as i64
    } else {
        integer as i64
    };
    if relative < 0 || relative >= len as i64 {
        return Ok(Value::Undefined);
    }
    object.get(
        cx,
        &PropertyKey::array_index(relative as u64),
        Value::Object(object),
    )
}

fn array_proto_push(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let mut len = length_of_array_like_f64(cx, object)?;
    if len + args.len() as f64 > 9_007_199_254_740_991.0 {
        return Err(JsError::type_error("array length overflow"));
    }
    for arg in args {
        set_property_or_throw(
            cx,
            object,
            PropertyKey::array_index(len as u64),
            arg.clone(),
        )?;
        len += 1.0;
    }
    set_property_or_throw(cx, object, PropertyKey::from("length"), Value::Number(len))?;
    Ok(Value::Number(len))
}

fn array_proto_pop(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    if len == 0.0 {
        set_property_or_throw(cx, object, PropertyKey::from("length"), Value::Number(0.0))?;
        return Ok(Value::Undefined);
    }
    let index = len - 1.0;
    let key = PropertyKey::array_index(index as u64);
    let value = object.get(cx, &key, Value::Object(object))?;
    delete_property_or_throw(cx, object, key)?;
    set_property_or_throw(
        cx,
        object,
        PropertyKey::from("length"),
        Value::Number(index),
    )?;
    Ok(value)
}

fn array_proto_shift(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like(cx, object)?;
    if len == 0 {
        set_property_or_throw(cx, object, PropertyKey::from("length"), Value::Number(0.0))?;
        return Ok(Value::Undefined);
    }
    let first = object.get(cx, &PropertyKey::array_index(0), Value::Object(object))?;
    for index in 1..len {
        let from = PropertyKey::array_index(index as u64);
        let to = PropertyKey::array_index((index - 1) as u64);
        if object.has_property(cx, &from)? {
            let value = object.get(cx, &from, Value::Object(object))?;
            set_property_or_throw(cx, object, to, value)?;
        } else {
            delete_property_or_throw(cx, object, to)?;
        }
    }
    delete_property_or_throw(cx, object, PropertyKey::array_index((len - 1) as u64))?;
    set_property_or_throw(
        cx,
        object,
        PropertyKey::from("length"),
        Value::Number((len - 1) as f64),
    )?;
    Ok(first)
}

fn array_proto_unshift(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    let count = args.len() as u64;
    const MAX_SAFE_LENGTH: f64 = 9_007_199_254_740_991.0;
    if len + count as f64 > MAX_SAFE_LENGTH {
        return Err(JsError::type_error(
            "array-like length exceeds safe integer",
        ));
    }
    if count > 0 {
        if len > 100_000.0 {
            array_unshift_sparse(cx, object, len as u64, count)?;
        } else {
            let mut index = len as u32;
            while index > 0 {
                index -= 1;
                let from = PropertyKey::array_index(index as u64);
                let to = PropertyKey::array_index(index as u64 + count);
                if object.has_property(cx, &from)? {
                    let value = object.get(cx, &from, Value::Object(object))?;
                    set_property_or_throw(cx, object, to, value)?;
                } else {
                    delete_property_or_throw(cx, object, to)?;
                }
            }
        }
        for (index, value) in args.iter().cloned().enumerate() {
            set_property_or_throw(cx, object, PropertyKey::array_index(index as u64), value)?;
        }
    }
    let new_len = len + count as f64;
    set_property_or_throw(
        cx,
        object,
        PropertyKey::from("length"),
        Value::Number(new_len),
    )?;
    Ok(Value::Number(new_len))
}

fn array_proto_reverse(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    if len > 100_000.0 {
        reverse_array_like_sparse(cx, object, len as u64)?;
    } else {
        reverse_array_like(cx, object, len as u32)?;
    }
    Ok(Value::Object(object))
}

fn array_proto_sort(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let comparator = args.first().cloned().unwrap_or(Value::Undefined);
    if !matches!(comparator, Value::Undefined) && !cx.is_callable(&comparator)? {
        return Err(JsError::type_error(
            "Array.prototype.sort comparator is not callable",
        ));
    }
    let len = length_of_array_like(cx, object)?;
    sort_array_like(cx, object, len, comparator)?;
    Ok(Value::Object(object))
}

fn array_proto_slice(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    let start = array_relative_index_f64(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        len,
        0,
    )?;
    let end = if let Some(value) = args
        .get(1)
        .filter(|value| !matches!(value, Value::Undefined))
    {
        array_relative_index_f64(cx, value.clone(), len, len as u64)?
    } else {
        len
    };
    let count = (end - start).max(0.0);
    if count > u32::MAX as f64 {
        return Err(JsError::range_error("invalid array length"));
    }
    let result = array_species_create(cx, object, count as u32)?;
    let mut index = start;
    let mut target = 0_u64;
    while index < end {
        let key = PropertyKey::array_index(index as u64);
        if object.has_property(cx, &key)? {
            let value = object.get(cx, &key, Value::Object(object))?;
            CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(target), value)?;
        }
        index += 1.0;
        target += 1;
    }
    Ok(Value::Object(result))
}

fn array_proto_to_reversed(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    reject_array_create_overflow(cx, object)?;
    let len = length_of_array_like(cx, object)?;
    let result = create_empty_array_with_length(cx, len)?;
    for index in 0..len {
        let from = len - 1 - index;
        let from_key = PropertyKey::array_index(from as u64);
        let value = object.get(cx, &from_key, Value::Object(object))?;
        CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(index as u64), value)?;
    }
    Ok(Value::Object(result))
}

fn array_proto_to_sorted(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    reject_array_create_overflow(cx, object)?;
    let comparator = args.first().cloned().unwrap_or(Value::Undefined);
    if !matches!(comparator, Value::Undefined) && !cx.is_callable(&comparator)? {
        return Err(JsError::type_error(
            "Array.prototype.toSorted comparator is not callable",
        ));
    }
    let len = length_of_array_like(cx, object)?;
    let result = create_empty_array_with_length(cx, len)?;
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        let value = object.get(cx, &key, Value::Object(object))?;
        CreateDataPropertyOrThrow(cx, result, key, value)?;
    }
    sort_array_like(cx, result, len, comparator)?;
    Ok(Value::Object(result))
}

fn array_proto_to_spliced(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    let start = array_relative_index_f64(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        len,
        0,
    )?;
    let delete_count = if args.len() < 2 {
        len - start
    } else {
        let raw = to_number_value(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
        if raw.is_nan() || raw <= 0.0 {
            0.0
        } else {
            raw.trunc().min(9_007_199_254_740_991.0).min(len - start)
        }
    };
    let insert_items = if args.len() > 2 { &args[2..] } else { &[] };
    let new_len = len - delete_count + insert_items.len() as f64;
    if !(0.0..=u32::MAX as f64).contains(&new_len) {
        return Err(JsError::range_error("invalid array length"));
    }
    let result = create_empty_array_with_length(cx, new_len as u32)?;
    let mut to = 0_u64;
    let mut index = 0.0;
    while index < start {
        let value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(to), value)?;
        to += 1;
        index += 1.0;
    }
    for value in insert_items {
        CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(to), value.clone())?;
        to += 1;
    }
    index = start + delete_count;
    while index < len {
        let value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(to), value)?;
        to += 1;
        index += 1.0;
    }
    Ok(Value::Object(result))
}

fn array_proto_with(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    reject_array_create_overflow(cx, object)?;
    let len = length_of_array_like(cx, object)?;
    let raw_index = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let integer = if raw_index.is_nan() {
        0.0
    } else {
        raw_index.trunc()
    };
    let actual = if integer < 0.0 {
        len as i64 + integer as i64
    } else {
        integer as i64
    };
    if actual < 0 || actual >= len as i64 {
        return Err(JsError::range_error("array index out of range"));
    }
    let replacement = args.get(1).cloned().unwrap_or(Value::Undefined);
    let result = create_empty_array_with_length(cx, len)?;
    for index in 0..len {
        let value = if index as i64 == actual {
            replacement.clone()
        } else {
            object.get(
                cx,
                &PropertyKey::array_index(index as u64),
                Value::Object(object),
            )?
        };
        CreateDataPropertyOrThrow(cx, result, PropertyKey::array_index(index as u64), value)?;
    }
    Ok(Value::Object(result))
}

fn array_proto_splice(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    const MAX_SAFE_LENGTH: f64 = 9_007_199_254_740_991.0;

    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    let start = array_relative_index_f64(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        len,
        0,
    )?;
    let delete_count = if args.len() < 2 {
        if args.is_empty() {
            0
        } else {
            (len - start).max(0.0) as u64
        }
    } else {
        let raw = to_number_value(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
        if raw.is_nan() || raw <= 0.0 {
            0
        } else {
            raw.trunc().min(MAX_SAFE_LENGTH).min(len - start) as u64
        }
    };
    let insert_items = if args.len() > 2 { &args[2..] } else { &[] };
    let insert_count = insert_items.len() as u64;
    let new_len = len - delete_count as f64 + insert_count as f64;
    if new_len > MAX_SAFE_LENGTH {
        return Err(JsError::type_error(
            "array-like length exceeds safe integer",
        ));
    }
    let start = start as u64;
    let len = len as u64;
    let deleted = create_splice_deleted_array(cx, object, start, delete_count)?;
    if insert_count < delete_count {
        array_splice_shift_left(cx, object, start, len, delete_count, insert_count)?;
    } else if insert_count > delete_count {
        array_splice_shift_right(cx, object, start, len, delete_count, insert_count)?;
    }
    for (offset, value) in insert_items.iter().cloned().enumerate() {
        set_property_or_throw(
            cx,
            object,
            PropertyKey::array_index(start + offset as u64),
            value,
        )?;
    }
    set_property_or_throw(
        cx,
        object,
        PropertyKey::from("length"),
        Value::Number(new_len),
    )?;
    Ok(Value::Object(deleted))
}

fn create_splice_deleted_array(
    cx: &mut Context,
    object: ObjectRef,
    start: u64,
    delete_count: u64,
) -> Completion<ObjectRef> {
    if delete_count > u32::MAX as u64 {
        return Err(JsError::range_error("invalid array length"));
    }
    let deleted = array_species_create(cx, object, delete_count as u32)?;
    for key_index in own_integer_indices(cx, object)? {
        if key_index >= start && key_index < start + delete_count {
            let from = PropertyKey::array_index(key_index);
            let value = object.get(cx, &from, Value::Object(object))?;
            CreateDataPropertyOrThrow(
                cx,
                deleted,
                PropertyKey::array_index(key_index - start),
                value,
            )?;
        }
    }
    Ok(deleted)
}

fn array_splice_shift_left(
    cx: &mut Context,
    object: ObjectRef,
    start: u64,
    len: u64,
    delete_count: u64,
    insert_count: u64,
) -> Completion<()> {
    let shift = delete_count - insert_count;
    let source_start = start + delete_count;
    let target_start = start + insert_count;
    let target_end = len - shift;
    let original_keys = own_integer_indices(cx, object)?;
    let original_set: HashSet<u64> = original_keys.iter().copied().collect();
    let mut moves = Vec::new();
    let mut deletes = Vec::new();

    for key_index in original_keys {
        if key_index >= source_start && key_index < len {
            let from = PropertyKey::array_index(key_index);
            let value = object.get(cx, &from, Value::Object(object))?;
            moves.push((key_index - shift, value));
            deletes.push(key_index);
        } else if key_index >= target_start
            && key_index < target_end
            && !original_set.contains(&(key_index + shift))
        {
            deletes.push(key_index);
        } else if key_index >= target_end && key_index < len {
            deletes.push(key_index);
        }
    }

    for (to, value) in moves {
        set_property_or_throw(cx, object, PropertyKey::array_index(to), value)?;
    }
    for key_index in deletes {
        delete_property_or_throw(cx, object, PropertyKey::array_index(key_index))?;
    }
    Ok(())
}

fn array_splice_shift_right(
    cx: &mut Context,
    object: ObjectRef,
    start: u64,
    len: u64,
    delete_count: u64,
    insert_count: u64,
) -> Completion<()> {
    let shift = insert_count - delete_count;
    let source_start = start + delete_count;
    let original_keys = own_integer_indices(cx, object)?;
    let original_set: HashSet<u64> = original_keys.iter().copied().collect();
    let mut moves = Vec::new();
    let mut deletes = Vec::new();

    for key_index in original_keys {
        if key_index >= source_start && key_index < len {
            let from = PropertyKey::array_index(key_index);
            let value = object.get(cx, &from, Value::Object(object))?;
            moves.push((key_index, key_index + shift, value));
        } else if key_index >= source_start + shift
            && key_index < len + shift
            && !original_set.contains(&(key_index - shift))
        {
            deletes.push(key_index);
        }
    }

    moves.sort_by(|left, right| right.0.cmp(&left.0));
    for (from, to, value) in moves {
        set_property_or_throw(cx, object, PropertyKey::array_index(to), value)?;
        if from != to {
            delete_property_or_throw(cx, object, PropertyKey::array_index(from))?;
        }
    }
    for key_index in deletes {
        delete_property_or_throw(cx, object, PropertyKey::array_index(key_index))?;
    }
    Ok(())
}

fn own_integer_indices(cx: &mut Context, object: ObjectRef) -> Completion<Vec<u64>> {
    let mut keys = Vec::new();
    for key in object.own_property_keys(cx)? {
        if let Some(index) = property_key_integer_index(&key) {
            keys.push(index);
        }
    }
    keys.sort_unstable();
    keys.dedup();
    Ok(keys)
}

fn property_key_integer_index(key: &PropertyKey) -> Option<u64> {
    let PropertyKey::String(value) = key else {
        return None;
    };
    if value.is_empty() || (value.len() > 1 && value.starts_with('0')) {
        return None;
    }
    let index = value.parse::<u64>().ok()?;
    if index > 9_007_199_254_740_991 || index.to_string() != *value {
        return None;
    }
    Some(index)
}

fn set_property_or_throw(
    cx: &mut Context,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
) -> Completion<()> {
    if object.set(cx, key.clone(), value, Value::Object(object))? {
        Ok(())
    } else {
        Err(JsError::type_error(format!(
            "property {key} could not be set"
        )))
    }
}

fn create_empty_array_with_length(cx: &mut Context, len: u32) -> Completion<ObjectRef> {
    let result = create_array_like_value(cx, Vec::new())?;
    result.define_own_property_or_throw(
        cx,
        PropertyKey::from("length"),
        Descriptor::data(Value::Number(len as f64), true, false, false),
    )?;
    Ok(result)
}

fn reverse_array_like(cx: &mut Context, object: ObjectRef, len: u32) -> Completion<()> {
    let middle = len / 2;
    for lower in 0..middle {
        let upper = len - lower - 1;
        let lower_key = PropertyKey::array_index(lower as u64);
        let upper_key = PropertyKey::array_index(upper as u64);
        let lower_exists = object.has_property(cx, &lower_key)?;
        let lower_value = if lower_exists {
            Some(object.get(cx, &lower_key, Value::Object(object))?)
        } else {
            None
        };
        let upper_exists = object.has_property(cx, &upper_key)?;
        let upper_value = if upper_exists {
            Some(object.get(cx, &upper_key, Value::Object(object))?)
        } else {
            None
        };
        match (lower_value, upper_value) {
            (Some(lower_value), Some(upper_value)) => {
                set_property_or_throw(cx, object, lower_key, upper_value)?;
                set_property_or_throw(cx, object, upper_key, lower_value)?;
            }
            (Some(lower_value), None) => {
                delete_property_or_throw(cx, object, lower_key)?;
                set_property_or_throw(cx, object, upper_key, lower_value)?;
            }
            (None, Some(upper_value)) => {
                set_property_or_throw(cx, object, lower_key, upper_value)?;
                delete_property_or_throw(cx, object, upper_key)?;
            }
            (None, None) => {}
        }
    }
    Ok(())
}

fn reverse_array_like_sparse(cx: &mut Context, object: ObjectRef, len: u64) -> Completion<()> {
    let keys = own_integer_indices(cx, object)?;
    let mut handled = HashSet::new();
    for lower in keys {
        if lower >= len || !handled.insert(lower) {
            continue;
        }
        let upper = len - lower - 1;
        handled.insert(upper);
        let lower_key = PropertyKey::array_index(lower);
        let upper_key = PropertyKey::array_index(upper);
        let lower_exists = object.has_property(cx, &lower_key)?;
        let lower_value = if lower_exists {
            Some(object.get(cx, &lower_key, Value::Object(object))?)
        } else {
            None
        };
        let upper_exists = object.has_property(cx, &upper_key)?;
        let upper_value = if upper_exists {
            Some(object.get(cx, &upper_key, Value::Object(object))?)
        } else {
            None
        };
        match (lower_value, upper_value) {
            (Some(lower_value), Some(upper_value)) => {
                set_property_or_throw(cx, object, lower_key, upper_value)?;
                set_property_or_throw(cx, object, upper_key, lower_value)?;
            }
            (Some(lower_value), None) => {
                delete_property_or_throw(cx, object, lower_key)?;
                set_property_or_throw(cx, object, upper_key, lower_value)?;
            }
            (None, Some(upper_value)) => {
                set_property_or_throw(cx, object, lower_key, upper_value)?;
                delete_property_or_throw(cx, object, upper_key)?;
            }
            (None, None) => {}
        }
    }
    Ok(())
}

fn array_unshift_sparse(
    cx: &mut Context,
    object: ObjectRef,
    len: u64,
    count: u64,
) -> Completion<()> {
    let mut from_indices = own_integer_indices(cx, object)?;
    from_indices.retain(|index| *index < len);
    from_indices.sort_by(|left, right| right.cmp(left));
    let mut next = len;
    for from_index in &from_indices {
        for gap in (*from_index + 1..next).rev() {
            delete_property_or_throw(cx, object, PropertyKey::array_index(gap + count))?;
        }
        let to_index = *from_index + count;
        let from = PropertyKey::array_index(*from_index);
        let to = PropertyKey::array_index(to_index);
        let value = object.get(cx, &from, Value::Object(object))?;
        set_property_or_throw(cx, object, to, value)?;
        next = *from_index;
    }
    Ok(())
}

fn sort_array_like(
    cx: &mut Context,
    object: ObjectRef,
    len: u32,
    comparator: Value,
) -> Completion<()> {
    let mut values = Vec::new();
    let mut holes = 0_u32;
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        if object.has_property(cx, &key)? {
            values.push(object.get(cx, &key, Value::Object(object))?);
        } else {
            holes += 1;
        }
    }

    let mut sorted: Vec<Value> = Vec::new();
    for value in values {
        let mut insert_at = sorted.len();
        for (index, existing) in sorted.iter().enumerate() {
            if array_sort_compare(cx, value.clone(), existing.clone(), comparator.clone())? < 0.0 {
                insert_at = index;
                break;
            }
        }
        sorted.insert(insert_at, value);
    }

    for (index, value) in sorted.into_iter().enumerate() {
        set_property_or_throw(cx, object, PropertyKey::array_index(index as u64), value)?;
    }
    let present = len - holes;
    for index in present..len {
        delete_property_or_throw(cx, object, PropertyKey::array_index(index as u64))?;
    }
    Ok(())
}

fn array_sort_compare(
    cx: &mut Context,
    left: Value,
    right: Value,
    comparator: Value,
) -> Completion<f64> {
    if matches!(left, Value::Undefined) && matches!(right, Value::Undefined) {
        return Ok(0.0);
    }
    if matches!(left, Value::Undefined) {
        return Ok(1.0);
    }
    if matches!(right, Value::Undefined) {
        return Ok(-1.0);
    }
    if !matches!(comparator, Value::Undefined) {
        let result = cx.call_mut(comparator, Value::Undefined, &[left, right])?;
        let number = to_number_value(cx, result)?;
        return Ok(if number.is_nan() { 0.0 } else { number });
    }
    let left = to_string_value(cx, left)?;
    let right = to_string_value(cx, right)?;
    Ok(match left.cmp(&right) {
        std::cmp::Ordering::Less => -1.0,
        std::cmp::Ordering::Equal => 0.0,
        std::cmp::Ordering::Greater => 1.0,
    })
}

fn delete_property_or_throw(
    cx: &mut Context,
    object: ObjectRef,
    key: PropertyKey,
) -> Completion<()> {
    if object.delete(cx, &key)? {
        Ok(())
    } else {
        Err(JsError::type_error("property could not be deleted"))
    }
}

fn array_proto_flat(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    reject_array_create_overflow(cx, object)?;
    let depth = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::Undefined => 1,
        value => to_number_value(cx, value)?.max(0.0) as u32,
    };
    let target = array_species_create(cx, object, 0)?;
    flatten_array_like_into_target(cx, target, object, depth, 0).map(|_| Value::Object(target))
}

fn array_proto_flat_map(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let callback = require_array_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "flatMap",
    )?;
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    let len = length_of_array_like(cx, object)?;
    let target = array_species_create(cx, object, 0)?;
    let mut target_index = 0;
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        let mapped = cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[value, Value::Number(index as f64), Value::Object(object)],
        )?;
        if let Value::Object(mapped_object) = mapped.clone() {
            if matches!(cx.heap().get(mapped_object)?.kind, super::ObjectKind::Array) {
                target_index =
                    flatten_array_like_into_target(cx, target, mapped_object, 1, target_index)?;
                continue;
            }
        }
        CreateDataPropertyOrThrow(cx, target, PropertyKey::array_index(target_index), mapped)?;
        target_index += 1;
    }
    Ok(Value::Object(target))
}

fn flatten_array_like_into_target(
    cx: &mut Context,
    target: ObjectRef,
    object: ObjectRef,
    depth: u32,
    mut target_index: u64,
) -> Completion<u64> {
    let len = length_of_array_like(cx, object)?;
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        if depth > 0 {
            if let Value::Object(candidate) = value.clone() {
                if matches!(cx.heap().get(candidate)?.kind, super::ObjectKind::Array) {
                    target_index = flatten_array_like_into_target(
                        cx,
                        target,
                        candidate,
                        depth - 1,
                        target_index,
                    )?;
                    continue;
                }
            }
        }
        CreateDataPropertyOrThrow(cx, target, PropertyKey::array_index(target_index), value)?;
        target_index += 1;
    }
    Ok(target_index)
}

fn array_species_create(
    cx: &mut Context,
    original: ObjectRef,
    length: u32,
) -> Completion<ObjectRef> {
    let mut constructor = Value::Undefined;
    if is_array_object(cx, original)? {
        constructor = original.get(
            cx,
            &PropertyKey::from("constructor"),
            Value::Object(original),
        )?;
        if matches!(constructor, Value::Undefined) {
            return create_empty_array_with_length(cx, length);
        }
        let Value::Object(constructor_object) = constructor.clone() else {
            return Err(JsError::type_error(
                "Array species constructor must be object or undefined",
            ));
        };
        let species = constructor_object.get(
            cx,
            &PropertyKey::Symbol(SYMBOL_SPECIES_ID),
            constructor.clone(),
        )?;
        constructor = if matches!(species, Value::Null | Value::Undefined) {
            Value::Undefined
        } else {
            species
        };
    }
    if matches!(constructor, Value::Undefined) {
        return create_empty_array_with_length(cx, length);
    }
    if !cx.is_constructor(&constructor)? {
        return Err(JsError::type_error("Array species is not a constructor"));
    }
    let Value::Object(result) = cx.construct_mut(constructor, &[Value::Number(length as f64)])?
    else {
        return Err(JsError::type_error(
            "Array species constructor returned non-object",
        ));
    };
    Ok(result)
}

fn reject_array_create_overflow(cx: &mut Context, object: ObjectRef) -> Completion<()> {
    let value = object.get(cx, &PropertyKey::from("length"), Value::Object(object))?;
    let length = to_number_value(cx, value)?;
    if length.is_finite() && length > u32::MAX as f64 {
        return Err(JsError::range_error("invalid array length"));
    }
    Ok(())
}

fn array_proto_map(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    reject_array_create_overflow(cx, object)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Array.prototype.map callback is not callable",
        ));
    }
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    let len = length_of_array_like(cx, object)?;
    let mut values = Vec::new();
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            values.push(Value::Undefined);
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        values.push(cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[value, Value::Number(index as f64), Value::Object(object)],
        )?);
    }
    create_array_like(cx, values)
}

fn array_proto_filter(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Array.prototype.filter callback is not callable",
        ));
    }
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    let len = length_of_array_like(cx, object)?;
    let mut values = Vec::new();
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        let selected = cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[
                value.clone(),
                Value::Number(index as f64),
                Value::Object(object),
            ],
        )?;
        if truthy(selected) {
            values.push(value);
        }
    }
    create_array_like(cx, values)
}

fn array_proto_for_each(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Array.prototype.forEach callback is not callable",
        ));
    }
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    let len = length_of_array_like(cx, object)?;
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[value, Value::Number(index as f64), Value::Object(object)],
        )?;
    }
    Ok(Value::Undefined)
}

enum PredicateScan {
    Every,
    Some,
}

enum SearchDirection {
    Forward,
    Reverse,
}

fn require_array_callback(cx: &Context, callback: Value, method_name: &str) -> Completion<Value> {
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(format!(
            "Array.prototype.{method_name} callback is not callable"
        )));
    }
    Ok(callback)
}

fn array_predicate_scan(
    cx: &mut Context,
    object: ObjectRef,
    callback: Value,
    this_arg: Value,
    len: u32,
    mode: PredicateScan,
) -> Completion<Value> {
    for index in 0..len {
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        let selected = cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[value, Value::Number(index as f64), Value::Object(object)],
        )?;
        let selected = truthy(selected);
        match mode {
            PredicateScan::Every if !selected => return Ok(Value::Boolean(false)),
            PredicateScan::Some if selected => return Ok(Value::Boolean(true)),
            _ => {}
        }
    }
    Ok(Value::Boolean(matches!(mode, PredicateScan::Every)))
}

fn array_index_search(
    cx: &mut Context,
    object: ObjectRef,
    search: Value,
    len: u32,
    start: u32,
    direction: SearchDirection,
) -> Completion<Value> {
    if len == 0 {
        return Ok(Value::Number(-1.0));
    }
    match direction {
        SearchDirection::Forward => {
            let mut index = start;
            while index < len {
                let key = PropertyKey::array_index(index as u64);
                if object.has_property(cx, &key)? {
                    let value = object.get(cx, &key, Value::Object(object))?;
                    if strict_equal_value(&value, &search) {
                        return Ok(Value::Number(index as f64));
                    }
                }
                index += 1;
            }
        }
        SearchDirection::Reverse => {
            let mut index = start.min(len - 1);
            loop {
                let key = PropertyKey::array_index(index as u64);
                if object.has_property(cx, &key)? {
                    let value = object.get(cx, &key, Value::Object(object))?;
                    if strict_equal_value(&value, &search) {
                        return Ok(Value::Number(index as f64));
                    }
                }
                if index == 0 {
                    break;
                }
                index -= 1;
            }
        }
    }
    Ok(Value::Number(-1.0))
}

fn strict_equal_value(left: &Value, right: &Value) -> bool {
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

fn array_proto_fill(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    let start =
        array_relative_index_f64(cx, args.get(1).cloned().unwrap_or(Value::Undefined), len, 0)?
            as u64;
    let end = array_relative_index_f64(
        cx,
        args.get(2).cloned().unwrap_or(Value::Undefined),
        len,
        len as u64,
    )? as u64;
    let mut index = start;
    while index < end {
        set_property_or_throw(cx, object, PropertyKey::array_index(index), value.clone())?;
        index += 1;
    }
    Ok(Value::Object(object))
}

fn array_proto_copy_within(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    let target = array_relative_index_f64(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        len,
        0,
    )? as u64;
    let start =
        array_relative_index_f64(cx, args.get(1).cloned().unwrap_or(Value::Undefined), len, 0)?
            as u64;
    let end = array_relative_index_f64(
        cx,
        args.get(2).cloned().unwrap_or(Value::Undefined),
        len,
        len as u64,
    )? as u64;
    let len = len as u64;
    let count = end.saturating_sub(start).min(len.saturating_sub(target));
    if count > 100_000 {
        array_copy_within_sparse(cx, object, target, start, count)?;
        return Ok(Value::Object(object));
    }
    let backwards = start < target && target < start + count;
    let mut offset = 0;
    while offset < count {
        let from_index = if backwards {
            start + count - 1 - offset
        } else {
            start + offset
        };
        let to_index = if backwards {
            target + count - 1 - offset
        } else {
            target + offset
        };
        let from_key = PropertyKey::array_index(from_index);
        let to_key = PropertyKey::array_index(to_index);
        if object.has_property(cx, &from_key)? {
            let value = object.get(cx, &from_key, Value::Object(object))?;
            set_property_or_throw(cx, object, to_key, value)?;
        } else {
            delete_property_or_throw(cx, object, to_key)?;
        }
        offset += 1;
    }
    Ok(Value::Object(object))
}

fn array_copy_within_sparse(
    cx: &mut Context,
    object: ObjectRef,
    target: u64,
    start: u64,
    count: u64,
) -> Completion<()> {
    let backwards = start < target && target < start + count;
    let keys = own_integer_indices(cx, object)?;
    let key_set: HashSet<u64> = keys.iter().copied().collect();
    let mut actions = Vec::new();

    for key_index in keys {
        if key_index >= start && key_index < start + count {
            let to = target + (key_index - start);
            let value = object.get(
                cx,
                &PropertyKey::array_index(key_index),
                Value::Object(object),
            )?;
            actions.push((key_index - start, to, Some(value)));
        } else if key_index >= target && key_index < target + count {
            let from = start + (key_index - target);
            if !key_set.contains(&from) {
                actions.push((key_index - target, key_index, None));
            }
        }
    }

    if backwards {
        actions.sort_by(|left, right| right.0.cmp(&left.0));
    } else {
        actions.sort_by_key(|action| action.0);
    }

    for (_, to, value) in actions {
        let key = PropertyKey::array_index(to);
        if let Some(value) = value {
            set_property_or_throw(cx, object, key, value)?;
        } else {
            delete_property_or_throw(cx, object, key)?;
        }
    }
    Ok(())
}

fn array_last_index_of_sparse(
    cx: &mut Context,
    object: ObjectRef,
    search: Value,
    start: u64,
) -> Completion<Value> {
    let mut keys = own_integer_indices(cx, object)?;
    keys.retain(|index| *index <= start);
    keys.sort_by(|left, right| right.cmp(left));
    for index in keys {
        let key = PropertyKey::array_index(index);
        let value = object.get(cx, &key, Value::Object(object))?;
        if strict_equal_value(&value, &search) {
            return Ok(Value::Number(index as f64));
        }
    }
    Ok(Value::Number(-1.0))
}

fn array_index_of_sparse(
    cx: &mut Context,
    object: ObjectRef,
    search: Value,
    start: u64,
    len: u64,
) -> Completion<Value> {
    let mut keys = own_integer_indices(cx, object)?;
    keys.retain(|index| *index >= start && *index < len);
    keys.sort_unstable();
    for index in keys {
        let key = PropertyKey::array_index(index);
        let value = object.get(cx, &key, Value::Object(object))?;
        if strict_equal_value(&value, &search) {
            return Ok(Value::Number(index as f64));
        }
    }
    Ok(Value::Number(-1.0))
}

fn array_includes_sparse(
    cx: &mut Context,
    object: ObjectRef,
    search: Value,
    start: u64,
    len: u64,
) -> Completion<Value> {
    if matches!(search, Value::Undefined) && start < len {
        return Ok(Value::Boolean(true));
    }
    let mut keys = own_integer_indices(cx, object)?;
    keys.retain(|index| *index >= start && *index < len);
    keys.sort_unstable();
    for index in keys {
        let key = PropertyKey::array_index(index);
        let value = object.get(cx, &key, Value::Object(object))?;
        if SameValueZero(&value, &search) {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

fn array_reduce_right_sparse(
    cx: &mut Context,
    object: ObjectRef,
    callback: Value,
    len: u64,
    initial: Option<Value>,
) -> Completion<Value> {
    let mut keys = own_integer_indices(cx, object)?;
    keys.retain(|index| *index < len);
    keys.sort_by(|left, right| right.cmp(left));

    let mut iter = keys.into_iter();
    let mut accumulator = if let Some(initial) = initial {
        initial
    } else {
        let Some(index) = iter.next() else {
            return Err(JsError::type_error(
                "reduceRight of empty array with no initial value",
            ));
        };
        object.get(cx, &PropertyKey::array_index(index), Value::Object(object))?
    };

    for index in iter {
        let value = object.get(cx, &PropertyKey::array_index(index), Value::Object(object))?;
        accumulator = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[
                accumulator,
                value,
                Value::Number(index as f64),
                Value::Object(object),
            ],
        )?;
    }
    Ok(accumulator)
}

enum FindMode {
    Value,
    Index,
}

enum FindDirection {
    Forward,
    Reverse,
}

fn array_find(
    cx: &mut Context,
    object: ObjectRef,
    callback: Value,
    this_arg: Value,
    mode: FindMode,
    direction: FindDirection,
) -> Completion<Value> {
    let len = length_of_array_like_f64(cx, object)?;
    if len == 0.0 {
        return Ok(match mode {
            FindMode::Value => Value::Undefined,
            FindMode::Index => Value::Number(-1.0),
        });
    }
    let mut index = match direction {
        FindDirection::Forward => 0.0,
        FindDirection::Reverse => len - 1.0,
    };
    loop {
        let value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        let selected = cx.call_mut(
            callback.clone(),
            this_arg.clone(),
            &[
                value.clone(),
                Value::Number(index as f64),
                Value::Object(object),
            ],
        )?;
        if truthy(selected) {
            return Ok(match mode {
                FindMode::Value => value,
                FindMode::Index => Value::Number(index as f64),
            });
        }
        match direction {
            FindDirection::Forward => {
                index += 1.0;
                if index >= len {
                    break;
                }
            }
            FindDirection::Reverse => {
                if index == 0.0 {
                    break;
                }
                index -= 1.0;
            }
        }
    }
    Ok(match mode {
        FindMode::Value => Value::Undefined,
        FindMode::Index => Value::Number(-1.0),
    })
}

fn array_proto_find(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let callback = require_array_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "find",
    )?;
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    array_find(
        cx,
        object,
        callback,
        this_arg,
        FindMode::Value,
        FindDirection::Forward,
    )
}

fn array_proto_find_index(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let callback = require_array_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "findIndex",
    )?;
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    array_find(
        cx,
        object,
        callback,
        this_arg,
        FindMode::Index,
        FindDirection::Forward,
    )
}

fn array_proto_find_last(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let callback = require_array_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "findLast",
    )?;
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    array_find(
        cx,
        object,
        callback,
        this_arg,
        FindMode::Value,
        FindDirection::Reverse,
    )
}

fn array_proto_find_last_index(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let callback = require_array_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "findLastIndex",
    )?;
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    array_find(
        cx,
        object,
        callback,
        this_arg,
        FindMode::Index,
        FindDirection::Reverse,
    )
}

fn array_proto_every(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like(cx, object)?;
    let callback = require_array_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "every",
    )?;
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    array_predicate_scan(cx, object, callback, this_arg, len, PredicateScan::Every)
}

fn array_proto_some(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like(cx, object)?;
    let callback = require_array_callback(
        cx,
        args.first().cloned().unwrap_or(Value::Undefined),
        "some",
    )?;
    let this_arg = args.get(1).cloned().unwrap_or(Value::Undefined);
    array_predicate_scan(cx, object, callback, this_arg, len, PredicateScan::Some)
}

fn array_proto_includes(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let len = length_of_array_like_f64(cx, object)?;
    if len == 0.0 {
        return Ok(Value::Boolean(false));
    }
    let mut index =
        forward_search_start_f64(cx, args.get(1).cloned().unwrap_or(Value::Undefined), len)?;
    if index >= len {
        return Ok(Value::Boolean(false));
    }
    if len > 100_000.0 || index > 100_000.0 {
        return array_includes_sparse(cx, object, search, index as u64, len as u64);
    }
    while index < len {
        let value = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        if SameValueZero(&value, &search) {
            return Ok(Value::Boolean(true));
        }
        index += 1.0;
    }
    Ok(Value::Boolean(false))
}

fn array_proto_index_of(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let len = length_of_array_like_f64(cx, object)?;
    if len == 0.0 {
        return Ok(Value::Number(-1.0));
    }
    let index =
        forward_search_start_f64(cx, args.get(1).cloned().unwrap_or(Value::Undefined), len)?;
    if index >= len {
        return Ok(Value::Number(-1.0));
    }
    if len > 100_000.0 || index > 100_000.0 {
        return array_index_of_sparse(cx, object, search, index as u64, len as u64);
    }
    array_index_search(
        cx,
        object,
        search,
        len as u32,
        index as u32,
        SearchDirection::Forward,
    )
}

fn array_proto_last_index_of(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let search = args.first().cloned().unwrap_or(Value::Undefined);
    let len = length_of_array_like_f64(cx, object)?;
    if len == 0.0 {
        return Ok(Value::Number(-1.0));
    }
    let Some(index) = reverse_search_start_f64(cx, args.get(1).cloned(), len)? else {
        return Ok(Value::Number(-1.0));
    };
    if index > 100_000.0 {
        return array_last_index_of_sparse(cx, object, search, index as u64);
    }
    array_index_search(
        cx,
        object,
        search,
        len as u32,
        index as u32,
        SearchDirection::Reverse,
    )
}

fn array_proto_reduce(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like(cx, object)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Array.prototype.reduce callback is not callable",
        ));
    }
    let mut index = 0;
    let mut accumulator = if let Some(initial) = args.get(1) {
        initial.clone()
    } else {
        let mut found = None;
        while index < len {
            let key = PropertyKey::array_index(index as u64);
            if object.has_property(cx, &key)? {
                found = Some(object.get(cx, &key, Value::Object(object))?);
                index += 1;
                break;
            }
            index += 1;
        }
        found.ok_or_else(|| JsError::type_error("reduce of empty array with no initial value"))?
    };
    while index < len {
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            index += 1;
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        accumulator = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[
                accumulator,
                value,
                Value::Number(index as f64),
                Value::Object(object),
            ],
        )?;
        index += 1;
    }
    Ok(accumulator)
}

fn array_proto_reduce_right(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_value(cx, this)?;
    let len = length_of_array_like_f64(cx, object)?;
    let callback = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "Array.prototype.reduceRight callback is not callable",
        ));
    }
    if len == 0.0 && args.get(1).is_none() {
        return Err(JsError::type_error(
            "reduceRight of empty array with no initial value",
        ));
    }
    if len > 100_000.0 {
        return array_reduce_right_sparse(cx, object, callback, len as u64, args.get(1).cloned());
    }
    let len = len as u32;
    let mut index = len;
    let mut accumulator = if let Some(initial) = args.get(1) {
        initial.clone()
    } else {
        let mut found = None;
        while index > 0 {
            index -= 1;
            let key = PropertyKey::array_index(index as u64);
            if object.has_property(cx, &key)? {
                found = Some(object.get(cx, &key, Value::Object(object))?);
                break;
            }
        }
        found.ok_or_else(|| {
            JsError::type_error("reduceRight of empty array with no initial value")
        })?
    };
    while index > 0 {
        index -= 1;
        let key = PropertyKey::array_index(index as u64);
        if !object.has_property(cx, &key)? {
            continue;
        }
        let value = object.get(cx, &key, Value::Object(object))?;
        accumulator = cx.call_mut(
            callback.clone(),
            Value::Undefined,
            &[
                accumulator,
                value,
                Value::Number(index as f64),
                Value::Object(object),
            ],
        )?;
    }
    Ok(accumulator)
}

fn primitive_proto_value_of(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    primitive_this_value(cx, this)
}

fn boolean_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    match primitive_this_value(cx, this)? {
        Value::Boolean(value) => Ok(Value::String(
            if value { "true" } else { "false" }.to_owned(),
        )),
        _ => Err(JsError::type_error(
            "Boolean.prototype.toString receiver is not boolean",
        )),
    }
}

fn number_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    match primitive_this_value(cx, this)? {
        Value::Number(value) => Ok(Value::String(number_to_property_string(value))),
        _ => Err(JsError::type_error(
            "Number.prototype.toString receiver is not number",
        )),
    }
}

fn number_proto_to_fixed(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_number_value(cx, this)?;
    let digits = number_format_digits(cx, args.first().cloned().unwrap_or(Value::Undefined), 0)?;
    if !value.is_finite() {
        return Ok(Value::String(number_to_property_string(value)));
    }
    Ok(Value::String(format!("{value:.digits$}")))
}

fn number_proto_to_exponential(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_number_value(cx, this)?;
    if !value.is_finite() {
        return Ok(Value::String(number_to_property_string(value)));
    }
    let text = if matches!(args.first(), None | Some(Value::Undefined)) {
        format!("{value:e}")
    } else {
        let digits =
            number_format_digits(cx, args.first().cloned().unwrap_or(Value::Undefined), 0)?;
        format!("{value:.digits$e}")
    };
    Ok(Value::String(normalize_exponent_text(text)))
}

fn number_proto_to_precision(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_number_value(cx, this)?;
    if matches!(args.first(), None | Some(Value::Undefined)) {
        return Ok(Value::String(number_to_property_string(value)));
    }
    let precision = number_format_digits(cx, args.first().cloned().unwrap_or(Value::Undefined), 1)?;
    if !value.is_finite() {
        return Ok(Value::String(number_to_property_string(value)));
    }
    Ok(Value::String(normalize_exponent_text(format!(
        "{value:.precision$e}",
        precision = precision - 1
    ))))
}

fn string_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    match primitive_this_value(cx, this)? {
        Value::String(value) => Ok(Value::String(value)),
        _ => Err(JsError::type_error(
            "String.prototype.toString receiver is not string",
        )),
    }
}

fn symbol_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    match primitive_this_value(cx, this)? {
        Value::Symbol(symbol) => Ok(Value::String(symbol_descriptive_string(cx, symbol))),
        _ => Err(JsError::type_error(
            "Symbol.prototype.toString receiver is not symbol",
        )),
    }
}

fn symbol_descriptive_string(cx: &Context, symbol: u64) -> String {
    cx.symbol_description(symbol)
        .map(|description| format!("Symbol({description})"))
        .unwrap_or_else(|| "Symbol()".to_owned())
}

fn string_proto_char_at(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let units = string_units(&value);
    let index = string_index_arg(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    if index < 0 || index as usize >= units.len() {
        return Ok(Value::String(String::new()));
    }
    Ok(Value::String(string_from_units(
        &units[index as usize..index as usize + 1],
    )))
}

fn string_proto_at(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let units = string_units(&value);
    let len = units.len() as i64;
    let index = string_index_arg(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let relative = if index < 0 { len + index } else { index };
    if relative < 0 || relative >= len {
        return Ok(Value::Undefined);
    }
    Ok(Value::String(string_from_units(
        &units[relative as usize..relative as usize + 1],
    )))
}

fn string_proto_char_code_at(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let units = string_units(&value);
    let index = string_index_arg(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    if index < 0 || index as usize >= units.len() {
        return Ok(Value::Number(f64::NAN));
    }
    Ok(Value::Number(units[index as usize] as f64))
}

fn string_proto_code_point_at(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let units = string_units(&value);
    let index = string_index_arg(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    if index < 0 || index as usize >= units.len() {
        return Ok(Value::Undefined);
    }
    let first = units[index as usize];
    if (0xd800..=0xdbff).contains(&first) {
        if let Some(second) = units.get(index as usize + 1) {
            if (0xdc00..=0xdfff).contains(second) {
                let high = (first as u32) - 0xd800;
                let low = (*second as u32) - 0xdc00;
                return Ok(Value::Number((0x10000 + ((high << 10) | low)) as f64));
            }
        }
    }
    Ok(Value::Number(first as f64))
}

fn string_proto_concat(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let mut value = this_string_value(cx, this)?;
    for arg in args {
        value.push_str(&to_string_value(cx, arg.clone())?);
    }
    Ok(Value::String(value))
}

fn string_proto_index_of(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let search = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let position = string_index_arg(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    let haystack = string_units(&value);
    let needle = string_units(&search);
    let start = position.max(0) as usize;
    Ok(Value::Number(
        find_units(&haystack, &needle, start.min(haystack.len()))
            .map(|index| index as f64)
            .unwrap_or(-1.0),
    ))
}

fn string_proto_last_index_of(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let search = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let haystack = string_units(&value);
    let needle = string_units(&search);
    let raw_position = args.get(1).cloned().unwrap_or(Value::Number(f64::INFINITY));
    let position = to_number_value(cx, raw_position)?;
    let start = if position.is_nan() || position.is_infinite() {
        haystack.len()
    } else {
        position.trunc().max(0.0).min(haystack.len() as f64) as usize
    };
    Ok(Value::Number(
        rfind_units(&haystack, &needle, start)
            .map(|index| index as f64)
            .unwrap_or(-1.0),
    ))
}

fn string_proto_locale_compare(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let other = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    Ok(Value::Number(match value.cmp(&other) {
        std::cmp::Ordering::Less => -1.0,
        std::cmp::Ordering::Equal => 0.0,
        std::cmp::Ordering::Greater => 1.0,
    }))
}

fn string_proto_normalize(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let form = to_string_value(
        cx,
        args.first()
            .cloned()
            .unwrap_or(Value::String("NFC".to_owned())),
    )?;
    match form.as_str() {
        "NFC" | "NFD" | "NFKC" | "NFKD" => Ok(Value::String(value)),
        _ => Err(JsError::range_error("invalid normalization form")),
    }
}

fn string_proto_repeat(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let count = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    if count.is_infinite() || count < 0.0 {
        return Err(JsError::range_error("invalid repeat count"));
    }
    if count.is_nan() || count == 0.0 {
        return Ok(Value::String(String::new()));
    }
    Ok(Value::String(value.repeat(count.trunc() as usize)))
}

fn string_proto_pad_start(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    string_pad(cx, this, args, true)
}

fn string_proto_pad_end(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    string_pad(cx, this, args, false)
}

fn string_proto_includes(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let search = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let position = string_index_arg(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    let haystack = string_units(&value);
    let needle = string_units(&search);
    Ok(Value::Boolean(
        find_units(&haystack, &needle, position.max(0) as usize).is_some(),
    ))
}

fn string_proto_starts_with(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let search = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let position = string_index_arg(cx, args.get(1).cloned().unwrap_or(Value::Undefined))?;
    let haystack = string_units(&value);
    let needle = string_units(&search);
    let start = position.max(0) as usize;
    Ok(Value::Boolean(
        start <= haystack.len() && haystack[start..].starts_with(&needle),
    ))
}

fn string_proto_ends_with(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let search = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let haystack = string_units(&value);
    let needle = string_units(&search);
    let end = args
        .get(1)
        .cloned()
        .map(|value| string_index_arg(cx, value))
        .transpose()?
        .unwrap_or(haystack.len() as i64)
        .max(0)
        .min(haystack.len() as i64) as usize;
    Ok(Value::Boolean(
        end >= needle.len() && haystack[..end].ends_with(&needle),
    ))
}

fn string_proto_slice(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let units = string_units(&value);
    let len = units.len() as i64;
    let start = relative_string_index(
        string_index_arg(cx, args.first().cloned().unwrap_or(Value::Undefined))?,
        len,
    );
    let end = args
        .get(1)
        .cloned()
        .map(|value| string_index_arg(cx, value))
        .transpose()?
        .map(|index| relative_string_index(index, len))
        .unwrap_or(len);
    if end <= start {
        return Ok(Value::String(String::new()));
    }
    Ok(Value::String(string_from_units(
        &units[start as usize..end as usize],
    )))
}

fn string_proto_split(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let limit = if let Some(value) = args.get(1) {
        to_uint32(to_number_value(cx, value.clone())?) as usize
    } else {
        usize::MAX
    };
    if limit == 0 {
        return Ok(Value::Object(create_array_like_value(cx, Vec::new())?));
    }
    let separator = args.first().cloned().unwrap_or(Value::Undefined);
    let values = if matches!(separator, Value::Undefined) {
        vec![Value::String(value)]
    } else {
        let separator = to_string_value(cx, separator)?;
        if separator.is_empty() {
            value
                .chars()
                .take(limit)
                .map(|ch| Value::String(ch.to_string()))
                .collect()
        } else {
            value
                .split(&separator)
                .take(limit)
                .map(|part| Value::String(part.to_owned()))
                .collect()
        }
    };
    Ok(Value::Object(create_array_like_value(cx, values)?))
}

fn string_proto_substring(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let units = string_units(&value);
    let len = units.len() as i64;
    let start = string_index_arg(cx, args.first().cloned().unwrap_or(Value::Undefined))?
        .max(0)
        .min(len);
    let end = args
        .get(1)
        .cloned()
        .map(|value| string_index_arg(cx, value))
        .transpose()?
        .map(|index| index.max(0).min(len))
        .unwrap_or(len);
    let (from, to) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    Ok(Value::String(string_from_units(
        &units[from as usize..to as usize],
    )))
}

fn string_proto_to_lower_case(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::String(this_string_value(cx, this)?.to_lowercase()))
}

fn string_proto_to_upper_case(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::String(this_string_value(cx, this)?.to_uppercase()))
}

fn string_proto_is_well_formed(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    let _ = this_string_value(cx, this)?;
    Ok(Value::Boolean(true))
}

fn string_proto_trim(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::String(trim_js_string(
        &this_string_value(cx, this)?,
        true,
        true,
    )))
}

fn string_proto_trim_start(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::String(trim_js_string(
        &this_string_value(cx, this)?,
        true,
        false,
    )))
}

fn string_proto_trim_end(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::String(trim_js_string(
        &this_string_value(cx, this)?,
        false,
        true,
    )))
}

fn string_proto_to_well_formed(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    Ok(Value::String(this_string_value(cx, this)?))
}

fn string_from_char_code(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let mut units = Vec::with_capacity(args.len());
    for arg in args {
        let number = to_number_value(cx, arg.clone())?;
        units.push((number as i64 & 0xffff) as u16);
    }
    Ok(Value::String(string_from_units(&units)))
}

fn string_from_code_point(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let mut out = String::new();
    for arg in args {
        let number = to_number_value(cx, arg.clone())?;
        if !number.is_finite() || number < 0.0 || number > 0x10ffff as f64 || number.fract() != 0.0
        {
            return Err(JsError::range_error("invalid code point"));
        }
        let Some(ch) = char::from_u32(number as u32) else {
            return Err(JsError::range_error("invalid code point"));
        };
        out.push(ch);
    }
    Ok(Value::String(out))
}

fn string_raw(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let template = ArgView::new(args.first().cloned().unwrap_or(Value::Undefined)).to_object(cx)?;
    let raw = template.get(cx, &PropertyKey::from("raw"), Value::Object(template))?;
    let raw_object = ArgView::new(raw).to_object(cx)?;
    let length = length_of_array_like(cx, raw_object)? as usize;
    let mut out = String::new();
    for index in 0..length {
        let raw_piece = raw_object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(raw_object),
        )?;
        out.push_str(&to_string_value(cx, raw_piece)?);
        if index + 1 < length {
            out.push_str(&to_string_value(
                cx,
                args.get(index + 1)
                    .cloned()
                    .unwrap_or(Value::String(String::new())),
            )?);
        }
    }
    Ok(Value::String(out))
}

fn regexp_proto_exec(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error(
            "RegExp.prototype.exec receiver must be object",
        ));
    };
    let (source, flags) = regexp_source_flags(cx, object)?;
    let input = to_string_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let Some((start, end)) = simple_regexp_find(&source, &flags, &input) else {
        return Ok(Value::Null);
    };
    let matched = input
        .get(start..end)
        .map(str::to_owned)
        .unwrap_or_else(String::new);
    let result = create_array_like_value(cx, vec![Value::String(matched)])?;
    result.define_own_property_or_throw(
        cx,
        PropertyKey::from("index"),
        Descriptor::data(Value::Number(start as f64), true, true, true),
    )?;
    result.define_own_property_or_throw(
        cx,
        PropertyKey::from("input"),
        Descriptor::data(Value::String(input), true, true, true),
    )?;
    Ok(Value::Object(result))
}

fn regexp_proto_test(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let result = regexp_proto_exec(cx, this, args)?;
    Ok(Value::Boolean(!matches!(result, Value::Null)))
}

fn regexp_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error(
            "RegExp.prototype.toString receiver must be object",
        ));
    };
    let (source, flags) = regexp_source_flags(cx, object)?;
    Ok(Value::String(format!("/{source}/{flags}")))
}

fn regexp_proto_symbol_replace(
    _cx: &mut Context,
    _this: Value,
    _args: &[Value],
) -> Completion<Value> {
    Ok(Value::Undefined)
}

fn regexp_proto_get_global(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    regexp_flag_value(cx, this, 'g')
}

fn regexp_proto_get_ignore_case(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    regexp_flag_value(cx, this, 'i')
}

fn regexp_proto_get_multiline(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    regexp_flag_value(cx, this, 'm')
}

fn regexp_proto_get_dot_all(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    regexp_flag_value(cx, this, 's')
}

fn regexp_proto_get_unicode(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    regexp_flag_value(cx, this, 'u')
}

fn regexp_proto_get_sticky(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    regexp_flag_value(cx, this, 'y')
}

fn regexp_proto_get_has_indices(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    regexp_flag_value(cx, this, 'd')
}

fn regexp_flag_value(cx: &mut Context, this: Value, flag: char) -> Completion<Value> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error(
            "RegExp.prototype flag getter receiver must be object",
        ));
    };
    let (_, flags) = regexp_source_flags(cx, object)?;
    Ok(Value::Boolean(flags.contains(flag)))
}

fn date_this_object(cx: &mut Context, this: Value) -> Completion<ObjectRef> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error(
            "Date.prototype method receiver must be object",
        ));
    };
    if !cx.heap().get(object)?.has_brand(super::Brand::Date) {
        return Err(JsError::type_error(
            "Date.prototype method receiver is not a Date object",
        ));
    }
    Ok(object)
}

fn date_this_time_value(cx: &mut Context, this: Value) -> Completion<f64> {
    let object = date_this_object(cx, this)?;
    match cx.heap().get(object)?.primitive_value.clone() {
        Some(Value::Number(value)) => Ok(value),
        _ => Ok(f64::NAN),
    }
}

fn date_set_time_value(cx: &mut Context, this: Value, time: f64) -> Completion<Value> {
    let object = date_this_object(cx, this)?;
    let clipped = time_clip(time);
    cx.heap_mut().get_mut(object)?.primitive_value = Some(Value::Number(clipped));
    Ok(Value::Number(clipped))
}

fn date_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this)?;
    if time.is_nan() {
        Ok(Value::String("Invalid Date".to_owned()))
    } else {
        Ok(Value::String(date_to_string(time)?))
    }
}

fn date_proto_to_date_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this)?;
    if time.is_nan() {
        Ok(Value::String("Invalid Date".to_owned()))
    } else {
        Ok(Value::String(date_date_string(time)?))
    }
}

fn date_proto_to_time_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this)?;
    if time.is_nan() {
        Ok(Value::String("Invalid Date".to_owned()))
    } else {
        Ok(Value::String(date_time_string(time)?))
    }
}

fn date_proto_to_utc_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this)?;
    if time.is_nan() {
        Ok(Value::String("Invalid Date".to_owned()))
    } else {
        Ok(Value::String(date_utc_string(time)?))
    }
}

fn date_proto_to_iso_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this)?;
    if time.is_nan() {
        Err(JsError::range_error("Invalid time value"))
    } else {
        Ok(Value::String(iso_string_from_time(time)?))
    }
}

fn date_proto_to_json(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let object = ArgView::new(this).to_object(cx)?;
    let primitive = ArgView::new(Value::Object(object)).to_primitive(cx)?;
    if let Value::Number(number) = primitive {
        if !number.is_finite() {
            return Ok(Value::Null);
        }
    }
    let method = object.get(cx, &PropertyKey::from("toISOString"), Value::Object(object))?;
    if !cx.is_callable(&method)? {
        return Err(JsError::type_error("toISOString is not callable"));
    }
    cx.call_mut(method, Value::Object(object), &[])
}

fn date_proto_symbol_to_primitive(
    cx: &mut Context,
    this: Value,
    args: &[Value],
) -> Completion<Value> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error(
            "Date.prototype[Symbol.toPrimitive] receiver must be object",
        ));
    };
    let string_first = match args.first().cloned().unwrap_or(Value::Undefined) {
        Value::String(hint) if hint == "string" || hint == "default" => true,
        Value::String(hint) if hint == "number" => false,
        _ => {
            return Err(JsError::type_error(
                "Date.prototype[Symbol.toPrimitive] invalid hint",
            ));
        }
    };
    let method_names = if string_first {
        ["toString", "valueOf"]
    } else {
        ["valueOf", "toString"]
    };
    for name in method_names {
        let method = object.get(cx, &PropertyKey::from(name), Value::Object(object))?;
        if matches!(method, Value::Undefined | Value::Null) || !cx.is_callable(&method)? {
            continue;
        }
        let result = cx.call_mut(method, Value::Object(object), &[])?;
        if !result.is_object() {
            return Ok(result);
        }
    }
    Err(JsError::type_error(
        "Date.prototype[Symbol.toPrimitive] could not produce primitive",
    ))
}

fn date_proto_value_of(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    Ok(Value::Number(date_this_time_value(cx, this)?))
}

fn date_utc_parts(time: f64) -> Option<(i32, u32, u32, u32, u32, u32, u32, u32)> {
    if time.is_nan() {
        return None;
    }
    let day = (time / 86_400_000.0).floor() as i64;
    let time_within_day = time - day as f64 * 86_400_000.0;
    let (year, month, date) = civil_from_days(day);
    let hour = (time_within_day / 3_600_000.0).floor() as u32;
    let minute = ((time_within_day % 3_600_000.0) / 60_000.0).floor() as u32;
    let second = ((time_within_day % 60_000.0) / 1000.0).floor() as u32;
    let millisecond = (time_within_day % 1000.0).floor() as u32;
    let weekday = (day + 4).rem_euclid(7) as u32;
    Some((
        year,
        month,
        date,
        weekday,
        hour,
        minute,
        second,
        millisecond,
    ))
}

fn date_utc_part(
    cx: &mut Context,
    this: Value,
    selector: fn((i32, u32, u32, u32, u32, u32, u32, u32)) -> f64,
) -> Completion<Value> {
    let time = date_this_time_value(cx, this)?;
    Ok(Value::Number(
        date_utc_parts(time).map_or(f64::NAN, selector),
    ))
}

fn date_proto_get_timezone_offset(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    let time = date_this_time_value(cx, this)?;
    if time.is_nan() {
        Ok(Value::Number(f64::NAN))
    } else {
        Ok(Value::Number(0.0))
    }
}

fn date_proto_get_utc_full_year(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.0 as f64)
}

fn date_proto_get_utc_month(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.1 as f64 - 1.0)
}

fn date_proto_get_utc_date(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.2 as f64)
}

fn date_proto_get_utc_day(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.3 as f64)
}

fn date_proto_get_utc_hours(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.4 as f64)
}

fn date_proto_get_utc_minutes(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.5 as f64)
}

fn date_proto_get_utc_seconds(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.6 as f64)
}

fn date_proto_get_utc_milliseconds(
    cx: &mut Context,
    this: Value,
    _args: &[Value],
) -> Completion<Value> {
    date_utc_part(cx, this, |parts| parts.7 as f64)
}

fn date_time_within_day(time: f64) -> f64 {
    let day = (time / 86_400_000.0).floor();
    time - day * 86_400_000.0
}

fn date_proto_set_time(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    date_this_object(cx, this.clone())?;
    let time = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    date_set_time_value(cx, this, time)
}

fn date_proto_set_utc_milliseconds(
    cx: &mut Context,
    this: Value,
    args: &[Value],
) -> Completion<Value> {
    let time = date_this_time_value(cx, this.clone())?;
    let millis = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let Some(parts) = date_utc_parts(time) else {
        return Ok(Value::Number(f64::NAN));
    };
    date_set_time_value(
        cx,
        this,
        make_date(
            make_day(parts.0 as f64, parts.1 as f64 - 1.0, parts.2 as f64),
            make_time(parts.4 as f64, parts.5 as f64, parts.6 as f64, millis),
        ),
    )
}

fn date_proto_set_utc_seconds(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this.clone())?;
    let seconds = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let millis = if let Some(value) = args.get(1) {
        to_number_value(cx, value.clone())?
    } else {
        date_utc_parts(time).map_or(f64::NAN, |parts| parts.7 as f64)
    };
    let Some(parts) = date_utc_parts(time) else {
        return Ok(Value::Number(f64::NAN));
    };
    date_set_time_value(
        cx,
        this,
        make_date(
            make_day(parts.0 as f64, parts.1 as f64 - 1.0, parts.2 as f64),
            make_time(parts.4 as f64, parts.5 as f64, seconds, millis),
        ),
    )
}

fn date_proto_set_utc_minutes(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this.clone())?;
    let minutes = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let seconds = if let Some(value) = args.get(1) {
        to_number_value(cx, value.clone())?
    } else {
        date_utc_parts(time).map_or(f64::NAN, |parts| parts.6 as f64)
    };
    let millis = if let Some(value) = args.get(2) {
        to_number_value(cx, value.clone())?
    } else {
        date_utc_parts(time).map_or(f64::NAN, |parts| parts.7 as f64)
    };
    let Some(parts) = date_utc_parts(time) else {
        return Ok(Value::Number(f64::NAN));
    };
    date_set_time_value(
        cx,
        this,
        make_date(
            make_day(parts.0 as f64, parts.1 as f64 - 1.0, parts.2 as f64),
            make_time(parts.4 as f64, minutes, seconds, millis),
        ),
    )
}

fn date_proto_set_utc_hours(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this.clone())?;
    let hours = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let minutes = if let Some(value) = args.get(1) {
        to_number_value(cx, value.clone())?
    } else {
        date_utc_parts(time).map_or(f64::NAN, |parts| parts.5 as f64)
    };
    let seconds = if let Some(value) = args.get(2) {
        to_number_value(cx, value.clone())?
    } else {
        date_utc_parts(time).map_or(f64::NAN, |parts| parts.6 as f64)
    };
    let millis = if let Some(value) = args.get(3) {
        to_number_value(cx, value.clone())?
    } else {
        date_utc_parts(time).map_or(f64::NAN, |parts| parts.7 as f64)
    };
    let Some(parts) = date_utc_parts(time) else {
        return Ok(Value::Number(f64::NAN));
    };
    date_set_time_value(
        cx,
        this,
        make_date(
            make_day(parts.0 as f64, parts.1 as f64 - 1.0, parts.2 as f64),
            make_time(hours, minutes, seconds, millis),
        ),
    )
}

fn date_proto_set_utc_date(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this.clone())?;
    let date = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let Some(parts) = date_utc_parts(time) else {
        return Ok(Value::Number(f64::NAN));
    };
    date_set_time_value(
        cx,
        this,
        make_date(
            make_day(parts.0 as f64, parts.1 as f64 - 1.0, date),
            date_time_within_day(time),
        ),
    )
}

fn date_proto_set_utc_month(cx: &mut Context, this: Value, args: &[Value]) -> Completion<Value> {
    let time = date_this_time_value(cx, this.clone())?;
    let month = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let date = if let Some(value) = args.get(1) {
        to_number_value(cx, value.clone())?
    } else {
        date_utc_parts(time).map_or(f64::NAN, |parts| parts.2 as f64)
    };
    let Some(parts) = date_utc_parts(time) else {
        return Ok(Value::Number(f64::NAN));
    };
    date_set_time_value(
        cx,
        this,
        make_date(
            make_day(parts.0 as f64, month, date),
            date_time_within_day(time),
        ),
    )
}

fn date_proto_set_utc_full_year(
    cx: &mut Context,
    this: Value,
    args: &[Value],
) -> Completion<Value> {
    let time = date_this_time_value(cx, this.clone())?;
    let year = to_number_value(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    let basis = if time.is_nan() { 0.0 } else { time };
    let parts = date_utc_parts(basis).expect("finite date basis must have UTC parts");
    let month = if let Some(value) = args.get(1) {
        to_number_value(cx, value.clone())?
    } else {
        parts.1 as f64 - 1.0
    };
    let date = if let Some(value) = args.get(2) {
        to_number_value(cx, value.clone())?
    } else {
        parts.2 as f64
    };
    date_set_time_value(
        cx,
        this,
        make_date(make_day(year, month, date), date_time_within_day(basis)),
    )
}

fn simple_regexp_find(source: &str, flags: &str, input: &str) -> Option<(usize, usize)> {
    let ignore_case = flags.contains('i');
    let anchored_start = source.starts_with('^');
    let anchored_end = source.ends_with('$') && !source.ends_with("\\$");
    let core_start = if anchored_start { 1 } else { 0 };
    let core_end = if anchored_end && source.len() > core_start {
        source.len() - 1
    } else {
        source.len()
    };
    let pattern = &source[core_start..core_end];
    if pattern.is_empty() {
        return Some((0, 0));
    }
    if pattern == ".*" {
        return Some((0, input.len()));
    }
    let haystack = if ignore_case {
        input.to_ascii_lowercase()
    } else {
        input.to_owned()
    };
    let needle = if ignore_case {
        unescape_simple_regexp(pattern).to_ascii_lowercase()
    } else {
        unescape_simple_regexp(pattern)
    };
    if pattern == "." {
        return input.chars().next().map(|ch| (0, ch.len_utf8()));
    }
    if pattern.contains('.') {
        return find_dot_pattern(&haystack, &needle, anchored_start, anchored_end);
    }
    if anchored_start && anchored_end {
        if haystack == needle {
            Some((0, input.len()))
        } else {
            None
        }
    } else if anchored_start {
        haystack.starts_with(&needle).then_some((0, needle.len()))
    } else if anchored_end {
        haystack
            .ends_with(&needle)
            .then_some((input.len().saturating_sub(needle.len()), input.len()))
    } else {
        haystack
            .find(&needle)
            .map(|index| (index, index + needle.len()))
    }
}

fn unescape_simple_regexp(pattern: &str) -> String {
    let mut result = String::new();
    let mut escaped = false;
    for ch in pattern.chars() {
        if escaped {
            result.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            result.push(ch);
        }
    }
    if escaped {
        result.push('\\');
    }
    result
}

fn find_dot_pattern(
    haystack: &str,
    pattern: &str,
    anchored_start: bool,
    anchored_end: bool,
) -> Option<(usize, usize)> {
    let pattern_chars: Vec<_> = pattern.chars().collect();
    let starts: Vec<usize> = if anchored_start {
        vec![0]
    } else {
        haystack.char_indices().map(|(index, _)| index).collect()
    };
    for start in starts {
        let slice = &haystack[start..];
        let mut offset = 0;
        let mut ok = true;
        for expected in &pattern_chars {
            let Some(actual) = slice[offset..].chars().next() else {
                ok = false;
                break;
            };
            if *expected != '.' && *expected != actual {
                ok = false;
                break;
            }
            offset += actual.len_utf8();
        }
        let end = start + offset;
        if ok && (!anchored_end || end == haystack.len()) {
            return Some((start, end));
        }
    }
    None
}

fn error_proto_to_string(cx: &mut Context, this: Value, _args: &[Value]) -> Completion<Value> {
    let Value::Object(object) = this else {
        return Err(JsError::type_error(
            "Error.prototype.toString receiver must be object",
        ));
    };
    let name = object.get(cx, &PropertyKey::from("name"), Value::Object(object))?;
    let message = object.get(cx, &PropertyKey::from("message"), Value::Object(object))?;
    let name = if matches!(name, Value::Undefined) {
        "Error".to_owned()
    } else {
        to_string_value(cx, name)?
    };
    let message = if matches!(message, Value::Undefined) {
        String::new()
    } else {
        to_string_value(cx, message)?
    };
    match (name.is_empty(), message.is_empty()) {
        (true, true) => Ok(Value::String(String::new())),
        (true, false) => Ok(Value::String(message)),
        (false, true) => Ok(Value::String(name)),
        (false, false) => Ok(Value::String(format!("{name}: {message}"))),
    }
}

fn primitive_this_value(cx: &mut Context, value: Value) -> Completion<Value> {
    match value {
        Value::Boolean(_) | Value::Number(_) | Value::String(_) | Value::Symbol(_) => Ok(value),
        Value::Object(object) => primitive_wrapper_value(cx, object)?
            .ok_or_else(|| JsError::type_error("receiver has no primitive wrapper slot")),
        _ => Err(JsError::type_error("receiver is not a primitive wrapper")),
    }
}

fn this_string_value(cx: &mut Context, value: Value) -> Completion<String> {
    if matches!(value, Value::Undefined | Value::Null) {
        Err(JsError::type_error(
            "cannot convert undefined or null to object",
        ))
    } else {
        to_string_value(cx, value)
    }
}

fn this_number_value(cx: &mut Context, value: Value) -> Completion<f64> {
    match primitive_this_value(cx, value)? {
        Value::Number(value) => Ok(value),
        _ => Err(JsError::type_error(
            "Number.prototype receiver is not number",
        )),
    }
}

fn number_format_digits(cx: &mut Context, value: Value, default: usize) -> Completion<usize> {
    if matches!(value, Value::Undefined) {
        return Ok(default);
    }
    let digits = to_number_value(cx, value)?;
    if digits.is_nan() {
        return Ok(0);
    }
    let digits = digits.trunc();
    if !(0.0..=100.0).contains(&digits) {
        return Err(JsError::range_error("fraction digits out of range"));
    }
    Ok(digits as usize)
}

fn normalize_exponent_text(mut text: String) -> String {
    if let Some(index) = text.find('e') {
        let mantissa = text[..index].trim_end_matches('0').trim_end_matches('.');
        let exponent = &text[index + 1..];
        let exponent = exponent.trim_start_matches('+');
        let exponent = exponent.trim_start_matches('0');
        let exponent = if exponent.is_empty() || exponent == "-" {
            "0"
        } else {
            exponent
        };
        text = format!("{mantissa}e{exponent}");
    }
    text
}

fn string_units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

fn string_from_units(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

fn string_index_arg(cx: &mut Context, value: Value) -> Completion<i64> {
    let number = to_number_value(cx, value)?;
    if number.is_nan() || number == 0.0 {
        return Ok(0);
    }
    if number.is_infinite() {
        return Ok(if number.is_sign_negative() {
            i64::MIN
        } else {
            i64::MAX
        });
    }
    Ok(number.trunc() as i64)
}

fn relative_string_index(index: i64, len: i64) -> i64 {
    if index < 0 {
        (len + index).max(0)
    } else {
        index.min(len)
    }
}

fn find_units(haystack: &[u16], needle: &[u16], start: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(start.min(haystack.len()));
    }
    if start > haystack.len() || needle.len() > haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|index| index + start)
}

fn rfind_units(haystack: &[u16], needle: &[u16], start: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(start.min(haystack.len()));
    }
    if needle.len() > haystack.len() {
        return None;
    }
    let max_start = start.min(haystack.len().saturating_sub(needle.len()));
    (0..=max_start)
        .rev()
        .find(|index| &haystack[*index..*index + needle.len()] == needle)
}

fn string_pad(cx: &mut Context, this: Value, args: &[Value], at_start: bool) -> Completion<Value> {
    let value = this_string_value(cx, this)?;
    let units = string_units(&value);
    let target_len = string_index_arg(cx, args.first().cloned().unwrap_or(Value::Undefined))?;
    if target_len <= units.len() as i64 {
        return Ok(Value::String(value));
    }
    let filler = match args.get(1).cloned().unwrap_or(Value::Undefined) {
        Value::Undefined => " ".to_owned(),
        value => to_string_value(cx, value)?,
    };
    if filler.is_empty() {
        return Ok(Value::String(value));
    }
    let fill_units = string_units(&filler);
    let needed = target_len as usize - units.len();
    let mut padding = Vec::with_capacity(needed);
    while padding.len() < needed {
        let take = (needed - padding.len()).min(fill_units.len());
        padding.extend_from_slice(&fill_units[..take]);
    }
    let mut out = Vec::with_capacity(target_len as usize);
    if at_start {
        out.extend_from_slice(&padding);
        out.extend_from_slice(&units);
    } else {
        out.extend_from_slice(&units);
        out.extend_from_slice(&padding);
    }
    Ok(Value::String(string_from_units(&out)))
}

fn trim_js_string(value: &str, start: bool, end: bool) -> String {
    let mut from = 0;
    let mut to = value.len();
    if start {
        for (index, ch) in value.char_indices() {
            if !is_ecmascript_whitespace(ch) {
                from = index;
                break;
            }
            from = index + ch.len_utf8();
        }
    }
    if end {
        for (index, ch) in value.char_indices().rev() {
            if !is_ecmascript_whitespace(ch) {
                to = index + ch.len_utf8();
                break;
            }
            to = index;
        }
    }
    if from > to {
        String::new()
    } else {
        value[from..to].to_owned()
    }
}

fn is_ecmascript_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\u{0009}' | '\u{000b}' | '\u{000c}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
                | '\n'
                | '\r'
    )
}

fn test262_assert(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    if truthy(args.first().cloned().unwrap_or(Value::Undefined)) {
        Ok(Value::Undefined)
    } else {
        Err(JsError::type_error("assert failed"))
    }
}

fn test262_assert_same_value(_cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let actual = args.first().cloned().unwrap_or(Value::Undefined);
    let expected = args.get(1).cloned().unwrap_or(Value::Undefined);
    if SameValue(&actual, &expected) {
        Ok(Value::Undefined)
    } else {
        Err(JsError::type_error("assert.sameValue failed"))
    }
}

fn test262_assert_not_same_value(
    _cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let actual = args.first().cloned().unwrap_or(Value::Undefined);
    let expected = args.get(1).cloned().unwrap_or(Value::Undefined);
    if !SameValue(&actual, &expected) {
        Ok(Value::Undefined)
    } else {
        Err(JsError::type_error("assert.notSameValue failed"))
    }
}

fn test262_assert_compare_array(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let left = require_object_arg(cx, args.first())?;
    let right = require_object_arg(cx, args.get(1))?;
    let left_len = length_of_array_like(cx, left)?;
    let right_len = length_of_array_like(cx, right)?;
    if left_len != right_len {
        return Ok(Value::Boolean(false));
    }
    for index in 0..left_len {
        let key = PropertyKey::array_index(index as u64);
        let left_value = left.get(cx, &key, Value::Object(left))?;
        let right_value = right.get(cx, &key, Value::Object(right))?;
        if !SameValue(&left_value, &right_value) {
            return Ok(Value::Boolean(false));
        }
    }
    Ok(Value::Boolean(true))
}

fn test262_assert_throws(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let callback = args.get(1).cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&callback)? {
        return Err(JsError::type_error(
            "assert.throws callback is not callable",
        ));
    }
    match cx.call_mut(callback, Value::Undefined, &[]) {
        Ok(_) => Err(JsError::type_error("assert.throws expected a throw")),
        Err(_) => Ok(Value::Undefined),
    }
}

fn test262_verify_property(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    if matches!(args.get(2), None | Some(Value::Undefined)) {
        if object.get_own_property(cx, &key)?.is_none() {
            return Ok(Value::Undefined);
        }
        return Err(JsError::type_error(
            "verifyProperty target property should be missing",
        ));
    }
    let expected = require_object_arg(cx, args.get(2))?;
    let Some(desc) = object.get_own_property(cx, &key)? else {
        return Err(JsError::type_error(
            "verifyProperty target property is missing",
        ));
    };

    verify_expected_field(
        cx,
        expected,
        "value",
        desc.value.clone().unwrap_or(Value::Undefined),
    )?;
    verify_expected_field(cx, expected, "writable", Value::Boolean(desc.writable()))?;
    verify_expected_field(
        cx,
        expected,
        "enumerable",
        Value::Boolean(desc.enumerable()),
    )?;
    verify_expected_field(
        cx,
        expected,
        "configurable",
        Value::Boolean(desc.configurable()),
    )?;
    Ok(Value::Undefined)
}

fn test262_verify_equal_to(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let expected = args.get(2).cloned().unwrap_or(Value::Undefined);
    let actual = object.get(cx, &key, Value::Object(object))?;
    if SameValue(&actual, &expected) {
        Ok(Value::Undefined)
    } else {
        Err(JsError::type_error("verifyEqualTo failed"))
    }
}

fn test262_verify_writable(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    verify_descriptor_flag(cx, args, "writable", true)
}

fn test262_verify_not_writable(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    verify_descriptor_flag(cx, args, "writable", false)
}

fn test262_verify_enumerable(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    verify_descriptor_flag(cx, args, "enumerable", true)
}

fn test262_verify_not_enumerable(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    verify_descriptor_flag(cx, args, "enumerable", false)
}

fn test262_verify_configurable(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    verify_descriptor_flag(cx, args, "configurable", true)
}

fn test262_verify_not_configurable(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    verify_descriptor_flag(cx, args, "configurable", false)
}

fn verify_descriptor_flag(
    cx: &mut Context,
    args: &[Value],
    flag: &str,
    expected: bool,
) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let key = to_property_key_arg(cx, args.get(1))?;
    let Some(desc) = object.get_own_property(cx, &key)? else {
        return Err(JsError::type_error(
            "verification target property is missing",
        ));
    };
    if flag == "writable" {
        if let Some(verify_key_value) = args.get(2) {
            if !matches!(verify_key_value, Value::Undefined) {
                let verify_key = ArgView::new(verify_key_value.clone()).to_property_key(cx)?;
                let sentinel = Value::Number(8675309.0);
                let _ = object.set(cx, key.clone(), sentinel.clone(), Value::Object(object))?;
                let observed = object.get(cx, &verify_key, Value::Object(object))?;
                let wrote = SameValue(&observed, &sentinel);
                if wrote == expected {
                    return Ok(Value::Undefined);
                }
                return Err(JsError::type_error(format!(
                    "descriptor flag {flag} mismatch"
                )));
            }
        }
    }
    let actual = match flag {
        "writable" => desc.writable(),
        "enumerable" => desc.enumerable(),
        "configurable" => desc.configurable(),
        _ => return Err(JsError::internal("unknown descriptor verification flag")),
    };
    if actual == expected {
        Ok(Value::Undefined)
    } else {
        Err(JsError::type_error(format!(
            "descriptor flag {flag} mismatch"
        )))
    }
}

fn test262_is_constructor(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let Some(Value::Object(object)) = args.first() else {
        return Ok(Value::Boolean(false));
    };
    let is_constructor = matches!(
        &cx.heap().get(*object)?.kind,
        super::ObjectKind::Function(data) if data.constructible
    );
    Ok(Value::Boolean(is_constructor))
}

fn test262_check_sequence(cx: &mut Context, _this: Value, args: &[Value]) -> Completion<Value> {
    let object = require_object_arg(cx, args.first())?;
    let len = length_of_array_like(cx, object)?;
    for index in 0..len {
        let actual = object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?;
        if !SameValue(&actual, &Value::Number((index + 1) as f64)) {
            return Err(JsError::type_error("checkSequence failed"));
        }
    }
    Ok(Value::Undefined)
}

fn test262_verify_primordial_callable_property(
    cx: &mut Context,
    _this: Value,
    args: &[Value],
) -> Completion<Value> {
    let value = args.first().cloned().unwrap_or(Value::Undefined);
    if !cx.is_callable(&value)? {
        return Err(JsError::type_error(
            "verifyPrimordialCallableProperty target is not callable",
        ));
    }
    let Value::Object(object) = value else {
        return Err(JsError::type_error(
            "verifyPrimordialCallableProperty target is not object",
        ));
    };
    let expected_name = args.get(1).cloned().unwrap_or(Value::Undefined);
    let expected_length = args.get(2).cloned().unwrap_or(Value::Undefined);
    let name = object.get(cx, &PropertyKey::from("name"), Value::Object(object))?;
    let length = object.get(cx, &PropertyKey::from("length"), Value::Object(object))?;
    if !SameValue(&name, &expected_name) {
        return Err(JsError::type_error(
            "verifyPrimordialCallableProperty name mismatch",
        ));
    }
    if !SameValue(&length, &expected_length) {
        return Err(JsError::type_error(
            "verifyPrimordialCallableProperty length mismatch",
        ));
    }
    for key in ["name", "length"] {
        let Some(desc) = object.get_own_property(cx, &PropertyKey::from(key))? else {
            return Err(JsError::type_error(
                "verifyPrimordialCallableProperty descriptor missing",
            ));
        };
        if desc.writable() || desc.enumerable() || !desc.configurable() {
            return Err(JsError::type_error(
                "verifyPrimordialCallableProperty descriptor mismatch",
            ));
        }
    }
    Ok(Value::Undefined)
}

fn verify_expected_field(
    cx: &mut Context,
    expected: ObjectRef,
    name: &str,
    actual: Value,
) -> Completion<()> {
    let key = PropertyKey::from(name);
    if expected.get_own_property(cx, &key)?.is_none() {
        return Ok(());
    }
    let expected_value = expected.get(cx, &key, Value::Object(expected))?;
    if SameValue(&actual, &expected_value) {
        Ok(())
    } else {
        Err(JsError::type_error(format!(
            "verifyProperty field {name} mismatch"
        )))
    }
}

fn truthy(value: Value) -> bool {
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

fn define_properties(cx: &mut Context, target: ObjectRef, properties: Value) -> Completion<()> {
    let props = ArgView::new(properties).to_object(cx)?;

    let mut descriptors = Vec::new();
    for key in props.own_property_keys(cx)? {
        let Some(prop_desc) = props.get_own_property(cx, &key)? else {
            continue;
        };
        if prop_desc.enumerable() {
            let desc_obj = props.get(cx, &key, Value::Object(props))?;
            descriptors.push((key, ToPropertyDescriptor(cx, desc_obj)?));
        }
    }

    for (key, desc) in descriptors {
        target.define_own_property_or_throw(cx, key, desc)?;
    }
    Ok(())
}

fn create_array_like(cx: &mut Context, values: Vec<Value>) -> Completion<Value> {
    create_array_like_value(cx, values).map(Value::Object)
}

fn create_array_like_value(cx: &mut Context, values: Vec<Value>) -> Completion<ObjectRef> {
    CreateArrayFromList(cx, values)
}

fn collect_array_like(cx: &mut Context, value: Value) -> Completion<Vec<Value>> {
    if matches!(value, Value::Undefined | Value::Null) {
        return Ok(Vec::new());
    }
    let object = ArgView::new(value).to_object(cx)?;
    let len = length_of_array_like(cx, object)?;
    let mut values = Vec::new();
    for index in 0..len {
        values.push(object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?);
    }
    Ok(values)
}

fn create_list_from_array_like(cx: &mut Context, value: Value) -> Completion<Vec<Value>> {
    let Value::Object(object) = value else {
        return Err(JsError::type_error("argumentsList must be an object"));
    };
    let len = length_of_array_like(cx, object)?;
    let mut values = Vec::new();
    for index in 0..len {
        values.push(object.get(
            cx,
            &PropertyKey::array_index(index as u64),
            Value::Object(object),
        )?);
    }
    Ok(values)
}

fn collect_iterable_or_array_like(cx: &mut Context, value: Value) -> Completion<Vec<Value>> {
    if matches!(value, Value::Undefined | Value::Null) {
        return Ok(Vec::new());
    }
    let object = ArgView::new(value.clone()).to_object(cx)?;
    let iterator_method =
        object.get(cx, &PropertyKey::Symbol(SYMBOL_ITERATOR_ID), value.clone())?;
    if matches!(iterator_method, Value::Undefined | Value::Null) {
        return collect_array_like(cx, value);
    }
    if !cx.is_callable(&iterator_method)? {
        return Err(JsError::type_error("@@iterator is not callable"));
    }

    let record = get_iterator(cx, value)?;
    let mut values = Vec::new();
    while let Some(value) = iterator_step_value(cx, &record)? {
        values.push(value);
    }
    Ok(values)
}

fn to_number_value(cx: &mut Context, value: Value) -> Completion<f64> {
    ToNumber(cx, value)
}

fn to_string_value(cx: &mut Context, value: Value) -> Completion<String> {
    ToString(cx, value)
}

fn length_of_array_like(cx: &mut Context, object: ObjectRef) -> Completion<u32> {
    LengthOfArrayLike(cx, object)
}

fn length_of_array_like_f64(cx: &mut Context, object: ObjectRef) -> Completion<f64> {
    let value = object.get(cx, &PropertyKey::from("length"), Value::Object(object))?;
    let integer = to_number_value(cx, value)?;
    if integer.is_nan() || integer <= 0.0 {
        Ok(0.0)
    } else if integer.is_infinite() {
        Ok(9_007_199_254_740_991.0)
    } else {
        Ok(integer.trunc().min(9_007_199_254_740_991.0))
    }
}

fn require_object_value(cx: &mut Context, value: Value) -> Completion<ObjectRef> {
    ArgView::new(value).to_object(cx)
}

fn array_relative_index_f64(
    cx: &mut Context,
    value: Value,
    len: f64,
    default: u64,
) -> Completion<f64> {
    if matches!(value, Value::Undefined) {
        return Ok(default as f64);
    }
    let raw = to_number_value(cx, value)?;
    if raw.is_nan() {
        return Ok(0.0);
    }
    let integer = if raw.is_infinite() { raw } else { raw.trunc() };
    if integer < 0.0 {
        Ok((len + integer).max(0.0))
    } else {
        Ok(integer.min(len))
    }
}

fn forward_search_start_f64(cx: &mut Context, value: Value, len: f64) -> Completion<f64> {
    if matches!(value, Value::Undefined) {
        return Ok(0.0);
    }
    let raw = to_number_value(cx, value)?;
    if raw.is_nan() || raw == f64::NEG_INFINITY {
        return Ok(0.0);
    }
    if raw == f64::INFINITY {
        return Ok(len);
    }
    let integer = raw.trunc();
    if integer < 0.0 {
        Ok((len + integer).max(0.0))
    } else {
        Ok(integer.min(len))
    }
}

fn reverse_search_start_f64(
    cx: &mut Context,
    value: Option<Value>,
    len: f64,
) -> Completion<Option<f64>> {
    let Some(value) = value else {
        return Ok(Some(len - 1.0));
    };
    let raw = to_number_value(cx, value)?;
    if raw.is_nan() {
        return Ok(Some(0.0));
    }
    if raw == f64::INFINITY {
        return Ok(Some(len - 1.0));
    }
    if raw == f64::NEG_INFINITY {
        return Ok(None);
    }
    let integer = raw.trunc();
    if integer < 0.0 {
        let index = len + integer;
        if index < 0.0 {
            Ok(None)
        } else {
            Ok(Some(index))
        }
    } else {
        Ok(Some(integer.min(len - 1.0)))
    }
}

fn bound_function_length(
    cx: &mut Context,
    value: &Value,
    bound_arg_count: usize,
) -> Completion<f64> {
    let Value::Object(object) = value else {
        return Ok(0.0);
    };
    if object
        .get_own_property(cx, &PropertyKey::from("length"))?
        .is_none()
    {
        return Ok(0.0);
    }
    let value = object.get(cx, &PropertyKey::from("length"), Value::Object(*object))?;
    let Value::Number(target_len) = value else {
        return Ok(0.0);
    };
    if target_len.is_nan() || target_len <= 0.0 {
        return Ok(0.0);
    }
    if target_len == f64::INFINITY {
        return Ok(f64::INFINITY);
    }
    let integer_len = target_len.trunc();
    Ok((integer_len - bound_arg_count as f64).max(0.0))
}

fn bound_function_target_name(cx: &mut Context, value: &Value) -> Completion<String> {
    let Value::Object(object) = value else {
        return Ok(String::new());
    };
    let value = object.get(cx, &PropertyKey::from("name"), Value::Object(*object))?;
    match value {
        Value::String(value) => Ok(value),
        _ => Ok(String::new()),
    }
}

fn function_data_length(length: f64) -> u32 {
    if !length.is_finite() || length >= u32::MAX as f64 {
        u32::MAX
    } else {
        length as u32
    }
}

fn set_integrity(cx: &mut Context, object: ObjectRef, frozen: bool) -> Completion<()> {
    let keys = object.own_property_keys(cx)?;
    for key in keys {
        let Some(mut desc) = object.get_own_property(cx, &key)? else {
            continue;
        };
        desc.configurable = Some(false);
        if frozen && desc.is_data_descriptor() {
            desc.writable = Some(false);
        }
        object.define_own_property_or_throw(cx, key, desc)?;
    }
    let data = cx.heap_mut().get_mut(object)?;
    data.extensible = false;
    data.sealed = true;
    if frozen {
        data.frozen = true;
    }
    Ok(())
}

fn require_object_arg(cx: &mut Context, value: Option<&Value>) -> Completion<ObjectRef> {
    ArgView::new(value.cloned().unwrap_or(Value::Undefined)).to_object(cx)
}

fn require_actual_object_arg(value: Option<&Value>) -> Completion<ObjectRef> {
    match value {
        Some(Value::Object(object)) => Ok(*object),
        _ => Err(JsError::type_error("argument must be an object")),
    }
}

fn to_property_key_arg(cx: &mut Context, value: Option<&Value>) -> Completion<PropertyKey> {
    ArgView::new(value.cloned().unwrap_or(Value::Undefined)).to_property_key(cx)
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

struct NativeErrorMetadata {
    name: &'static str,
    constructor: ObjectRef,
    prototype: ObjectRef,
}

fn define_native_error_properties(
    heap: &mut Heap,
    error_ctor: ObjectRef,
    native_errors: &[NativeErrorMetadata],
) {
    for metadata in native_errors {
        if let Ok(constructor) = heap.get_mut(metadata.constructor) {
            constructor.prototype = Some(error_ctor);
        }
        define_data(
            heap,
            metadata.constructor,
            "prototype",
            Value::Object(metadata.prototype),
            false,
        );
        define_data_with_attrs(
            heap,
            metadata.prototype,
            "constructor",
            Value::Object(metadata.constructor),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            heap,
            metadata.prototype,
            "name",
            Value::String(metadata.name.to_owned()),
            true,
            false,
            true,
        );
        define_data_with_attrs(
            heap,
            metadata.prototype,
            "message",
            Value::String(String::new()),
            true,
            false,
            true,
        );
    }
}

fn define_data(heap: &mut Heap, object: ObjectRef, name: &str, value: Value, writable: bool) {
    define_data_with_attrs(heap, object, name, value, writable, false, false);
}

fn define_number_constants(heap: &mut Heap, number_ctor: ObjectRef) {
    for (name, value) in [
        ("MAX_VALUE", f64::MAX),
        ("MIN_VALUE", f64::MIN_POSITIVE),
        ("MAX_SAFE_INTEGER", 9_007_199_254_740_991.0),
        ("MIN_SAFE_INTEGER", -9_007_199_254_740_991.0),
        ("EPSILON", f64::EPSILON),
        ("NaN", f64::NAN),
        ("NEGATIVE_INFINITY", f64::NEG_INFINITY),
        ("POSITIVE_INFINITY", f64::INFINITY),
    ] {
        define_data_with_attrs(
            heap,
            number_ctor,
            name,
            Value::Number(value),
            false,
            false,
            false,
        );
    }
}

fn define_data_with_attrs(
    heap: &mut Heap,
    object: ObjectRef,
    name: impl Into<PropertyKey>,
    value: Value,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) {
    let key = name.into();
    define_data_key_with_attrs(heap, object, key, value, writable, enumerable, configurable);
}

fn define_data_key_with_attrs(
    heap: &mut Heap,
    object: ObjectRef,
    key: PropertyKey,
    value: Value,
    writable: bool,
    enumerable: bool,
    configurable: bool,
) {
    let target = heap.get_mut(object).expect("intrinsic target should exist");
    let is_new = !target.properties.contains_key(&key);
    target.properties.insert(
        key.clone(),
        Descriptor::data(value, writable, enumerable, configurable),
    );
    if is_new {
        target.property_order.push(key);
    }
}

fn define_accessor_key_with_attrs(
    heap: &mut Heap,
    object: ObjectRef,
    key: PropertyKey,
    get: Option<Value>,
    set: Option<Value>,
    enumerable: bool,
    configurable: bool,
) {
    let target = heap.get_mut(object).expect("intrinsic target should exist");
    let is_new = !target.properties.contains_key(&key);
    target.properties.insert(
        key.clone(),
        Descriptor::accessor(get, set, enumerable, configurable),
    );
    if is_new {
        target.property_order.push(key);
    }
}
