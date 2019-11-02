mod store;

pub use store::state_getter;
pub use store::use_state;
pub use store::StateAccess;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
