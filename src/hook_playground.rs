use super::Msg;
use crate::store::*;
use seed::prelude::*;
// use std::sync::Arc;
// use wasm_bindgen::JsCast;
// use wasm_bindgen_futures;
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{Request, RequestInit, RequestMode, Response};

mod form_state;
use form_state::UpdateElLocal;

#[topo::nested]
fn hook_style_button() -> Node<Msg> {
    // Declare a new state variable which we'll call "count"
    let (get_count, set_count) = use_state(0);
    let count = get_count();
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
fn hook_style_input() -> Node<Msg> {
    let (get_string, set_string) = use_state("".to_string());
    let mut input_string = get_string();

    if input_string == "Seed" {
        input_string = "is pretty cool!".to_string();
    }
    div![
        "Try typing 'Seed'",
        input![
            attrs! {At::Type => "text", At::Value => input_string},
            input_ev("input", move |text| {
                set_string(text);
                Msg::DoNothing
            })
        ]
    ]
}

#[topo::nested]
pub fn complex_form_test() -> Node<Msg> {
    let (form_state, ctl) = form_state::use_form_state_builder::<Msg>().build();
    div![
        div![
            input![ctl
                .text("description")
                .validate_with(|value| {
                    if !value.contains("seed") {
                        log!("Input validation failed for field 'description!'");
                        Err("This field must contain the word 'seed'".to_string())
                    } else {
                        Ok(())
                    }
                })
                // .validate_on_blur()
                .render()],
            ctl.input_errors_for("description"),
        ],
        div![
            input![ctl.text("name").render()],
            ctl.input_errors_for("name"),
        ],
        div![
            input![ctl
                .text("email")
                .on_blur(|value| {
                    log!("form input element lost focus (i.e. blur event)");
                    log!(format!("-> {}", value));
                })
                .render()],
            ctl.input_errors_for("email"),
        ],
    ]
}

pub fn view() -> Node<Msg> {
    div![
        complex_form_test!(),
        // hook_style_button!(),
        // hook_style_button!(),
        // hook_style_button!(),
        // hook_style_button!(),
        // hook_style_input!(),
        // hook_style_input!(),
        // hook_style_input!(),
    ]
}
