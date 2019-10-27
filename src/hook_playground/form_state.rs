use crate::store::*;
use fancy_regex::Regex;
use seed::prelude::*;
use std::sync::Arc;

//A local copy of seed's UpdateEl Trait, this allows the form helper return a tuple of (Attrs,Vec<Listeners)
use seed::dom_types::UpdateEl;
pub trait UpdateElLocal<T> {
    fn update(self, el: &mut T);
}

impl<Ms> UpdateElLocal<El<Ms>> for (seed::dom_types::Attrs, Vec<seed::events::Listener<Ms>>) {
    fn update(self, el: &mut El<Ms>) {
        self.0.update(el);
        self.1.update(el);
    }
}

impl<Ms> UpdateElLocal<El<Ms>> for (seed::dom_types::Attrs, seed::events::Listener<Ms>) {
    fn update(self, el: &mut El<Ms>) {
        self.0.update(el);
        self.1.update(el);
    }
}

// Struct for storing form values
#[derive(Debug, Default, Clone)]
pub struct FormState {
    pub values: Vec<InputState>,
}

// Each Input element has a name, value, can be touched, is or isnt valid and has a list of errors
#[derive(Clone, Debug)]
pub struct InputState {
    pub name: String,
    pub value: String,
    pub touched: bool,
    pub valid: bool,
    pub errors: Vec<String>,
}

impl InputState {
    fn new<T: Into<String>>(name: T) -> InputState {
        let name = name.into();
        InputState {
            name,
            value: "".to_string(),
            touched: false,
            valid: false,
            errors: vec![],
        }
    }
}

impl FormState {
    // fn clear(&mut self) {
    //     for input in self.values.iter_mut() {
    //         input.value = "".to_string();
    //         input.touched = false;
    //         input.valid = false;
    //         input.errors = vec![];
    //     }
    // }

    // pub fn clone_value<T: Into<String>>(&self, name: T) -> String {
    //     let name = name.into();

    //     self.values
    //         .iter()
    //         .find(|inp| inp.name == name)
    //         .map(|inp| inp.value.clone())
    //         .unwrap_or_default()
    // }
}

// The controller struct that provides methods to output (Attrs,Vec<Listener) and also vec of errors
// PhantomData needed as Ms is the Msg type that is application specific and used in method types
use std::marker::PhantomData;
pub struct FormControl<Ms> {
    _phantom: PhantomData<Ms>,
    // id: topo::Id,
}

type ValidationClosure = Arc<dyn Fn(String) -> Result<(), String>>;

pub struct InputBuilder<Ms> {
    name: String,
    input_type: InputType,
    _phantom: PhantomData<Ms>,
    on_blur_closure: Option<Arc<dyn Fn(String) -> ()>>,
    validate_closures: Vec<ValidationClosure>,
    validate_on_blur_only: bool,
}

