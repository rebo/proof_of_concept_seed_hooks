use crate::{use_state, StateAccess};

pub fn use_list<F, T>(initial_list_fn: F) -> (List<T>, ListControl<T>)
where
    F: FnOnce() -> Vec<T>,
    T: Clone,
{
    let (list, list_access) = use_state(|| List::new(initial_list_fn()));

    (list, ListControl::new(list_access))
}

#[derive(Clone)]
pub struct ListControl<T>
where
    T: Clone + 'static,
{
    list_access: StateAccess<List<T>>,
}

impl<T> ListControl<T>
where
    T: Clone + 'static,
{
    fn new(list_access: StateAccess<List<T>>) -> ListControl<T> {
        ListControl { list_access }
    }

    pub fn get_list(&self) -> List<T> {
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

#[derive(Clone)]
pub struct List<T>
where
    T: Clone + 'static,
{
    pub items: Vec<T>,
}

impl<T> List<T>
where
    T: Clone + 'static,
{
    fn new(items: Vec<T>) -> List<T> {
        List { items }
    }
}
