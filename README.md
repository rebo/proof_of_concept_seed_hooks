# EXPERIMENTAL: React style Hook & Custom Hooks in Seed 


Seed is a frontend framework written in Rust and compiled to Wasm. This is an attempt to see if react style 'hooks' can work with seed.

**Example:**

This is a complete counting button with state, no messages required: 

```rust
#[topo::nested]
fn hook_style_button() -> Node<Msg> {
    // Declare a new state variable which we'll call "count"
    let (count, count_access) = use_state(|| 0);
    div![
        p![format!("You clicked {} times", count)],
        button![
            input_ev("click", move |_| {
                count_access.set(count + 1);
                Msg::DoNothing
            }),
            format!("Click Me × {}", count)
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

It's also possible to write 'custom hooks' for instance form validation that allows the below to all happen in a view function (Note that this is just a toy example and not fully usable):

```rust
#[topo::nested]
pub fn simple_form_test() -> Node<Msg> {
    
    // form custom hook
    // form ctl allows for rendering of various input types and 
    // handles all callbacks and local state
    let (_form_state, ctl) = form_state::use_form_state::<Msg>();
    div![
        div![
            label!["description"],
            // a 'required' text input field
            input![ctl.text("description").required().render()],
            ctl.input_errors_for("description"),
        ],
        div![
            label!["password"],
            // a password field that is both required and also must include various characters
            input![ctl
                .password("password")
                .required()
                .letters_num_and_special_required()
                .render()],
            ctl.input_errors_for("password"),
        ],
    ]
}
```

This workspace is divided into 3 sub-projects.

1. comp_state - A wrapper/plumbing for moxie/topo sets up a Global for contexts to store arbitraty data in. This provides the basic use_state() function that lets a topological aware execution context store and retreive state.  This is analogous to use_state in React. This crate does not depend on seed.
2. seed_comp_helpers - Experimental helper functions for use within seed. Effectively these are custom hooks that build on functionality provided by use_state()
Current helpers include -  form state,  list management ,  memoization and two_way component communicaiton. All of these are proof of concept only and need to be fully developed to be any real use.
3. example_seed_app - This is a full app that demonstrates the use of the seed helper and comp_state directly. Everything is loaded in one page, see hook_playground.

**Caveats:**

- This actually now seems to work more or less okay.

- this 100% uses topo code from Moxie (https://github.com/anp/moxie), cut and pasted into this example because the latest version of topo was not published as a separate crate. 

-  The core technology (topo) is 100% created by and the idea of the Moxie team, I have nothing to do with them and just wanted to get topo working with seed.

**How does it work?**

- topo creates a new execution context for every `[topo::nested]` annoted function, and every `topo::call!` block. Further calls to `topo::root!` re-roots the execution context. The re-rooting allows for consistent execution contexts for the same components as long as you re-root at the start of the base view function. This means that one can store and retrieve local data for an individual component which has been annoted by `topo::nested`.

- The execution context is not only determined by the order of calling `[topo::nested]` functions but also the source location of these calls. This means that state is consistent and stable even though branching logic might call topologically aware functions in different orders.

- See this awesome talk explaining how topo works: https://www.youtube.com/watch?v=tmM756XZt20

- a type gets stored with : `let (my_string, string_access) = use_state::<String>(||text)` which stores `text` in the component for the `String` type. This returns a tuple, with my_string being a clone of the latest state and string_access being an accessor which can get or set this value. 

- The accessor is useful because it can be passed to callbacks or cloned or called from different topological contexts. i.e. `string_acces.set(new_text)` will work no matter where it is called.

- currently comp_state only exposes a clone to stored values. If you want references you have to hack it and dig into `STORE.lock().unwrap().get::<String>()`.

- currently only 1 type per context is storable, however if you want to store more than 1 String say, you can create a `HashMap<key,String>` , `Vec`, or NewType and store that.

- After some testing this now seems fairly stable. You still shouldn't use it though.

**Why would anyone want to do this?**

- I wanted to see what all the fuss is about with React Hooks so thought i'd implement in Seed in a very Hacky way.

- Just sometimes I get put off adjusting my app because of the Msg:: chain chase all over the place to keep track of one simple thing. I wanted to see if I could somehow have some more isolated changes in one function (component) with stored state.

- Per compoment data means for simple components I dont need to pollute the app with non-business logic.

- This should make it easier to create reusable chunks of logic that can be helpful in a different apps without having to wire in everything TEA style.

- In theory this might be good, who knows, so lets try...

**Standard quickstart stuff**

`cd example_seed_app`

`rustup update`

`rustup target add wasm32-unknown-unknown`

`cargo install --force cargo-make`

Run `cargo make build` in a terminal to build the app, and `cargo make serve` to start a dev server
on `127.0.0.1:8000`.

If you'd like the compiler automatically check for changes, recompiling as
needed, run `cargo make watch` instead of `cargo make build`.

