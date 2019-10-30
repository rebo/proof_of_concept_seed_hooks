use crate::store::*;
use seed::prelude::*;

pub type SharedAccess<T> = (StateAccess<(StateAccess<T>, StateAccess<T>)>);

pub trait OtherState<T> {
    fn get_right_state(&self) -> Option<T>;
    fn get_left_state(&self) -> Option<T>;
    fn set_right_state(&self, value: T);
    fn set_left_state(&self, value: T);
}

impl<T> OtherState<T> for SharedAccess<T>
where
    T: Send + Sync + 'static + Clone,
{
    fn get_right_state(&self) -> Option<T> {
        self.get().unwrap().1.get()
    }

    fn set_right_state(&self, value: T) {
        self.get().unwrap().1.set(value);
    }

    fn get_left_state(&self) -> Option<T> {
        self.get().unwrap().0.get()
    }

    fn set_left_state(&self, value: T) {
        self.get().unwrap().0.set(value);
    }
}

fn use_shared_access<T: Send + Sync + 'static + Clone>() -> SharedAccess<T> {
    use_state(|| {
        let id = topo::Id::current();
        (StateAccess::new(id), StateAccess::new(id))
    })
    .1
}

pub fn use_two_way<
    T: Send + Sync + 'static + Clone,
    F: Fn(SharedAccess<T>) -> Node<Ms>,
    G: Fn(SharedAccess<T>) -> Node<Ms>,
    Ms,
>(
    default: T,
    func1: F,
    func2: G,
) -> (Node<Ms>, Node<Ms>) {
    topo::call!({
        // let _ = use_state(|| default.clone());
        let shared_access = use_shared_access::<T>();
        let clone_default = default.clone();
        let (first_component_access, view1) = topo::call!({
            let (_, context_access) = use_state(|| clone_default);
            // we can pass the shared access to func via the closure
            (context_access, func1(shared_access.clone()))
        });
        let (second_component_access, view2) = topo::call!({
            let (_, context_access) = use_state(|| default);
            (context_access, func2(shared_access.clone()))
        });

        shared_access.set((first_component_access, second_component_access));
        (view1, view2)
    })
}
