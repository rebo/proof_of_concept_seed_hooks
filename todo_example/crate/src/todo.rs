#![allow(non_snake_case)]
use super::{Model, Msg};
use crate::generated::css_classes::C;
use comp_state::do_once;
use comp_state::{set_state, use_state};
use comp_state::{use_list, ListControl};
use comp_state::{use_memo, watch};
use enclose::enclose as e;
use seed::dom_types::UpdateEl;
use seed::{prelude::*, *};
use seed_comp_helpers::use_fetch_helper::use_fetch;
use seed_comp_helpers::use_fetch_helper::{UseFetchStatus, UseFetchStatusTrait};

use serde::Deserialize;
#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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
        let (_list, list_control) = use_list(|| tasks);
        // within this component allow only global access to the list control
        set_state(list_control);
        div![render_list(), list_controls(),]
    })
}

fn render_list() -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
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
        list.items()
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
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
    // let list = list_control.get_list();
    if idx != 0 {
        i![
            class!["fas fa-arrow-up", C.cursor_pointer, C.flex_none, C.mr_4],
            mouse_ev("click", move |_| {
                list_control.move_item_up(idx);
                Msg::DoNothing
            },)
        ]
    } else {
        i![class!["fas fa-stop", C.cursor_pointer, C.flex_none, C.mr_4],]
    }
}

fn move_down_button(idx: usize) -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
    let list = list_control.get_list();
    if idx != list.items().count() - 1 {
        i![
            class!["fas fa-arrow-down", C.cursor_pointer, C.flex_none, C.mr_4],
            mouse_ev("click", move |_| {
                list_control.move_item_down(idx);
                Msg::DoNothing
            },)
        ]
    } else {
        i![class!["fas fa-stop", C.cursor_pointer, C.flex_none, C.mr_4],]
    }
}

// this function shows an example of using memoization on Node<Msgs>
fn completed_item_view(idx: usize, item: &Item) -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
    let (item, idx) = (watch(item), watch(&idx));
    let (nodes, _memo_ctl) = use_memo(item.changed || idx.changed, || {
        // If using use_memo you need to ensure anything that can change is updated via a function
        // and not just used in the closure becuase it gets captured on first run.
        let list = list_control.get_list();
        let (item, idx) = (item.hard_get(), idx.hard_get());
        //
        //

        span![
            class![C.flex_1],
            i![class!["far fa-check-circle", C.cursor_pointer, C.mr_4], {
                mouse_ev(
                    "click",
                    e!((list_control) move |_| {
                        let mut item = list.items().nth(idx).unwrap().clone();
                        item.status = Status::Todo;
                        list_control.replace(idx, item);
                        Msg::DoNothing
                    }),
                )
            },],
            del![
                class![C.mr_2],
                format!("{} ) {}", idx + 1, item.description)
            ],
        ]
    });
    nodes
}
fn item_view(idx: usize, item: &Item) -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
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
                    let mut item = list.items().nth(idx).unwrap().clone();
                    item.status = Status::Completed;
                    list_control.replace(idx, item);
                    Msg::DoNothing
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
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
    let (item_state, item_state_access) = use_state(ItemState::default);
    div![
        label!["Add Task"],
        {
            input![
                attrs![At::Value => item_state.adding],
                input_ev(
                    "input",
                    e!( (item_state_access) move |text| {
                        item_state_access.update(|item_state| item_state.adding = text);
                        Msg::DoNothing
                    })
                )
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
        fetch_todo_with_seed_msg_hooks(),
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
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
    let (fetched, fetch_control) = use_fetch::<Todo>(
        "https://jsonplaceholder.typicode.com/todos/1".to_string(),
        Method::Get,
    );

    div![
        button![class![C.p_4, C.bg_gray_5, C.m_4], "Dispatch Json", {
            mouse_ev(
                Ev::Click,
                e!( (fetch_control) move |_ev| {
                    fetch_control.dispatch::<Msg, Model>();
                    Msg::DoNothing
                }),
            )
        }],
        match fetch_control.status() {
            UseFetchStatus::Initialized => "Initialized (Ready to Dispatch)".to_string(),
            UseFetchStatus::Loading => "Loading...".to_string(),
            UseFetchStatus::Complete => {
                do_once(|| {
                    list_control.push(Item {
                        status: Status::Todo,
                        description: format!("{:#?}", fetched.clone().unwrap()),
                    });
                    seed_comp_helpers::schedule_update::<_, Model>(Msg::DoNothing);
                });
                format!("Downloaded Task: {}", fetched.unwrap().title)
            }
            UseFetchStatus::Failed => "Failed!".to_string(),
        }
    ]
}

fn fetch_todo_with_seed_msg_hooks() -> Node<Msg> {
    let list_control = comp_state::clone_state::<ListControl<Item>>().unwrap();
    let (fetched, fetch_control) = use_fetch::<Todo>(
        "https://jsonplaceholder.typicode.com/todos/1".to_string(),
        Method::Get,
    );

    let (fetched2, fetch_control2) = use_fetch::<Todo>(
        "https://jsonplaceholder.typicode.com/todos/2".to_string(),
        Method::Get,
    );

    div![
        button![
            class![C.p_4, C.bg_gray_5, C.m_4],
            "Dispatch Json",
            mouse_ev(
                Ev::Click,
                e!( (fetch_control) move |_ev| {
                    fetch_control.dispatch_with_seed::<Msg, Model>();
                    Msg::DoNothing
                })
            )
        ],
        button![
            class![C.p_4, C.bg_gray_5, C.m_4],
            "Dispatch Json",
            mouse_ev(
                Ev::Click,
                e!((fetch_control2) move |_ev| {
                    fetch_control2.dispatch_with_seed::<Msg, Model>();
                    Msg::DoNothing
                })
            )
        ],
        match fetch_control.status() {
            UseFetchStatus::Initialized => "Initialized (Ready to Dispatch)".to_string(),
            UseFetchStatus::Loading => "Loading...".to_string(),
            UseFetchStatus::Complete => {
                do_once(|| {
                    list_control.push(Item {
                        status: Status::Todo,
                        description: format!("{:#?}", fetched.clone().unwrap()),
                    });
                    seed_comp_helpers::schedule_update::<_, Model>(Msg::DoNothing);
                });
                format!("Downloaded Task: {}", fetched.unwrap().title)
            }
            UseFetchStatus::Failed => "Failed!".to_string(),
        },
        match fetch_control2.status() {
            UseFetchStatus::Initialized => "Initialized (Ready to Dispatch)".to_string(),
            UseFetchStatus::Loading => "Loading...".to_string(),
            UseFetchStatus::Complete => {
                do_once(|| {
                    list_control.push(Item {
                        status: Status::Todo,
                        description: format!("{:#?}", fetched2.clone().unwrap()),
                    });
                    seed_comp_helpers::schedule_update::<_, Model>(Msg::DoNothing);
                });
                format!("Downloaded Task: {}", fetched2.unwrap().title)
            }
            UseFetchStatus::Failed => "Failed!".to_string(),
        }
    ]
}