impl<Ms> InputBuilder<Ms>
where
    Ms: Default,
{
    fn new<T: Into<String>>(name: T, input_type: InputType) -> InputBuilder<Ms> {
        InputBuilder {
            _phantom: PhantomData,
            name: name.into(),
            on_blur_closure: None,
            validate_closures: vec![],
            validate_on_blur_only: false,
            input_type,
        }
    }

    pub fn letters_num_and_special_required(mut self) -> Self {
        self.validate_closures.push(Arc::new(|value| {
            let re = Regex::new(r"^(?=.*?[0-9])(?=.*?[A-Za-z])(?=.*[^0-9A-Za-z]).+$").unwrap();
            if !re.is_match(&value).unwrap() {
                Err(
                    "This field needs at least one letter, one number and one special character!"
                        .to_string(),
                )
            } else {
                Ok(())
            }
        }));
        self
    }
    pub fn required(mut self) -> Self {
        self.validate_closures.push(Arc::new(|value| {
            if value.is_empty() {
                Err("This field cannot be empty!".to_string())
            } else {
                Ok(())
            }
        }));
        self
    }

    pub fn validate_on_blur_only(mut self) -> Self {
        self.validate_on_blur_only = true;
        self
    }

    pub fn on_blur<F: Fn(String) -> () + 'static>(mut self, func: F) -> Self {
        self.on_blur_closure = Some(Arc::new(func));
        self
    }

    pub fn validate_with<F: Fn(String) -> Result<(), String> + 'static>(mut self, func: F) -> Self {
        self.validate_closures.push(Arc::new(func));
        self
    }

    pub fn render(&self) -> (seed::dom_types::Attrs, Vec<seed::events::Listener<Ms>>) {
        (
            match &self.input_type {
                InputType::Text => self.text_attrs(self.name.clone()),
                InputType::Password => self.password_attrs(self.name.clone()),
            },
            self.events(self.name.clone()),
        )
    }

    fn password_attrs<T: Into<String>>(&self, name: T) -> seed::dom_types::Attrs {
        let name = name.into();
        // state and access to the form_state, form_state needs to be mutated with new InputState if one does not already exist
        let (mut form_state, set_state) = use_state(FormState::default());

        let text_input_value =
            if let Some(input) = form_state.values.iter().find(|input| input.name == name) {
                input.value.clone()
            } else {
                form_state.values.push(InputState::new(name));
                set_state(form_state.clone());
                "".to_string()
            };
        attrs! {At::Type => "password", At::Value => text_input_value}
    }

    fn text_attrs<T: Into<String>>(&self, name: T) -> seed::dom_types::Attrs {
        let name = name.into();
        // state and access to the form_state, form_state needs to be mutated with new InputState if one does not already exist
        let (mut form_state, set_state) = use_state(FormState::default());

        let text_input_value =
            if let Some(input) = form_state.values.iter().find(|input| input.name == name) {
                input.value.clone()
            } else {
                form_state.values.push(InputState::new(name));
                set_state(form_state.clone());
                "".to_string()
            };
        attrs! {At::Type => "text", At::Value => text_input_value}
    }

    fn events<T: Into<String>>(&self, name: T) -> Vec<seed::events::Listener<Ms>> {
        let name = name.into();
        let mut listeners = vec![];

        let (_form_state, set_state) = use_state(FormState::default());
        let form_state_getter = state_getter::<FormState>();
        let field_name = name.clone();
        listeners.push(input_ev("blur", move |_text| {
            if let Some(mut form_state) = form_state_getter() {
                if let Some(input) = form_state
                    .values
                    .iter_mut()
                    .find(|input| input.name == field_name)
                {
                    input.errors = vec![];
                }
                set_state(form_state);
            }
            Ms::default()
        }));

        let (_form_state, set_state) = use_state(FormState::default());
        let form_state_getter = state_getter::<FormState>();
        let field_name = name.clone();
        listeners.push(input_ev("input", move |_text| {
            if let Some(mut form_state) = form_state_getter() {
                if let Some(input) = form_state
                    .values
                    .iter_mut()
                    .find(|input| input.name == field_name)
                {
                    input.errors = vec![];
                }
                set_state(form_state);
            }
            Ms::default()
        }));

        // pushes an input event listener which only does something if form_state exists.
        // If it does exit the callback will each for an inputs state with matching name
        // and then update it if the text value has changed.
        let field_name = name.clone();
        let (_form_state, set_state) = use_state(FormState::default());
        let form_state_getter = state_getter::<FormState>();
        listeners.push(input_ev("input", move |text| {
            if let Some(mut form_state) = form_state_getter() {
                if let Some(input) = form_state
                    .values
                    .iter_mut()
                    .find(|input| input.name == field_name)
                {
                    if input.value != text {
                        input.errors = vec![];
                        input.touched = true;
                        input.value = text;
                    }
                }
                set_state(form_state);
            }
            Ms::default()
        }));

        // if there is an onblur closure then add an event listener to call it
        let closure = self.on_blur_closure.clone();
        if closure.is_some() {
            listeners.push(input_ev("blur", move |text| {
                if let Some(callback) = closure {
                    callback(text);
                };
                Ms::default()
            }))
        }

        // determine whether a validation callback is on blur or on input
        if !self.validate_on_blur_only {
            // if a validate closure exist add it to the input event callback chain.
            for closure in self.validate_closures.iter().cloned() {
                // set state accessor, note we cannot use _form_state in a closure as it's
                // state will be as per when this function is called, not at time of callback
                // we need a form_state_getter which is a function not a struct that gets called on callback
                let (_form_state, set_state) = use_state(FormState::default());
                let form_state_getter = state_getter::<FormState>();
                let field_name = name.clone();
                listeners.push(input_ev("input", move |text| {
                    if let Some(mut form_state) = form_state_getter() {
                        // if the callback generates an error add it to the errors vec field
                        if let Err(error) = closure(text) {
                            if let Some(input) = form_state
                                .values
                                .iter_mut()
                                .find(|input| input.name == field_name)
                            {
                                input.errors.push(error);
                            }
                            set_state(form_state);
                        };
                    }
                    Ms::default()
                }))
            }
        }
        // if a validate closure exist add it to the input event callback chain.
        for closure in self.validate_closures.iter().cloned() {
            // set state accessor, note we cannot use _form_state in a closure as it's
            // state will be as per when this function is called, not at time of callback
            // we need a form_state_getter which is a function not a struct that gets called on callback
            let (_form_state, set_state) = use_state(FormState::default());
            let form_state_getter = state_getter::<FormState>();
            let field_name = name.clone();
            listeners.push(input_ev("blur", move |text| {
                if let Some(mut form_state) = form_state_getter() {
                    // if the callback generates an error add it to the errors vec field
                    if let Err(error) = closure(text) {
                        if let Some(input) = form_state
                            .values
                            .iter_mut()
                            .find(|input| input.name == field_name)
                        {
                            input.errors.push(error);
                        }
                        set_state(form_state);
                    };
                }
                Ms::default()
            }))
        }
        listeners
    }
}

