use super::Msg;
use clone_all::clone_all;
use comp_state::{use_state, StateAccess};
use seed::prelude::*;
use std::sync::Arc;
// use wasm_bindgen::JsCast;
// use wasm_bindgen_futures;
// use wasm_bindgen_futures::JsFuture;
// use web_sys::{Request, RequestInit, RequestMode, Response};

use comp_state::{use_list, use_memo};
use seed_comp_helpers::form_state::{use_form_state, use_form_state_builder, UpdateElLocal};
use seed_comp_helpers::two_way::*;

#[topo::nested]
fn memoize_example() -> Node<Msg> {
    // memoize executes a closure on first run and stores the result
    // it also returns a recalc_trigger which will re run the block
    // if the recalc trigger is passed true.
    // this is useful for expensive calls that you only want to be recalculated on demand
    // it would be good to memoize entire Note<Msg> trees howeever trait constraints does
    // allow for this
    let (date_time, memo_ctl) = use_memo(false, || {
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
    // However inside use_memo is run inside its own call context
    // therefore everything should be ok

    let (other_string, other_memo_ctl) = use_memo(false, || {
        let date = js_sys::Date::new_0();
        format!("milliseconds only: {}", date.get_milliseconds())
    });

    div![
        div![date_time],
        div![other_string],
        div![button![
            "Recalculate expensive js-sys closure",
            input_ev("click", move |_event| {
                memo_ctl.recalc(true);
                other_memo_ctl.recalc(true);
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

#[topo::nested]
fn simplified_two_way() -> Node<Msg> {
    let (view1, view2) = use_two_way("".to_string(), first, second);
    div![view1, view2]
}

fn first(shared: SharedAccess<String>) -> Node<Msg> {
    // let cloned_shared_channel_access = shared.clone();
    div![
        label!["First Child Component, this input will send to the second component"],
        div![format!(
            "Second Child input : {}",
            shared.get_left_state().unwrap_or_default()
        )],
        input![input_ev("input", move |text| {
            shared.set_right_state(text);
            Msg::DoNothing
        })],
    ]
}

fn second(shared: SharedAccess<String>) -> Node<Msg> {
    div![
        label!["Second Child Component, this input will send to the first component"],
        div![format!(
            "First Child input : {}",
            shared.get_right_state().unwrap_or_default()
        )],
        input![input_ev("input", move |text| {
            shared.set_left_state(text);
            Msg::DoNothing
        })],
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
    let (form_state, ctl) = use_form_state::<Msg>();
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
    let (_form_state, ctl) = use_form_state::<Msg>();
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
    let (_form_state, ctl) = use_form_state_builder::<Msg>()
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

fn pretend_modal_view() -> Node<Msg> {
    div!["THIS IS A PRETEND MODAL"]
}

fn send_view_to_back(
    portal_access: StateAccess<Arc<dyn Fn() -> Node<Msg> + Send + Sync>>,
) -> Node<Msg> {
    button![
        "render at end",
        input_ev("click", move |_| {
            portal_access.set(Arc::new(pretend_modal_view));
            Msg::DoNothing
        })
    ]
}

pub fn list_example() -> Node<Msg> {
    // use_list produces a list_control that can control the list
    // the Msg is the message to return with the list has been modified or updated
    let (list, list_control) = use_list(|| {
        vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
            "four".to_string(),
            "five".to_string(),
        ]
    });

    let (add_state, add_state_access) = use_state(|| "".to_string());
    let list_control_clone = list_control.clone();
    let add_state_access_clone = add_state_access.clone();
    div![
        div![div![ul![list
            .items
            .iter()
            .cloned()
            .enumerate()
            .map(|(idx, item)| {
                li![
                    span![item],
                    {
                        clone_all!(list_control);
                        button![
                            "UP",
                            input_ev("click", move |_| {
                                list_control.move_item_up(idx);
                                Msg::DoNothing
                            },)
                        ]
                    },
                    {
                        clone_all!(list_control);
                        button![
                            "DOWN",
                            input_ev("click", move |_| {
                                list_control.move_item_down(idx);
                                Msg::DoNothing
                            },)
                        ]
                    }
                ]
            })
            .collect::<Vec<Node<Msg>>>()]],],
        div![
            label!["Add:"],
            input![
                attrs!(At::Value => add_state),
                input_ev("input", move |text| {
                    add_state_access.set(text);
                    Msg::DoNothing
                })
            ],
            button![
                "Add",
                input_ev("click", move |_| {
                    list_control_clone.push(add_state);
                    add_state_access_clone.set("".to_string());
                    Msg::DoNothing
                })
            ]
        ]
    ]
}

pub fn view() -> Node<Msg> {
    // we can set up a 'portal' state which means i can send views to
    // the portal 'getter' from anywhere on the page.
    let default_arced_closure: Arc<dyn Fn() -> Node<Msg> + Send + Sync> = Arc::new(|| empty![]);
    let (_, portal_access) = use_state(|| default_arced_closure);

    div![
        div![h1!["Managed list Example"], list_example()],
        div![
            h1!["Sending Node<Msg> to another part of the view hierarchy Example"],
            send_view_to_back(portal_access.clone())
        ],
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
            simplified_two_way!()
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
        ],
        div![h1!["moved components to this 'portal'"]],
        if let Some(portal_content) = portal_access.get() {
            portal_content()
        } else {
            empty![]
        }
    ]
}
