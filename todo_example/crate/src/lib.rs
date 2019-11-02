#![allow(clippy::used_underscore_binding)]
#![allow(clippy::non_ascii_literal)]
#![allow(clippy::enum_glob_use)]

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

pub fn init(_url: Url, _orders: &mut impl Orders<Msg>) -> Init<Model> {
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
}

pub fn update(msg: Msg, _model: &mut Model, _orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::DoNothing => {}
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
    topo::root!(todo::masterview())
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

    App::build(init, update, view).routes(routes).finish().run();

    log!("App started.");
}
