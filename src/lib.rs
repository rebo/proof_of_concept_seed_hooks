mod store;

#[macro_use]
extern crate seed;
use seed::prelude::*;

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
#[derive(Clone)]
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

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Model::default(), update, view)
        .finish()
        .run();
}
