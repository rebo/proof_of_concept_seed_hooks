#[macro_use]
extern crate seed;
use seed::prelude::*;
use serde::Serialize;
mod hook_playground;

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
#[derive(Clone, Serialize)]
pub enum Msg {
    Increment,
    DoNothing,
}

impl Default for Msg {
    fn default() -> Msg {
        Msg::DoNothing
    }
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.val += 1,
        Msg::DoNothing => {}
    }
}

// View
fn view(model: &Model) -> impl View<Msg> {
    topo::root!({
        div![
            hook_playground::view(),
            button![
                simple_ev(Ev::Click, Msg::Increment),
                format!("Traditional Msg Button × {}", model.val)
            ],
        ]
    })
}

fn init(_url: Url, orders: &mut impl Orders<Msg>) -> Init<Model> {
    seed_comp_helpers::init::<Msg, Model, _>(orders);
    Init::new(Model::default())
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(init, update, view).build_and_start();
}
