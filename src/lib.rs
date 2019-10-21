mod store;
use store::*;

#[macro_use]
extern crate seed;
use seed::prelude::*;

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
    DoNothing,
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.val += 1,
        Msg::DoNothing => {}
    }
}

#[topo::nested]
fn example() -> Node<Msg> {
    // Declare a new state variable which we'll call "count"
    let (count, set_count) = store::use_state(0);

    div![
        p![format!("You clicked {} times", count)],
        button![
            input_ev("click", move |_| {
                set_count(count + 1);
                Msg::DoNothing
            }),
            format!("Click Me × {}", count)
        ]
    ]
}

#[topo::nested]
fn hook_style_button() -> Node<Msg> {
    let current_id = topo::Id::current();
    let button_count = clone_state::<u32>().unwrap_or(0);

    div![button![
        input_ev("click", move |_| {
            set_state_with_topo_id::<u32>(button_count + 1, current_id);
            Msg::DoNothing
        }),
        format!("Hook State Button × {}", button_count)
    ]]
}

#[topo::nested]
fn hook_style_input() -> Node<Msg> {
    let current_id = topo::Id::current();

    let mut input_string = clone_state::<String>().unwrap_or_else(|| "".to_string());
    if input_string == "Seed" {
        input_string = "is pretty cool!".to_string();
    }
    div![
        "Try typing 'Seed'",
        input![
            attrs! {At::Type => "text", At::Value => input_string},
            input_ev("input", move |text| {
                set_state_with_topo_id::<String>(text, current_id);
                Msg::DoNothing
            })
        ]
    ]
}

// View
fn view(model: &Model) -> impl View<Msg> {
    topo::root!({
        div![
            example!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_button!(),
            hook_style_input!(),
            hook_style_input!(),
            hook_style_input!(),
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
