use anymap::any::Any;
use slotmap::{DefaultKey, Key, SecondaryMap, SlotMap};
use std::cell::RefCell;
use std::collections::HashMap;
pub use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Arc;
use topo::*;

#[derive(Clone, Debug)]
pub struct StateAccess<T> {
    pub id: topo::Id,
    _phantom_data: PhantomData<T>,
}

impl<T> StateAccess<T>
where
    T: 'static + Clone,
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

    pub fn update<F: FnOnce(&mut T) -> ()>(&self, func: F) {
        let item = &mut self.get().unwrap();
        func(item);
        self.set(item.clone());
    }

    pub fn get(&self) -> Option<T> {
        get_state_with_topo_id::<T>(self.id)
    }

    pub fn hard_get(&self) -> T {
        get_state_with_topo_id::<T>(self.id).unwrap()
    }
}

pub fn clone_state<T: 'static + Clone>() -> Option<T> {
    let store = topo::Env::get::<RefCell<Store>>();
    store.unwrap().borrow_mut().get_state::<T>().cloned()
}

pub fn init_root_context() {
    if topo::Env::get::<RefCell<Store>>().is_none() {
        topo::Env::add(RefCell::new(Store::default()));
    }
}

pub fn set_state<T: 'static + Clone>(data: T) {
    let current_id = topo::Id::current();
    assert!(topo::Env::get::<RefCell<Store>>().is_some());

    let store = topo::Env::get::<RefCell<Store>>();
    store
        .unwrap()
        .borrow_mut()
        .set_state_with_topo_id::<T>(data, current_id);
}

pub fn set_state_with_topo_id<T: 'static + Clone>(data: T, current_id: topo::Id) {
    let store = topo::Env::get::<RefCell<Store>>();
    store
        .unwrap()
        .borrow_mut()
        .set_state_with_topo_id::<T>(data, current_id);
}

pub fn get_state_with_topo_id<T: 'static + Clone>(id: topo::Id) -> Option<T> {
    let store = topo::Env::get::<RefCell<Store>>();
    store
        .unwrap()
        .borrow_mut()
        .get_state_with_topo_id::<T>(id)
        .cloned()
}

pub fn update_state_with_topo_id<T: Clone + 'static, F: FnOnce(&mut T) -> ()>(
    id: topo::Id,
    func: F,
) {
    let item = &mut get_state_with_topo_id::<T>(id).unwrap();
    func(item);
    set_state_with_topo_id(item.clone(), id);
}
//

pub fn reset_unseen_id_list() {
    let store = topo::Env::get::<RefCell<Store>>();
    let store = store.unwrap();
    let mut store_mut = store.borrow_mut();

    store_mut.unseen_ids = HashSet::new();
    let ids = store_mut.id_to_key_map.keys().cloned().collect::<Vec<_>>();
    for id in ids {
        store_mut.unseen_ids.insert(id);
    }
}

// pub fn current_unseen_id_list() -> HashSet<topo::Id> {
//     let store = topo::Env::get::<RefCell<Store>>();
//     let store = store.unwrap();
//     let store_mut = store.borrow();

//     store_mut.unseen_ids.clone()
// }

fn purge_unseen_ids() {
    let store = topo::Env::get::<RefCell<Store>>();
    let store = store.unwrap();
    let mut store_mut = store.borrow_mut();

    let ids = store_mut.unseen_ids.iter().cloned().collect::<Vec<_>>();

    for id in ids {
        let key = store_mut.id_to_key_map.remove(&id);
        if let Some(key) = key {
            store_mut.primary_slotmap.remove(key);
        }
    }
}

pub fn purge_and_reset_unseen_ids() {
    purge_unseen_ids();
    reset_unseen_id_list();
}

pub fn use_state<T: 'static + Clone, F: FnOnce() -> T>(data_fn: F) -> (T, StateAccess<T>) {
    let current_id = topo::Id::current();
    if let Some(stored_data) = clone_state::<T>() {
        (stored_data, StateAccess::new(current_id))
    } else {
        let data = data_fn();
        set_state_with_topo_id::<T>(data.clone(), current_id);
        // get the current topo id context and add the current id
        (data, StateAccess::new(current_id))
    }
}

#[derive(Default, Debug)]
pub struct Store {
    pub id_to_key_map: HashMap<topo::Id, DefaultKey>,
    pub primary_slotmap: SlotMap<DefaultKey, Id>,
    pub anymap: anymap::Map<dyn Any>,
    pub unseen_ids: HashSet<topo::Id>,
}

impl Store {
    // Constructor
    pub fn new() -> Store {
        Store {
            id_to_key_map: HashMap::new(),
            primary_slotmap: SlotMap::new(),
            anymap: anymap::Map::new(),
            unseen_ids: HashSet::new(),
        }
    }

    pub fn get_state<T: 'static>(&mut self) -> Option<&T> {
        let current_id = topo::Id::current();

        self.get_state_with_topo_id(current_id)
    }

    pub fn get_state_with_topo_id<T: 'static>(&mut self, current_id: topo::Id) -> Option<&T> {
        self.unseen_ids.remove(&current_id);
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

    pub fn remove_topo_id(&mut self, id: topo::Id) {
        let key = self.id_to_key_map.get(&id).copied().unwrap_or_default();
        if !key.is_null() {
            self.primary_slotmap.remove(key);
            self.id_to_key_map.remove(&id);
        }
    }

    pub fn set_state_with_topo_id<T: 'static>(&mut self, data: T, current_id: topo::Id) {
        self.unseen_ids.remove(&current_id);

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

    fn get_secondarymap<T: 'static>(&self) -> Option<&SecondaryMap<DefaultKey, T>> {
        self.anymap.get::<SecondaryMap<DefaultKey, T>>()
    }

    fn get_mut_secondarymap<T: 'static>(&mut self) -> Option<&mut SecondaryMap<DefaultKey, T>> {
        self.anymap.get_mut::<SecondaryMap<DefaultKey, T>>()
    }

    pub fn register_secondarymap<T: 'static>(&mut self) {
        let sm: SecondaryMap<DefaultKey, T> = SecondaryMap::new();
        self.anymap.insert(sm);
    }
}

pub fn state_getter<T: 'static + Clone>() -> Arc<dyn Fn() -> Option<T>> {
    let current_id = topo::Id::current();
    Arc::new(move || get_state_with_topo_id::<T>(current_id))
}
