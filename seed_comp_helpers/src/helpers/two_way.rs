use comp_state::{use_state, StateAccess};
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

// Above code based on below
// #[topo::nested]
// fn two_way_components_example() -> Node<Msg> {
//     // two way communication between two compoments
//     // we need an accessor that can hold two access states.
//     // after this is passed to peer compoments this is then updated with reference
//     // to all peers
//     //
//     // That said at this stage it is probably better to just use the whole traditional Msg:: stack.
//     //
//     let (_state, shared_channel_access): ((StateAccess<String>, StateAccess<String>), _) =
//         use_state(|| {
//             let id = topo::Id::current();
//             (StateAccess::new(id), StateAccess::new(id))
//         });

//     // execute first peer in a call! context returning the view and peer accessor
//     let (first_component_access, view1) = topo::call!({
//         let (_, context_access) = use_state(|| "".to_string());
//         (
//             context_access,
//             first_child_component(shared_channel_access.clone()),
//         )
//     });
//     // execute second peer in a call! context returning the view and peer accessor
//     let (second_component_access, view2) = topo::call!({
//         let (_, context_access) = use_state(|| "".to_string());
//         (
//             context_access,
//             second_child_component(shared_channel_access.clone()),
//         )
//     });

//     // set both peer accessors to the main shared accessor
//     shared_channel_access.set((first_component_access, second_component_access));
//     div![view1, view2]
// }

// fn first_child_component(shared_channel_access: SharedAccess<String>) -> Node<Msg> {
//     let cloned_shared_channel_access = shared_channel_access.clone();
//     div![
//         label!["First Child Component, this input will send to the second component"],
//         input![input_ev("input", move |text| {
//             cloned_shared_channel_access.get().unwrap().1.set(text);
//             Msg::DoNothing
//         })],
//         div![format!(
//             "Second Child input : {}",
//             shared_channel_access
//                 .get()
//                 .unwrap()
//                 .0
//                 .get()
//                 .unwrap_or_default()
//         )]
//     ]
// }

// fn second_child_component(
//     shared_channel_access: StateAccess<(StateAccess<String>, StateAccess<String>)>,
// ) -> Node<Msg> {
//     let cloned_shared_channel_access = shared_channel_access.clone();
//     div![
//         label!["First Child Component, this input will send to the second component"],
//         input![input_ev("input", move |text| {
//             cloned_shared_channel_access.get().unwrap().0.set(text);
//             Msg::DoNothing
//         })],
//         div![format!(
//             "Second Child input : {}",
//             shared_channel_access
//                 .get()
//                 .unwrap()
//                 .1
//                 .get()
//                 .unwrap_or_default()
//         )]
//     ]
// }
