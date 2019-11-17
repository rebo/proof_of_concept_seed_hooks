use crate::{use_state, StateAccess};
use slotmap::{new_key_type, DenseSlotMap, Key};

new_key_type! {
    pub struct ListKey;
}

pub fn use_list<T, F>(initial_list_fn: F) -> (List<T>, ListControl<T>)
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

    pub fn clear(&self) {
        self.list_access.update(|list| {
            list.items_map = ListKeyDenseSlotMap::new();
            list.items_order = vec![];
        });
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
        if new_idx > list.items_order.len() || old_idx > list.items_order.len() - 1 {
            return;
        }

        let old_item = list.items_order.remove(old_idx);
        use std::cmp::Ordering;
        match old_idx.cmp(&new_idx) {
            Ordering::Less => list.items_order.insert(new_idx - 1, old_item),
            Ordering::Greater => list.items_order.insert(new_idx, old_item),
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
        let inserted_key = list.items_map.0.insert(item);
        list.items_order.insert(idx, inserted_key);
        self.list_access.set(list);
    }

    pub fn remove(&self, idx: usize) -> T {
        let mut list = self.list_access.get().unwrap();
        let removed_key = list.items_order.remove(idx);
        let obj = list.items_map.0.remove(removed_key).unwrap();
        self.list_access.set(list);
        obj
    }

    pub fn replace(&self, idx: usize, item: T) -> T {
        let mut list = self.list_access.get().unwrap();
        let inserted_key = list.items_map.0.insert(item);
        list.items_order.insert(idx, inserted_key);
        let replaced_key = list.items_order.remove(idx + 1);
        let obj = list.items_map.0.remove(replaced_key).unwrap();
        self.list_access.set(list);
        obj
    }

    pub fn push(&self, item: T) {
        let mut list = self.list_access.get().unwrap();
        let pushed_key = list.items_map.0.insert(item);
        list.items_order.push(pushed_key);
        self.list_access.set(list);
    }
}

#[derive(Clone, Default)]
pub struct ListKeyDenseSlotMap<T>(DenseSlotMap<ListKey, T>);

impl<T> ListKeyDenseSlotMap<T> {
    pub fn new() -> ListKeyDenseSlotMap<T> {
        ListKeyDenseSlotMap(DenseSlotMap::<ListKey, T>::with_key())
    }
}

#[derive(Clone, PartialEq)]
pub struct List<T>
where
    T: Clone + 'static,
{
    pub items_map: ListKeyDenseSlotMap<T>,
    pub items_order: Vec<ListKey>,
    selected_key: ListKey,
}

impl<T> PartialEq for ListKeyDenseSlotMap<T>
where
    T: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        let mut self_keys = self.0.keys().collect::<Vec<ListKey>>();
        let mut other_keys = other.0.keys().collect::<Vec<ListKey>>();
        self_keys.sort();
        other_keys.sort();
        self_keys == other_keys
        // self.isbn == other.isbn
    }
}

impl<T> List<T>
where
    T: Clone + 'static,
{
    fn new(mut items: Vec<T>) -> List<T> {
        let mut sm = DenseSlotMap::default();
        for item in items.drain(..) {
            sm.insert(item);
        }
        let keys = sm.keys().collect::<Vec<_>>();
        List {
            items_map: ListKeyDenseSlotMap(sm),
            items_order: keys,
            selected_key: ListKey::null(),
        }
    }

    pub fn items(&self) -> Vec<&T> {
        self.items_order
            .iter()
            .map(move |list_key| self.items_map.0.get(*list_key).unwrap())
            .collect::<Vec<_>>()
    }

    pub fn selected(&self) -> Option<&T> {
        self.items_map.0.get(self.selected_key)
    }
}
