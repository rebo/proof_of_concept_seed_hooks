#![allow(clippy::used_underscore_binding)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::enum_glob_use)]
use comp_state::Store;
use seed_comp_helpers::use_fetch_helper;
use seed_comp_helpers::use_fetch_helper::UseFetchMsgTrait;
use std::cell::RefCell;
mod generated;
mod todo;

use seed::{prelude::*, *};

const STATIC_PATH: &str = "static";
const IMAGES_PATH: &str = "static/images";

// ------ ------
//     Model
// ------ ------

#[derive(Default)]
pub struct Model {}

// ------ ------
//     Init
// ------ ------

// type AppType = seed::App<Msg, Model, Node<Msg>>;

pub fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Init<Model> {
    seed_comp_helpers::init::<Msg, Model, _>(orders);

    if let Some(mount_point_element) = document().get_element_by_id("app") {
        mount_point_element.set_inner_html("");
    }
    Init::new(Model::default())
}
// ------ ------
//    Routes
// ------ ------

pub fn routes(_url: Url) -> Option<Msg> {
    None
}

// ------ ------
//    Update
// ------ ------

#[derive(Clone)]
pub enum Msg {
    DoNothing,
    // below are needed for use_fetch hook
    Fetch(topo::Id, String, Method),
    Fetched(topo::Id, String),
}

// Needed for use_fetch hook to work. links enum method interface to specific variants.
impl UseFetchMsgTrait for Msg {
    fn fetch_message(id: topo::Id, url: String, method: Method) -> Self {
        Msg::Fetch(id, url, method)
    }
    fn fetched_message(id: topo::Id, response: String) -> Self {
        Msg::Fetched(id, response)
    }
}

pub fn update(msg: Msg, _model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::DoNothing => {}
        // Below are needed to ensure use_fetch hook works.
        Msg::Fetch(id, url, method) => use_fetch_helper::update_fetch(orders, id, url, method),
        Msg::Fetched(id, string_response) => {
            use_fetch_helper::update_fetched(id, string_response);
        }
    }
}

// ------ ------
//     View
// ------ ------

// Notes:
// - \u{00A0} is the non-breaking space
//   - https://codepoints.net/U+00A0
//
// - "▶\u{fe0e}" - \u{fe0e} is the variation selector, it prevents ▶ to change to emoji in some browsers
//   - https://codepoints.net/U+FE0E

pub fn view(_model: &Model) -> impl View<Msg> {
    // One advantage of state stored in components is that
    // One can simply repeatedly 'render' the view
    // and each view will have its own state that just "works"
    topo::root!(
        div![
            h1!["Household Chores"],
            todo::masterview(&[
                "Feed the cat",
                "Do the Washing",
                "Mow the lawn",
                "Buy Flowers"
            ]),
            h1!["Business Tasks"],
            todo::masterview(&[
                "Complete purchase order",
                "File paperwork",
                "Issue Invoices",
                "Write report"
            ]),
            h1!["TV Shows to Watch"],
            todo::masterview(&[
                "Watch DARK",
                "Watch The Expanse",
                "Watch Patriot",
                "Watch Mr Robot"
            ]),
        ] // env! {RefCell<Store> => ),}
    )
}

pub fn image_src(image: &str) -> String {
    format!("{}/{}", IMAGES_PATH, image)
}

pub fn asset_path(asset: &str) -> String {
    format!("{}/{}", STATIC_PATH, asset)
}

// ------ ------
//     Start
// ------ ------

#[wasm_bindgen(start)]
pub fn run() {
    log!("Starting app...");
    if topo::Env::get::<RefCell<Store>>().is_none() {
        topo::Env::add(RefCell::new(Store::default()));
    }
    App::build(init, update, view).routes(routes).finish().run();

    log!("App started.");
}
