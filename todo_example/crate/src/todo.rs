use super::Msg;
use crate::generated::css_classes::C;
use clone_all::clone_all;
use comp_state::{use_state, StateAccess};
use seed::dom_types::UpdateEl;
use seed::{prelude::*, *};
use seed_comp_helpers::list;

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

pub fn masterview() -> Node<Msg> {
    let (_list, list_control) = list::use_list(
        || {
            vec![
                Item::new("Do the washing up"),
                Item::new("Walk the dog"),
                Item::new("Pay Bills"),
                Item::new("Do the shopping"),
            ]
        },
        Msg::DoNothing,
    );
    div![
        render_list(list_control.clone()),
        list_controls(list_control),
    ]
}

fn render_list(list_control: list::ListControl<Item, Msg>) -> Node<Msg> {
    let list = list_control.get_list();
    div![ul![
        class![
            C.p_3,
            C.bg_gray_9,
            C.border_solid,
            C.border_gray_4,
            C.m_4,
            C.shadow_lg
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
                        C.shadow_lg
                    ],
                    if item.status == Status::Completed {
                        span![
                            i![
                                class![
                                    "far fa-check-circle",
                                    C.cursor_pointer,
                                    C.flex_none,
                                    C.mr_4
                                ],
                                {
                                    clone_all!(list, list_control);
                                    input_ev("click", move |_| {
                                        let mut item = list.items[idx].clone();
                                        item.status = Status::Todo;
                                        list_control.replace(idx, item);
                                        list.list_updated_msg
                                    })
                                },
                            ],
                            del![format!("{} ) {}", idx + 1, item.description)],
                        ]
                    } else {
                        span![
                            i![
                                class![
                                    "far fa-check-circle",
                                    C.cursor_pointer,
                                    C.flex_none,
                                    C.text_gray_3,
                                    C.mr_4
                                ],
                                {
                                    clone_all!(list, list_control);
                                    input_ev("click", move |_| {
                                        let mut item = list.items[idx].clone();
                                        item.status = Status::Completed;
                                        list_control.replace(idx, item);
                                        list.list_updated_msg
                                    })
                                },
                            ],
                            format!("{} ) {}", idx + 1, item.description)
                        ]
                    },
                    {
                        clone_all!(list, list_control);
                        if idx > 0 {
                            button![
                                i![class![
                                    "fas fa-arrow-up",
                                    C.cursor_pointer,
                                    C.flex_none,
                                    C.mr_4
                                ]],
                                input_ev("click", move |_| {
                                    list_control.move_item_up(idx);
                                    list.list_updated_msg
                                },)
                            ]
                        } else {
                            empty![]
                        }
                    },
                    {
                        clone_all!(list, list_control);
                        if idx != list.items.len() - 1 {
                            button![
                                i![class![
                                    "fas fa-arrow-down",
                                    C.cursor_pointer,
                                    C.flex_none,
                                    C.mr_4
                                ]],
                                input_ev("click", move |_| {
                                    list_control.move_item_down(idx);
                                    list.list_updated_msg
                                },)
                            ]
                        } else {
                            empty![]
                        }
                    },
                ]
            })
            .collect::<Vec<Node<Msg>>>()
    ]]
}

fn list_controls(list_control: list::ListControl<Item, Msg>) -> Node<Msg> {
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
                input_ev("click", move |_| {
                    list_control.push(Item::new(item_state.adding));
                    item_state_access.set(ItemState::default());
                    Msg::DoNothing
                })
            }
        ]
    ]
}
