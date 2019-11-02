use clone_all::clone_all;
use comp_state::{use_state, StateAccess};
use seed::prelude::*;

pub fn basic_render<T, Ms>(list_control: ListControl<T, Ms>) -> Node<Ms>
where
    T: Into<String> + Send + Sync + 'static + Clone,
    Ms: Clone + Send + Sync + 'static,
{
    let list = list_control.list_access.get().unwrap();
    div![ul![list
        .items
        .iter()
        .cloned()
        .enumerate()
        .map(|(idx, item)| {
            let item_s: String = item.into();
            li![
                span![item_s],
                {
                    clone_all!(list, list_control);
                    button![
                        "UP",
                        input_ev("click", move |_| {
                            list_control.move_item_up(idx);
                            list.list_updated_msg
                        },)
                    ]
                },
                {
                    clone_all!(list, list_control);
                    button![
                        "DOWN",
                        input_ev("click", move |_| {
                            list_control.move_item_down(idx);
                            list.list_updated_msg
                        },)
                    ]
                }
            ]
        })
        .collect::<Vec<Node<Ms>>>()]]
}

pub fn use_list<F, T, Ms>(
    initial_list_fn: F,
    list_updated_msg: Ms,
) -> (List<T, Ms>, ListControl<T, Ms>)
where
    F: FnOnce() -> Vec<T>,
    T: Send + Sync + 'static + Clone,
    Ms: Clone + Send + Sync + 'static,
{
    let (list, list_access) = use_state(|| List::new(initial_list_fn(), list_updated_msg));

    (list, ListControl::new(list_access))
}

#[derive(Clone)]
pub struct ListControl<T, Ms>
where
    T: Clone + Send + Sync + 'static,
    Ms: Clone + Send + Sync + 'static,
{
    list_access: StateAccess<List<T, Ms>>,
}

impl<T, Ms> ListControl<T, Ms>
where
    T: Clone + Send + Sync + 'static,
    Ms: Clone + Send + Sync + 'static,
{
    fn new(list_access: StateAccess<List<T, Ms>>) -> ListControl<T, Ms> {
        ListControl { list_access }
    }

    pub fn get_list(&self) -> List<T, Ms> {
        self.list_access.get().unwrap()
    }

    // brain always gets this messed up so I have to write it down!
    // 0 1 2 3 4 5 6
    // a b c d e f g

    // I want to move c after d (which should be remove 2 put in 3)
    // remove(2)

    // 0 1 2 3 4 5 6
    // a b d e f g

    // insert(3)

    // 0 1 2 3 4 5 6
    // a b d e f g
    //
    //
    // 0 1 2 3 4 5 6
    // a b c d e f g

    // I want to move f after d (which should be remove 5 put in 4)
    // remove(2)

    // 0 1 2 3 4 5 6
    // a b d e f g

    // insert(3)

    // 0 1 2 3 4 5 6
    // a b d e f g
    pub fn move_item_to_position(&self, old_idx: usize, new_idx: usize) {
        let mut list = self.list_access.get().unwrap();
        if new_idx > list.items.len() || old_idx > list.items.len() - 1 {
            return;
        }

        let old_item = list.items.remove(old_idx);
        use std::cmp::Ordering;
        match old_idx.cmp(&new_idx) {
            Ordering::Less => list.items.insert(new_idx - 1, old_item),
            Ordering::Greater => list.items.insert(new_idx, old_item),
            Ordering::Equal => {}
        }
        self.list_access.set(list);
    }

    pub fn move_item_up(&self, old_idx: usize) {
        if old_idx == 0 {
            return;
        }
        self.move_item_to_position(old_idx, old_idx - 1);
    }

    pub fn move_item_down(&self, old_idx: usize) {
        self.move_item_to_position(old_idx, old_idx + 2);
    }

    pub fn insert(&self, idx: usize, item: T) {
        let mut list = self.list_access.get().unwrap();
        list.items.insert(idx, item);
        self.list_access.set(list);
    }

    pub fn remove(&self, idx: usize) -> T {
        let mut list = self.list_access.get().unwrap();
        let obj = list.items.remove(idx);
        self.list_access.set(list);
        obj
    }

    pub fn replace(&self, idx: usize, item: T) -> T {
        let mut list = self.list_access.get().unwrap();
        list.items.insert(idx, item);
        let obj = list.items.remove(idx + 1);
        self.list_access.set(list);
        obj
    }

    pub fn push(&self, item: T) {
        let mut list = self.list_access.get().unwrap();
        list.items.push(item);
        self.list_access.set(list);
    }
}

#[derive(Clone, Debug)]
pub struct List<T, Ms>
where
    T: Clone + Send + Sync + 'static,
    Ms: Clone + Send + Sync + 'static,
{
    pub items: Vec<T>,
    pub list_updated_msg: Ms,
}

impl<T, Ms> List<T, Ms>
where
    T: Clone + Send + Sync + 'static,
    Ms: Clone + Send + Sync + 'static,
{
    fn new(items: Vec<T>, list_updated_msg: Ms) -> List<T, Ms> {
        List {
            items,
            list_updated_msg,
        }
    }
}
