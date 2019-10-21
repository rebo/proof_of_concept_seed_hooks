# EXPERIMENTAL AND PROBABLY BROKEN 
# React set state Hooks in Seed Proof of concept.

**Example:**

This is a complete counting button with state, no messages required: 

```rust
#[topo::nested]
fn example() -> Node<Msg> {
    // Declare a new state variable which we'll call "count"
    let (count, set_count) = store::use_state::<u32>(0);

    div![
        p![format!("You clicked {} times", count)],
        button![
            input_ev("click", move |_| {
                set_count(count + 1);
                Msg::DoNothing
            }),
            Click Me
        ]
    ]
}
```

vs ReactJs:

```javascript
import React, { useState } from 'react';
function Example() {
  // Declare a new state variable, which we'll call "count"
  const [count, setCount] = useState(0);

  return (
    <div>
      <p>You clicked {count} times</p>
      <button onClick={() => setCount(count + 1)}>
        Click me
      </button>
    </div>
  );
}
```
See the first State Hook Example for a comparision: https://reactjs.org/docs/hooks-overview.html

Below is a slightly less magic version : 

```rust
#[topo::nested]
fn hook_style_button() -> Node<Msg> {
    // Needed to pass local execution topology to event callback
    let current_id = topo::Id::current();

    // On first execution sets the current button count to zero
    // on subsequent executions retrieves the stored button count
    let button_count = clone_state::<u32>().unwrap_or(0);
    
    button![
        input_ev("click", move |_| {
            
            // Increments the u32 count of the components execution state.
            set_state_with_topo_id::<u32>(button_count + 1, current_id);

            // Look Ma No Messages (I have a bad feeling about this...)
            Msg::DoNothing
        }),
        format!("Hook State Button × {}", button_count)
    ]
}
```


**Caveats:**

- Dont judge me I was bored this afteroon, this is almost certainly broken and a bad idea

- this 100% uses topo code from Moxie (https://github.com/anp/moxie), cut and pasted into this example because the latest version of topo was not published as a separate crate. 

-  The core technology (topo) is 100% created by and the idea of the Moxie team, I have nothing to do with them and just wanted to get topo working with seed.

**How does it work?**

- topo creates a new execution context for every `[topo::nested]` annoted function, and every `topo::call!` block. Further calls to `topo::root!` re-roots the execution context. The re-rooting allows for consistent execution contexts for the same components as long as you re-root at the start of the base view function. This means that one can store and retrieve local data for an individual component which has been annoted by `topo::nested`.

- See this awesome talk explaining how topo works: https://www.youtube.com/watch?v=tmM756XZt20

- a type gets stored with : `set_state::<String>(text)` which stores `text` in the component for the `String` type.

- there are several ways of getting a reference to a stored value. If you are okay with a clone the easiest way is `clone_state::<String>()`. However if you need a reference you need to access the global static `STORE.lock().unwrap().get::<String>()` and deal with the borrow checker issues.

- currently only 1 type per context is storable, however if you want to store more than 1 String say, you can create a `HashMap<key,String>` or `Vec` and store that.

- if you want to set the state from an event callback (quite common) you should use  `set_state_with_topo_id::<String>(text, current_id)` where current_id is obtained in the component itself. The reason for this is that the event callback will not run in the same context as the component. You can do this with `let current_id = topo::Id::current()`.

- I have no idea how 'stable' this pattern is particularly when you might have views iterating all over the place, particulary if you can't be certain of the order they are called in.

- If you want to use react style you can call `store::use_state::<u32>(0)` which returns a  `(count, set_count)`. `count` is the data being returned and `set_count` is an Arc boxed closure that can be called to update the state. The advantage of this is that the currentId is generated and passed automatically.

**Why would anyone want to do this?**

- I wanted to see what all the fuss is about with React Hooks so thought i'd implement in Seed in a very Hacky way.

- Just sometimes I get put off adjusting my app because of the Msg:: chain chase all over the place to keep track of one simple thing. I wanted to see if I could somehow have some more isolated changes in one function (component) with stored state.

- Per compoment data means for simple compoents I dont need to pollute the app with non-business logic.

- In theory this might be good, who knows, so lets try...

- This only stores state, which was the main reason I needed it.

**Standard quickstart stuff**

`rustup update`

`rustup target add wasm32-unknown-unknown`

`cargo install --force cargo-make`

Run `cargo make build` in a terminal to build the app, and `cargo make serve` to start a dev server
on `127.0.0.1:8000`.

If you'd like the compiler automatically check for changes, recompiling as
needed, run `cargo make watch` instead of `cargo make build`.

