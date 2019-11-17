#![allow(clippy::map_clone)]

// use crate::{StateAccess, Store};
// use std::cell::RefCell;
// use crate::{get_state_with_topo_id, set_state_with_topo_id, update_state_with_topo_id, use_state};
use crate::use_state;
use std::any::Any;
use std::collections::HashSet;
use std::ops::Deref;
// sets immutable state on the current context and for all children.
pub fn set_context<T: 'static>(context: T) {
    // topo::Env::get::<RefCell<Store>>().is_none() {
    topo::Env::add(context);
}

pub fn get_context<E>() -> Option<impl Deref<Target = E> + 'static>
where
    E: Any + 'static,
{
    topo::Env::get::<E>()
}

#[derive(Clone, Default)]
pub struct ContextIds {
    ids: HashSet<topo::Id>,
}

#[derive(Clone)]
pub struct TopoIdMemo(pub topo::Id);

#[derive(Clone)]
pub struct TopoContext {
    id: topo::Id,
    child_ids: Vec<topo::Id>,
}

// retreives the parents id (as long as it has been memoized) and sets the current parent.
// This is useful for child parent communication.
pub fn use_parent_memo() -> Option<TopoIdMemo> {
    // if there is already a parent_id do nothing
    // as it has already been stored.context
    let (parent_id, _parent_id_accesor) =
        use_state(|| topo::Env::get::<TopoIdMemo>().map(|d| d.clone()));
    topo::Env::add(TopoIdMemo(topo::Id::current()));
    parent_id
}

pub fn do_once<F: Fn() -> ()>(func: F) {
    topo::call!({
        let (has_done, has_done_access) = use_state(|| false);
        if !has_done {
            func();
            has_done_access.set(true);
        }
    });
}
