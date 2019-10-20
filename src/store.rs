use once_cell::sync::Lazy;
use std::sync::Mutex;

use anymap::any::Any;
use slotmap::{DefaultKey, Key, SecondaryMap, SlotMap};
use std::collections::HashMap;
use topo::*;

#[derive(Default)]
pub struct Store {
    pub id_to_key_map: HashMap<topo::Id, DefaultKey>,
    pub primary_slotmap: SlotMap<DefaultKey, Id>,
    pub anymap: anymap::Map<dyn Any + Sync + Send>,
}

pub static STORE: Lazy<Mutex<Store>> = Lazy::new(|| Mutex::new(Store::new()));

pub fn clone_state<T: Send + Sync + 'static + Clone>() -> Option<T> {
    STORE.lock().unwrap().get_state::<T>().cloned()
}

pub fn set_state_with_topo_id<T: Send + Sync + 'static>(data: T, current_id: topo::Id) {
    STORE
        .lock()
        .unwrap()
        .set_state_with_topo_id::<T>(data, current_id);
}

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