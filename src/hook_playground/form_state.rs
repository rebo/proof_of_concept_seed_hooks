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
    on_blur_closure: Option<Arc<dyn Fn(FormState) -> ()>>,
    // id: topo::Id,
}

type ValidationClosure = Arc<dyn Fn(String) -> Result<(), String>>;

pub struct InputBuilder<Ms> {
    name: String,
    input_type: InputType,
    _phantom: PhantomData<Ms>,
    default_value: Option<String>,
    on_blur_closure: Option<Arc<dyn Fn(String) -> ()>>,
    form_on_blur_closure: Option<Arc<dyn Fn(FormState) -> ()>>,
    validate_closures: Vec<ValidationClosure>,
    validate_on: InputBlurBothEnum,
}

impl<Ms> InputBuilder<Ms>
where
    Ms: Default,
{
    fn new<T: Into<String>>(
        name: T,
        input_type: InputType,
        form_on_blur_closure: Option<Arc<dyn Fn(FormState) -> ()>>,
    ) -> InputBuilder<Ms> {
        InputBuilder {
            _phantom: PhantomData,
            name: name.into(),
            default_value: None,
            on_blur_closure: None,
            form_on_blur_closure: form_on_blur_closure.clone(),
            validate_closures: vec![],
            validate_on: InputBlurBothEnum::Both,
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

    pub fn default_value<F: Into<String>>(mut self, value: F) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn validate_on_blur_only(mut self) -> Self {
        self.validate_on = InputBlurBothEnum::Blur;
        self
    }

    pub fn validate_on_input_only(mut self) -> Self {
        self.validate_on = InputBlurBothEnum::Input;
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
            } else if let Some(value) = &self.default_value {
                form_state.values.push(InputState::new(name));
                set_state(form_state.clone());
                value.clone()
            } else {
                form_state.values.push(InputState::new(name));
                set_state(form_state.clone());
                "".to_string()
            };
        attrs! {At::Type => "text", At::Value => text_input_value}
    }

    fn clear_errors_blur_event(&self, name: String) -> seed::events::Listener<Ms> {
        let (_form_state, set_state) = use_state(FormState::default());
        let form_state_getter = state_getter::<FormState>();
        let field_name = name.clone();
        input_ev("blur", move |_text| {
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
        })
    }

    fn clear_errors_input_event(&self, name: String) -> seed::events::Listener<Ms> {
        let (_form_state, set_state) = use_state(FormState::default());
        let form_state_getter = state_getter::<FormState>();
        let field_name = name.clone();
        input_ev("input", move |_text| {
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
        })
    }

    fn update_value_input_event(&self, name: String) -> seed::events::Listener<Ms> {
        let field_name = name.clone();
        let (_form_state, set_state) = use_state(FormState::default());
        let form_state_getter = state_getter::<FormState>();
        input_ev("input", move |text| {
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
                    } else {
                        input.touched = false;
                    }
                }
                set_state(form_state);
            }
            Ms::default()
        })
    }

    fn input_specific_on_blur_event(&self) -> Option<seed::events::Listener<Ms>> {
        let closure = self.on_blur_closure.clone();
        if closure.is_some() {
            Some(input_ev("blur", move |text| {
                if let Some(callback) = closure {
                    callback(text);
                };
                Ms::default()
            }))
        } else {
            None
        }
    }

    fn form_on_blur_event(&self) -> Option<seed::events::Listener<Ms>> {
        let closure = self.form_on_blur_closure.clone();
        let form_state_getter = state_getter::<FormState>();
        if closure.is_some() {
            Some(input_ev("blur", move |_text| {
                if let (Some(callback), Some(form_state)) = (closure, form_state_getter()) {
                    callback(form_state);
                };
                Ms::default()
            }))
        } else {
            None
        }
    }

    fn validation_closures_event(
        &self,
        name: String,
        event_type: &'static str,
    ) -> Option<seed::events::Listener<Ms>> {
        let closures = self.validate_closures.clone();

        let (_form_state, set_state) = use_state(FormState::default());
        let form_state_getter = state_getter::<FormState>();
        let field_name = name.clone();
        if !closures.is_empty() {
            Some(input_ev(event_type, move |text| {
                if let Some(mut form_state) = form_state_getter() {
                    // if the callback generates an error add it to the errors vec field
                    for closure in closures.iter() {
                        let text = text.clone();
                        if let Err(error) = closure(text) {
                            if let Some(input) = form_state
                                .values
                                .iter_mut()
                                .find(|input| input.name == field_name)
                            {
                                input.errors.push(error);
                            }
                            set_state(form_state.clone());
                        };
                    }
                }
                Ms::default()
            }))
        } else {
            None
        }
    }

    fn events<T: Into<String>>(&self, name: T) -> Vec<seed::events::Listener<Ms>> {
        let name = name.into();

        // setup event to clear errors first for every blur and every input
        let mut listeners = vec![
            self.clear_errors_blur_event(name.clone()),
            self.clear_errors_input_event(name.clone()),
            self.update_value_input_event(name.clone()),
        ];
        if let Some(event) = self.input_specific_on_blur_event() {
            listeners.push(event);
        }
        if !self.validate_closures.is_empty() {
            match self.validate_on {
                InputBlurBothEnum::Both => {
                    listeners.push(
                        self.validation_closures_event(name.clone(), "blur")
                            .unwrap(),
                    );
                    listeners.push(
                        self.validation_closures_event(name.clone(), "input")
                            .unwrap(),
                    );
                }
                InputBlurBothEnum::Blur => {
                    listeners.push(
                        self.validation_closures_event(name.clone(), "blur")
                            .unwrap(),
                    );
                }
                InputBlurBothEnum::Input => {
                    listeners.push(
                        self.validation_closures_event(name.clone(), "input")
                            .unwrap(),
                    );
                }
            }
        }
        if let Some(event) = self.form_on_blur_event() {
            listeners.push(event);
        }
        // if a validate closure exist add it to the input event callback chain.
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
        InputBuilder::new(name, InputType::Text, self.on_blur_closure.clone())
    }

    pub fn password<T: Into<String>>(&self, name: T) -> InputBuilder<Ms> {
        InputBuilder::new(name, InputType::Password, self.on_blur_closure.clone())
    }
}

enum InputType {
    Password,
    Text,
}

pub fn use_form_state<Ms: Default>() -> (FormState, FormControl<Ms>) {
    use_form_state_builder::<Ms>().build()
}

enum InputBlurBothEnum {
    Input,
    Blur,
    Both,
}

impl Default for InputBlurBothEnum {
    fn default() -> InputBlurBothEnum {
        InputBlurBothEnum::Both
    }
}

#[derive(Default)]
pub struct StateFormBuilder<Ms> {
    _phantom: PhantomData<Ms>,
    on_blur_closure: Option<Arc<dyn Fn(FormState) -> ()>>,
    validate_on: InputBlurBothEnum,
}

impl<Ms> StateFormBuilder<Ms>
where
    Ms: Default,
{
    pub fn on_blur<F: Fn(FormState) -> () + 'static>(mut self, func: F) -> Self {
        self.on_blur_closure = Some(Arc::new(func));
        self
    }

    pub fn build(&self) -> (FormState, FormControl<Ms>) {
        let (form_state, _set_state) = use_state(FormState::default());
        (
            form_state,
            FormControl {
                _phantom: PhantomData,
                on_blur_closure: self.on_blur_closure.clone(),
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
