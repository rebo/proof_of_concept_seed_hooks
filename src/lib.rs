use anymap::any::Any;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

use topo::*;

#[macro_use]
extern crate seed;
use seed::prelude::*;

use slotmap::{DefaultKey, SecondaryMap, SlotMap};

static GLOBAL_DATA: Lazy<Mutex<HashMap<topo::Id, DefaultKey>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static GLOBAL_DATA_SM: Lazy<Mutex<SlotMap<DefaultKey, Id>>> =
    Lazy::new(|| Mutex::new(SlotMap::new()));

// struct State(String);

#[derive(Default)]
pub struct Store {
    pub anymap: anymap::Map<dyn Any + Sync + Send>,
}

impl Store {
    // Constructor
    pub fn new() -> Store {
        Store {
            anymap: anymap::Map::new(),
        }
    }
    fn get_secondarymap<T: Send + Sync + 'static>(&self) -> &SecondaryMap<DefaultKey, T> {
        self.anymap
            .get::<SecondaryMap<DefaultKey, T>>()
            .expect("Entity Slotmap doesn't exist")
    }

    fn get_mut_secondarymap<T: Send + Sync + 'static>(
        &mut self,
    ) -> &mut SecondaryMap<DefaultKey, T> {
        self.anymap
            .get_mut::<SecondaryMap<DefaultKey, T>>()
            .expect("Entity Slotmap doesn't exist")
    }

    pub fn register_secondarymap<T: Send + Sync + 'static>(&mut self) {
        let sm: SecondaryMap<DefaultKey, T> = SecondaryMap::new();
        self.anymap.insert(sm);
    }
}

static STORE: Lazy<Mutex<Store>> = Lazy::new(|| {
    let mut store = Store::new();
    store.register_secondarymap::<String>();

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
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.val += 1,
    }
}

#[topo::nested]
fn topo_nested_view() -> Node<Msg> {
    trial_current_global_access("inner");
    span!["Test"]
}

fn trial_current_global_access<T: Into<String>>(a_string: T) {
    let a_string = a_string.into();
    let current_id = topo::Id::current();
    if let (Ok(mut data), Ok(mut data_sm)) = (GLOBAL_DATA.lock(), GLOBAL_DATA_SM.lock()) {
        if let Some(existing_key) = data.get(&current_id) {
            let mut global_store = STORE.lock().unwrap();

            let mut string_map = global_store.get_mut_secondarymap::<String>();
            let my_string = string_map.get(*existing_key).unwrap();
            log!(my_string);

            let new_string = format!("{} - {}", my_string.clone(), my_string);
            string_map.insert(*existing_key, new_string);
        } else {
            let mut global_store = STORE.lock().unwrap();
            let key = data_sm.insert(current_id);
            data.insert(current_id, key);
            log!(format!(
                "No data in {} context, creating entry key",
                a_string
            ));
            let mut mut_string_map = global_store.get_mut_secondarymap::<String>();

            mut_string_map.insert(key, a_string);
        }
    }
}

// View
fn view(model: &Model) -> impl View<Msg> {
    log!("Regenerating View...");
    topo::root!({
        trial_current_global_access("root");

        topo::call!({
            trial_current_global_access("call site alpha");
        });
        div![
            topo_nested_view!(),
            button![
                simple_ev(Ev::Click, Msg::Increment),
                format!("Hello, World × {}", model.val)
            ]
        ]
    })
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Model::default(), update, view)
        .finish()
        .run();
}
