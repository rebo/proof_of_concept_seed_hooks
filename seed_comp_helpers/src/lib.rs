#[macro_use]
extern crate seed;
pub mod helpers;

pub use helpers::form_state;
pub use helpers::list;
pub use helpers::memo;
pub use helpers::two_way;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
