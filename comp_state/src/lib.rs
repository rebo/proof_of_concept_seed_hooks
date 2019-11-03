mod store;

pub use store::state_getter;
pub use store::StateAccess;
pub use store::{clone_state, set_state, use_state};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
