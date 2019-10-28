use super::Msg;
use crate::store::*;
use seed::prelude::*;
// use std::sync::Arc;
// use wasm_bindgen::JsCast;
// use wasm_bindgen_futures;
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{Request, RequestInit, RequestMode, Response};

mod form_state;
use form_state::*;

#[topo::nested]
fn hook_style_button() -> Node<Msg> {
    // Declare a new state variable which we'll call "count"
    let (count, set_count) = use_state(0);
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
    let (mut input_string, set_string) = use_state("".to_string());

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

// A very simple form that only renders form elements and keeps track of state
// It does however output the form state below the form though
#[topo::nested]
pub fn very_simple_form_test() -> Node<Msg> {
    let (form_state, ctl) = form_state::use_form_state::<Msg>();
    div![
        div![input![ctl.text("username").render()],],
        div![input![ctl.password("pword").render()],],
        div![span![format!("{:#?}", form_state)]],
    ]
}

// A Simple form that doens't have any custom callbacks
// But rather relies on prebaked methods
#[topo::nested]
pub fn simple_form_test() -> Node<Msg> {
    let (_form_state, ctl) = form_state::use_form_state::<Msg>();
    div![
        div![
            label!["description"],
            input![ctl
                .text("description")
                .required()
                .default_value("I love Seed!")
                .render()],
            ctl.input_errors_for("description"),
        ],
        div![
            label!["password"],
            input![ctl
                .password("password")
                .required()
                .letters_num_and_special_required()
                .render()],
            ctl.input_errors_for("password"),
        ],
        div![
            label!["email"],
            input![ctl
                .text("email")
                .required()
                .validate_on_blur_only()
                .render()],
            ctl.input_errors_for("email"),
        ],
    ]
}

// A complex form demonstrating a formwide on blur callback
// As well as  custom validation
// and error outputs
#[topo::nested]
pub fn complex_form_test() -> Node<Msg> {
    let (_form_state, ctl) = form_state::use_form_state_builder::<Msg>()
        .on_blur(|form_state| {
            log!("Outputing the form state on blur due to the #on_blur closure");
            log!(form_state);
        })
        .build();

    div![
        div![
            label!["description"],
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
                .render()],
            ctl.input_errors_for("description"),
        ],
        div![
            label!["name"],
            input![ctl.text("name").render()],
            ctl.input_errors_for("name"),
        ],
        div![
            label!["email"],
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
        span!["Very Simple Form"],
        very_simple_form_test!(),
        span!["Simple Form"],
        simple_form_test!(),
        span!["Complex Form"],
        complex_form_test!(),
        hook_style_button!(),
        hook_style_button!(),
        hook_style_input!(),
    ]
}
