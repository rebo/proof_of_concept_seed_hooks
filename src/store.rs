use anymap::any::Any;
use once_cell::sync::Lazy;
use slotmap::{DefaultKey, Key, SecondaryMap, SlotMap};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use topo::*;

use std::marker::PhantomData;

#[derive(Clone)]
pub struct StateAccess<T> {
    pub id: topo::Id,
    _phantom_data: PhantomData<T>,
}

impl<T> StateAccess<T>
where
    T: Send + Sync + 'static + Clone,
{
    pub fn new(id: topo::Id) -> StateAccess<T> {
        StateAccess {
            id,
            _phantom_data: PhantomData,
        }
    }

    pub fn set(&self, value: T) {
        set_state_with_topo_id(value, self.id);
    }

    pub fn get(&self) -> Option<T> {
        get_state_with_topo_id::<T>(self.id)
    }
}

#[derive(Default, Debug)]
pub struct Store {
    pub id_to_key_map: HashMap<topo::Id, DefaultKey>,
    pub primary_slotmap: SlotMap<DefaultKey, Id>,
    pub anymap: anymap::Map<dyn Any + Sync + Send>,
}

pub static STORE: Lazy<Mutex<Store>> = Lazy::new(|| Mutex::new(Store::new()));

pub fn clone_state<T: Send + Sync + 'static + Clone>() -> Option<T> {
    STORE.lock().unwrap().get_state::<T>().cloned()
}

pub fn set_state_with_topo_id<T: Send + Sync + 'static + Clone>(data: T, current_id: topo::Id) {
    STORE
        .lock()
        .unwrap()
        .set_state_with_topo_id::<T>(data, current_id);
}

pub fn get_state_with_topo_id<T: Send + Sync + 'static + Clone>(current_id: topo::Id) -> Option<T> {
    STORE
        .lock()
        .unwrap()
        .get_state_with_topo_id::<T>(current_id)
        .cloned()
}

pub fn use_state<T: Send + Sync + 'static + Clone, F: Fn() -> T>(
    data_fn: F,
) -> (T, StateAccess<T>) {
    let current_id = topo::Id::current();
    // log!(current_id);
    if let Some(stored_data) = clone_state::<T>() {
        (stored_data, StateAccess::new(current_id))
    } else {
        let data = data_fn();
        set_state_with_topo_id::<T>(data.clone(), current_id);
        (data, StateAccess::new(current_id))
    }
}

pub fn state_getter<T: Send + Sync + 'static + Clone>() -> Arc<dyn Fn() -> Option<T>> {
    let current_id = topo::Id::current();
    Arc::new(move || get_state_with_topo_id::<T>(current_id))
}

// }
// pub fn get_state<T: Send + Sync + 'static + Clone>() -> Option<T> {
//     STORE.lock().unwrap().get_state::<T>().cloned();
// }

impl Store {
    // Constructor
    pub fn new() -> Store {
        Store {
            id_to_key_map: HashMap::new(),
            primary_slotmap: SlotMap::new(),
            anymap: anymap::Map::new(),
        }
    }

    pub fn get_state<T: Send + Sync + 'static>(&self) -> Option<&T> {
        let current_id = topo::Id::current();

        match (
            self.id_to_key_map.get(&current_id),
            self.get_secondarymap::<T>(),
        ) {
            (Some(existing_key), Some(existing_secondary_map)) => {
                existing_secondary_map.get(*existing_key)
            }
            (_, _) => None,
        }
    }

    pub fn set_state<T: Send + Sync + 'static>(&mut self, data: T) {
        let current_id = topo::Id::current();
        self.set_state_with_topo_id::<T>(data, current_id);
    }

    pub fn get_state_with_topo_id<T: Send + Sync + 'static>(
        &mut self,
        current_id: topo::Id,
    ) -> Option<&T> {
        let key = self
            .id_to_key_map
            .get(&current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            None
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.get(key)
        } else {
            None
        }
    }

    pub fn set_state_with_topo_id<T: Send + Sync + 'static>(
        &mut self,
        data: T,
        current_id: topo::Id,
    ) {
        //unwrap or default to keep borrow checker happy
        let key = self
            .id_to_key_map
            .get(&current_id)
            .copied()
            .unwrap_or_default();

        if key.is_null() {
            let key = self.primary_slotmap.insert(current_id);
            self.id_to_key_map.insert(current_id, key);
            if let Some(sec_map) = self.get_mut_secondarymap::<T>() {
                sec_map.insert(key, data);
            } else {
                self.register_secondarymap::<T>();
                self.get_mut_secondarymap::<T>().unwrap().insert(key, data);
            }
        } else if let Some(existing_secondary_map) = self.get_mut_secondarymap::<T>() {
            existing_secondary_map.insert(key, data);
        } else {
            self.register_secondarymap::<T>();
            self.get_mut_secondarymap::<T>().unwrap().insert(key, data);
        }
    }

    fn get_secondarymap<T: Send + Sync + 'static>(&self) -> Option<&SecondaryMap<DefaultKey, T>> {
        self.anymap.get::<SecondaryMap<DefaultKey, T>>()
    }

    fn get_mut_secondarymap<T: Send + Sync + 'static>(
        &mut self,
    ) -> Option<&mut SecondaryMap<DefaultKey, T>> {
        self.anymap.get_mut::<SecondaryMap<DefaultKey, T>>()
    }

    pub fn register_secondarymap<T: Send + Sync + 'static>(&mut self) {
        let sm: SecondaryMap<DefaultKey, T> = SecondaryMap::new();
        self.anymap.insert(sm);
    }
}
