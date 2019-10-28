// use crate::store::*;
// use anymap::any::Any;
// // use seed::prelude::*;
use std::sync::Arc;

// pub fn memoize_value<T: Sync + Send + 'static + Clone>(value: T) -> T {
//     let mut map = anymap::Map::new();
//     map.insert::<T>(value);
//     let arc_anymap: Arc<anymap::Map<dyn Any + Sync + Send>> = Arc::new(map);
//     let (value, set_value) = use_state(arc_anymap);
//     value.get::<T>().cloned().unwrap()
// }

use crate::store::*;

pub fn use_memoize<T: Sync + Send + 'static + Clone, F: Fn() -> T>(
    func: F,
) -> (T, Arc<dyn Fn(bool)>) {
    let (update, set_recalc_trigger) = use_state(|| false);

    // let arc_func = Arc::new(func);
    let new_func = || func();

    // by definition this will keep returning 'value'
    // until update is set to true.

    let (value, set_value) = use_state(new_func);

    if update {
        let value = func();
        set_value(value.clone());
        set_recalc_trigger(false);
        (value, set_recalc_trigger)
    } else {
        (value, set_recalc_trigger)
    }
}