impl<Ms> FormControl<Ms>
where
    Ms: Default,
{
    pub fn input_errors_for<T: Into<String>>(&self, name: T) -> Vec<Node<Ms>> {
        let name = name.into();
        let (form_state, _set_state) = use_state(FormState::default());
        if let Some(input) = form_state.values.iter().find(|inp| inp.name == name) {
            input.errors.iter().map(|err| span![err]).collect::<_>()
        } else {
            vec![]
        }
    }

    pub fn text<T: Into<String>>(&self, name: T) -> InputBuilder<Ms> {
        InputBuilder::new(name, InputType::Text)
    }

    pub fn password<T: Into<String>>(&self, name: T) -> InputBuilder<Ms> {
        InputBuilder::new(name, InputType::Password)
    }
}

enum InputType {
    Password,
    Text,
}

pub fn use_form_state<Ms: Default>() -> (FormState, FormControl<Ms>) {
    use_form_state_builder::<Ms>().build()
}

#[derive(Default)]
pub struct StateFormBuilder<Ms> {
    _phantom: PhantomData<Ms>,
    on_touched_closure: Option<Arc<dyn Fn(FormState) -> ()>>,
    validate_on_blur_only: bool,
}

impl<Ms> StateFormBuilder<Ms>
where
    Ms: Default,
{
    pub fn build(&self) -> (FormState, FormControl<Ms>) {
        let (form_state, _set_state) = use_state(FormState::default());
        (
            form_state,
            FormControl {
                _phantom: PhantomData,
                // id: topo::Id::current(),
            },
        )
    }
}

pub fn use_form_state_builder<Ms>() -> StateFormBuilder<Ms>
where
    Ms: Default,
{
    StateFormBuilder::default()
}
