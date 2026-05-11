use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use ezrs_error::{Error, Result};

#[derive(Clone, Default)]
pub(crate) struct TypeStore {
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl TypeStore {
    pub(crate) fn insert<T>(&mut self, value: T)
    where
        T: Clone + Send + Sync + 'static,
    {
        self.values.insert(TypeId::of::<T>(), Arc::new(value));
    }

    pub(crate) fn insert_arc(
        &mut self,
        type_id: TypeId,
        value: Arc<dyn Any + Send + Sync>,
    ) -> Option<Arc<dyn Any + Send + Sync>> {
        self.values.insert(type_id, value)
    }

    pub(crate) fn get<T>(&self, label: &str) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        let value = self.values.get(&TypeId::of::<T>()).ok_or_else(|| {
            Error::not_found(format!(
                "{label} value for type {}",
                std::any::type_name::<T>()
            ))
        })?;

        value.downcast_ref::<T>().cloned().ok_or_else(|| {
            Error::invalid_input(format!(
                "{label} type mismatch for {}",
                std::any::type_name::<T>()
            ))
        })
    }
}
