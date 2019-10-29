use super::Msg;
use crate::store::*;
use seed::prelude::*;
// use std::sync::Arc;
// use wasm_bindgen::JsCast;
// use wasm_bindgen_futures;
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{Request, RequestInit, RequestMode, Response};

mod memo;

mod form_state;
use form_state::*;
use memo::*;

#[topo::nested]
fn memoize_example() -> Node<Msg> {
    // memoize executes a closure on first run and stores the result
    // it also returns a recalc_trigger which will re run the block
    // if the recalc trigger is passed true.
    // this is useful for expensive calls that you only want to be recalculated on demand
    // it would be good to memoize entire Note<Msg> trees howeever trait constraints does
    // allow for this
    let (date_time, recalc_trigger) = use_memoize(|| {
        let date = js_sys::Date::new_0();
        format!(
            "Day: {}, Month: {}, Year:{}, Hours: {}, Minutes: {}, seconds: {}, milliseconds: {}",
            date.get_date(),
            date.get_month(),
            date.get_full_year(),
            date.get_hours(),
            date.get_minutes(),
            date.get_seconds(),
            date.get_milliseconds()
        )
    });

    // Normally one issue is that only one value of one type can be memoized per
    // execution context.
    // However inside use_memoize is run inside its own call context
    // therefore everything should be ok

    let (other_string, other_recalc_trigger) = use_memoize(|| {
        let date = js_sys::Date::new_0();
        format!("milliseconds only: {}", date.get_milliseconds())
    });

    div![
        div![date_time],
        div![other_string],
        div![button![
            "Recalculate expensive js-sys closure",
            input_ev("click", move |_event| {
                recalc_trigger(true);
                other_recalc_trigger(true);
                Msg::DoNothing
            })
        ]]
    ]
}

#[topo::nested]
fn child_component_example(button_disabled_status_access: StateAccess<bool>) -> Node<Msg> {
    div![button![
        "Child Button - Triggers change in parent state",
        input_ev("click", move |_text| {
            button_disabled_status_access.set(!button_disabled_status_access.get().unwrap());
            Msg::DoNothing
        })
    ],]
}
#[topo::nested]
fn parent_and_child_components_example() -> Node<Msg> {
    // by passing the accessor to child components we can access different 'components'.

    let (button_disabled_status, button_disabled_status_access) = use_state(|| false);
    div![
        button![
            "Parent Button",
            attrs![At::Disabled => button_disabled_status.as_at_value()]
        ],
        child_component_example!(button_disabled_status_access)
    ]
}

#[topo::nested]
fn complex_child_component_example(parent_text_access: StateAccess<String>) -> Node<Msg> {
    div![
        label!["this is a child component's input:"],
        input![input_ev("input", move |text| {
            parent_text_access.set(text);
            Msg::DoNothing
        })],
    ]
}

#[topo::nested]
fn complex_parent_and_child_components_example() -> Node<Msg> {
    // by passing the accessor to child components we can access different 'components'.

    let (parent_text, parent_text_access) = use_state(|| "".to_string());
    div![
        h3![format!("text from child component: {}", parent_text)],
        complex_child_component_example!(parent_text_access)
    ]
}

// #[topo::nested]
// fn complex_child_component_example(parent_text_access: StateAccess<String>) -> Node<Msg> {
//     div![
//         label!["this is a child component's input:"],
//         input![input_ev("input", move |text| {
//             parent_text_access.set(text);
//             Msg::DoNothing
//         })],
//     ]
// }

#[topo::nested]
fn two_way_components_example() -> Node<Msg> {
    // two way communication between two compoments
    // we need an accessor that can hold two access states.
    // after this is passed to peer compoments this is then updated with reference
    // to all peers
    let (_state, shared_channel_access): ((StateAccess<String>, StateAccess<String>), _) =
        use_state(|| {
            let id = topo::Id::current();
            (StateAccess::new(id), StateAccess::new(id))
        });

    // execute first peer in a call! context returning the view and peer accessor
    let (first_component_access, view1) = topo::call!({
        let (_, context_access) = use_state(|| "".to_string());
        (
            context_access,
            first_child_component(shared_channel_access.clone()),
        )
    });
    // execute second peer in a call! context returning the view and peer accessor
    let (second_component_access, view2) = topo::call!({
        let (_, context_access) = use_state(|| "".to_string());
        (
            context_access,
            second_child_component(shared_channel_access.clone()),
        )
    });

    // set both peer accessors to the main shared accessor
    shared_channel_access.set((first_component_access, second_component_access));
    div![view1, view2]
}

fn first_child_component(
    shared_channel_access: StateAccess<(StateAccess<String>, StateAccess<String>)>,
) -> Node<Msg> {
    let cloned_shared_channel_access = shared_channel_access.clone();
    div![
        label!["First Child Component, this input will send to the second component"],
        input![input_ev("input", move |text| {
            cloned_shared_channel_access.get().unwrap().1.set(text);
            Msg::DoNothing
        })],
        div![format!(
            "Second Child input : {}",
            shared_channel_access
                .get()
                .unwrap()
                .0
                .get()
                .unwrap_or_default()
        )]
    ]
}

fn second_child_component(
    shared_channel_access: StateAccess<(StateAccess<String>, StateAccess<String>)>,
) -> Node<Msg> {
    let cloned_shared_channel_access = shared_channel_access.clone();
    div![
        label!["First Child Component, this input will send to the second component"],
        input![input_ev("input", move |text| {
            cloned_shared_channel_access.get().unwrap().0.set(text);
            Msg::DoNothing
        })],
        div![format!(
            "Second Child input : {}",
            shared_channel_access
                .get()
                .unwrap()
                .1
                .get()
                .unwrap_or_default()
        )]
    ]
}

#[topo::nested]
fn hook_style_button() -> Node<Msg> {
    // Declare a new state variable which we'll call "count"
    let (count, count_access) = use_state(|| 0);
    div![
        p![format!("You clicked {} times", count)],
        button![
            input_ev("click", move |_| {
                count_access.set(count + 1);
                Msg::DoNothing
            }),
            format!("Click Me × {}", count)
        ]
    ]
}

#[topo::nested]
fn hook_style_input() -> Node<Msg> {
    let (mut input_string, string_access) = use_state(|| "".to_string());

    if input_string == "Seed" {
        input_string = "is pretty cool!".to_string();
    }
    div![
        "Try typing 'Seed'",
        input![
            attrs! {At::Type => "text", At::Value => input_string},
            input_ev("input", move |text| {
                string_access.set(text);
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
        div![
            h1!["Parent Child Example"],
            parent_and_child_components_example!()
        ],
        div![
            h1!["Complex Parent Child Example"],
            complex_parent_and_child_components_example!()
        ],
        div![
            h1!["Two Way communcation between peers Example"],
            two_way_components_example!()
        ],
        div![h1!["Memoize Example"], memoize_example!()],
        div![
            h1!["Forms Examples"],
            div![h3!["Very Simple Form"], very_simple_form_test!(),],
            div![h3!["Simple Form"], simple_form_test!()],
            div![h3!["Complex Form"], complex_form_test!()],
        ],
        div![
            h1!["Button Examples"],
            hook_style_button!(),
            hook_style_button!(),
            hook_style_input!(),
        ]
    ]
}
