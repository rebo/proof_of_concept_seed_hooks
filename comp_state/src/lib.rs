pub mod actor;
mod context;
mod list;
mod memo;
mod store;

pub use context::do_once;
pub use context::use_parent_memo;
pub use context::{get_context, set_context};
pub use list::{use_list, ListControl};
pub use memo::{use_memo, watch};
pub use store::init_root_context;
pub use store::state_getter;
pub use store::StateAccess;
pub use store::Store;

pub use store::{
    clone_state, current_unseen_id_list, get_state_with_topo_id, purge_unseen_ids,
    reset_unseen_id_list, set_state, set_state_with_topo_id, update_state_with_topo_id, use_state,
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 3, 4);
    }
}
