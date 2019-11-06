mod store;

pub use store::state_getter;
pub use store::StateAccess;
pub use store::Store;
pub use store::{
    clone_state, get_state_with_topo_id, set_state, set_state_with_topo_id,
    update_state_with_topo_id, use_state,
};
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
