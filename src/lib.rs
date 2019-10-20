use anymap::any::Any;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

use topo::*;

#[macro_use]
extern crate seed;
use seed::prelude::*;

use slotmap::{DefaultKey, Key, SecondaryMap, SlotMap};

#[derive(Default)]
pub struct Store {
    pub id_to_key_map: HashMap<topo::Id, DefaultKey>,
    pub primary_slotmap: SlotMap<DefaultKey, Id>,
    pub anymap: anymap::Map<dyn Any + Sync + Send>,
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

    fn get_state<T: Send + Sync + 'static>(&self) -> Option<&T> {
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

    fn set_state<T: Send + Sync + 'static>(&mut self, data: T) {
        let current_id = topo::Id::current();
        self.set_state_with_specific_id::<T>(data, current_id);
    }

    fn set_state_with_specific_id<T: Send + Sync + 'static>(
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
            self.register_secondarymap::<T>();
            self.get_mut_secondarymap::<T>().unwrap().insert(key, data);
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

static STORE: Lazy<Mutex<Store>> = Lazy::new(|| {
    let mut store = Store::new();

    Mutex::new(store)
});

// Model
struct Model {
    pub val: i32,
}

impl Default for Model {
    fn default() -> Self {
        Self { val: 0 }
    }
}

// Update
#[derive(Clone)]
enum Msg {
    Increment,
    Nothing,
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.val += 1,
        Msg::Nothing => {}
    }
}

#[topo::nested]
fn hook_style_button() -> Node<Msg> {
    let store = STORE.lock().unwrap();
    let current_id = topo::Id::current();

    let button_count = store.get_state::<u32>().unwrap_or(&0).clone();
    button![
        input_ev("click", move |_| {
            STORE
                .lock()
                .unwrap()
                .set_state_with_specific_id::<u32>(button_count + 1, current_id);
            log!("shim shim");
            log!(topo::Id::current());
            log!("shim shimd");
            let button_count = STORE
                .lock()
                .unwrap()
                .get_state::<u32>()
                .cloned()
                .unwrap_or(0);
            log!(button_count);
            Msg::Nothing
        }),
        format!("Hello, World1212 × {}", button_count)
    ]
}

#[topo::nested]
fn topo_nested_view() -> Node<Msg> {
    store_or_retrieve_string_state("nested");
    div!["Press button to re-render the page and re-trigger the component calculations (Check the console)"]
}

fn store_or_retrieve_string_state<S: Into<String>>(a_string: S) {
    let a_string = a_string.into();
    let mut store = STORE.lock().unwrap();
    if let Some(my_string_state) = store.get_state::<String>().cloned() {
        log!(format!(
            "Inside callsite #{} The stored state is: {}",
            a_string.clone(),
            my_string_state.clone()
        ));

        store.set_state::<String>(format!(
            "{} -> {} has been called Again!",
            my_string_state, a_string
        ));
    } else {
        store.set_state::<String>(a_string.clone());
        log!(format!("storing fresh string inside callsite {}", a_string));
    }
}

// View
fn view(model: &Model) -> impl View<Msg> {
    log!("Regenerating View...");
    topo::root!({
        // store_or_retrieve_string_state("root");

        // topo::call!({
        //     store_or_retrieve_string_state("inner");
        // });
        div![
            topo_nested_view!(),
            hook_style_button!(),
            button![
                simple_ev(Ev::Click, Msg::Increment),
                format!("Hello, World × {}", model.val)
            ],
        ]
    })
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Model::default(), update, view)
        .finish()
        .run();
}
