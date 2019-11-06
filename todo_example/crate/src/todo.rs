#![allow(non_snake_case)]
use super::{Model, Msg};
use crate::generated::css_classes::C;
use comp_state::{set_state, use_state};
use list::ListControl;
use seed::dom_types::UpdateEl;
use seed::{prelude::*, *};
use seed_comp_helpers::list;
use seed_comp_helpers::use_fetch_helper::use_fetch;
use seed_comp_helpers::use_fetch_helper::{UseFetchStatus, UseFetchStatusTrait};

use serde::Deserialize;
#[derive(Clone)]
struct Item {
    description: String,
    status: Status,
}

impl Item {
    fn new<T: Into<String>>(description: T) -> Item {
        Item {
            description: description.into(),
            status: Status::Todo,
        }
    }
}

#[derive(Clone, PartialEq)]
enum Status {
    Todo,
    Completed,
}

#[derive(Clone, Default)]
struct ItemState {
    adding: String,
    editing: String,
    editing_idx: Option<usize>,
}

pub fn masterview(tasks: &[&str]) -> Node<Msg> {
    let tasks = tasks.iter().cloned().map(Item::new).collect::<Vec<Item>>();

    // gives the block inside its only execution context.
    topo::call!({
        // list control lets you interact with items in the list
        let (_list, list_control) = list::use_list(|| tasks, Msg::DoNothing);
        // within this component allow only global access to the list control
        set_state(list_control);
        div![render_list(), list_controls(),]
    })
}

fn render_list() -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item, Msg>>().unwrap();
    let list = list_control.get_list();
    div![ul![
        class![
            C.p_3,
            C.bg_gray_9,
            C.border_solid,
            C.border_gray_4,
            C.m_4,
            C.rounded,
        ],
        list.items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                li![
                    class![
                        C.rounded,
                        C.bg_gray_1,
                        C.p_3,
                        C.border_solid,
                        C.border_gray_4,
                        C.m_4,
                        C.shadow,
                        C.flex,
                        C.flex_row,
                    ],
                    if item.status == Status::Completed {
                        completed_item_view(idx, item)
                    } else {
                        item_view(idx, item)
                    },
                    move_up_button(idx),
                    move_down_button(idx),
                ]
            })
            .collect::<Vec<Node<Msg>>>()
    ]]
}

fn move_up_button(idx: usize) -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item, Msg>>().unwrap();
    let list = list_control.get_list();
    if idx != 0 {
        i![
            class!["fas fa-arrow-up", C.cursor_pointer, C.flex_none, C.mr_4],
            mouse_ev("click", move |_| {
                list_control.move_item_up(idx);
                list.list_updated_msg
            },)
        ]
    } else {
        i![class!["fas fa-stop", C.cursor_pointer, C.flex_none, C.mr_4],]
    }
}

fn move_down_button(idx: usize) -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item, Msg>>().unwrap();
    let list = list_control.get_list();
    if idx != list.items.len() - 1 {
        i![
            class!["fas fa-arrow-down", C.cursor_pointer, C.flex_none, C.mr_4],
            mouse_ev("click", move |_| {
                list_control.move_item_down(idx);
                list.list_updated_msg
            },)
        ]
    } else {
        i![class!["fas fa-stop", C.cursor_pointer, C.flex_none, C.mr_4],]
    }
}

fn completed_item_view(idx: usize, item: &Item) -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item, Msg>>().unwrap();
    let list = list_control.get_list();
    span![
        class![C.flex_1],
        i![class!["far fa-check-circle", C.cursor_pointer, C.mr_4], {
            mouse_ev("click", move |_| {
                let mut item = list.items[idx].clone();
                item.status = Status::Todo;
                list_control.replace(idx, item);
                list.list_updated_msg
            })
        },],
        del![
            class![C.mr_2],
            format!("{} ) {}", idx + 1, item.description)
        ],
    ]
}
fn item_view(idx: usize, item: &Item) -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item, Msg>>().unwrap();
    let list = list_control.get_list();
    span![
        class![C.flex_1],
        i![
            class![
                "far fa-check-circle",
                C.cursor_pointer,
                C.flex_none,
                C.text_gray_3,
                C.mr_4
            ],
            {
                mouse_ev("click", move |_| {
                    let mut item = list.items[idx].clone();
                    item.status = Status::Completed;
                    list_control.replace(idx, item);
                    list.list_updated_msg
                })
            },
        ],
        span![
            class![C.flex_1, C.mr_2],
            format!("{} ) {}", idx + 1, item.description)
        ]
    ]
}

fn list_controls() -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item, Msg>>().unwrap();
    let (item_state, item_state_access) = use_state(ItemState::default);
    div![
        label!["Add Task"],
        {
            let is_access_clone = item_state_access.clone();
            input![
                attrs![At::Value => item_state.adding],
                input_ev("input", move |text| {
                    is_access_clone.update(|item_state| item_state.adding = text);
                    Msg::DoNothing
                })
            ]
        },
        button![
            i![class![
                "fas fa-plus-square",
                C.cursor_pointer,
                C.flex_none,
                C.mr_4
            ]],
            {
                mouse_ev("click", move |_| {
                    list_control.push(Item::new(item_state.adding));
                    item_state_access.set(ItemState::default());
                    Msg::DoNothing
                })
            }
        ],
        fetch_todo(),
    ]
}

#[derive(Clone, Debug, Deserialize)]
struct Todo {
    userId: u32,
    id: u32,
    title: String,
    completed: bool,
}

fn fetch_todo() -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item, Msg>>().unwrap();
    let (fetched, fetch_control) = use_fetch::<Todo>(
        "https://jsonplaceholder.typicode.com/todos/1".to_string(),
        Method::Get,
    );

    div![
        button![class![C.p_4, C.bg_gray_5, C.m_4], "Dispatch Json", {
            let fetch_control = fetch_control.clone();
            mouse_ev(Ev::Click, move |_ev| {
                fetch_control.dispatch::<Msg, Model>();
                Msg::DoNothing
            })
        }],
        match fetch_control.status() {
            UseFetchStatus::Initialized => "Initialized (Ready to Dispatch)".to_string(),
            UseFetchStatus::Loading => "Loading...".to_string(),
            UseFetchStatus::Complete => {
                let (has_done_once, once_access) = use_state(|| false);
                if !has_done_once {
                    list_control.push(Item {
                        status: Status::Todo,
                        description: format!("{:#?}", fetched.clone().unwrap()),
                    });
                    seed_comp_helpers::schedule_update::<_, Model>(Msg::DoNothing);
                    once_access.set(true);
                }
                format!("Downloaded Task: {}", fetched.unwrap().title)
            }
            UseFetchStatus::Failed => "Failed!".to_string(),
        }
    ]
}

// fn on_off_toggle() -> (bool, Node<Msg>) {
//     topo::call!({
//         let (state, state_access) = use_state(|| false);
//         (
//             state,
//             div![
//                 class![C.w_2, C.cursor_pointer],
//                 if state { div!["ON"] } else { div!["OFF"] },
//                 mouse_ev(Ev::Click, move |_| {
//                     state_access.set(!state_access.get().unwrap());
//                     Msg::DoNothing
//                 })
//             ],
//         )
//     })
// }
