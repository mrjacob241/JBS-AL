use super::{Completion, JsError, JsObject, ObjectRef};

#[derive(Default)]
pub struct Heap {
    objects: Vec<JsObject>,
}

impl Heap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn allocate(&mut self, object: JsObject) -> ObjectRef {
        let id = ObjectRef(self.objects.len());
        self.objects.push(object);
        id
    }

    pub fn get(&self, object: ObjectRef) -> Completion<&JsObject> {
        self.objects
            .get(object.0)
            .ok_or_else(|| JsError::internal(format!("invalid object ref {}", object.0)))
    }

    pub fn get_mut(&mut self, object: ObjectRef) -> Completion<&mut JsObject> {
        self.objects
            .get_mut(object.0)
            .ok_or_else(|| JsError::internal(format!("invalid object ref {}", object.0)))
    }
}
