use crate::store::*;
use seed::prelude::*;
use std::sync::Arc;

use seed::dom_types::UpdateEl;
pub trait UpdateElLocal<T> {
    // T is the type of thing we're updating; eg attrs, style, events etc.
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

#[derive(Debug, Default, Clone)]
pub struct FormState {
    pub values: Vec<InputState>,
}

// #[derive(Default, Clone)]
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

    pub fn clone_value<T: Into<String>>(&self, name: T) -> String {
        let name = name.into();

        self.values
            .iter()
            .find(|inp| inp.name == name)
            .map(|inp| inp.value.clone())
            .unwrap_or_default()
    }
}

use std::marker::PhantomData;
pub struct FormControl<Ms> {
    _phantom: PhantomData<Ms>,
    id: topo::Id,
}

pub struct TextInputBuilder<Ms> {
    name: String,
    _phantom: PhantomData<Ms>,
    on_blur_closure: Option<Arc<dyn Fn(String) -> ()>>,
    validate_closure: Option<Arc<dyn Fn(String) -> Result<(), String>>>,
    validate_on_blur: bool,
}

impl<Ms> TextInputBuilder<Ms>
where
    Ms: Default,
{
    fn new<T: Into<String>>(name: T) -> TextInputBuilder<Ms> {
        TextInputBuilder {
            _phantom: PhantomData,
            name: name.into(),
            on_blur_closure: None,
            validate_closure: None,
            validate_on_blur: false,
        }
    }

    pub fn validate_on_blur(mut self) -> Self {
        self.validate_on_blur = true;
        self
    }

    pub fn on_blur<F: Fn(String) -> () + 'static>(mut self, func: F) -> Self {
        self.on_blur_closure = Some(Arc::new(func));
        self
    }

    pub fn validate_with<F: Fn(String) -> Result<(), String> + 'static>(mut self, func: F) -> Self {
        self.validate_closure = Some(Arc::new(func));
        self
    }

    pub fn render(&self) -> (seed::dom_types::Attrs, Vec<seed::events::Listener<Ms>>) {
        (
            self.text_attrs(self.name.clone()),
            self.events(self.name.clone()),
        )
    }
    fn text_attrs<T: Into<String>>(&self, name: T) -> seed::dom_types::Attrs {
        let name = name.into();
        let (get_form_state, get_set_state) = use_state(FormState::default());

        let text_input_value = if let Some(input) = get_form_state()
            .values
            .iter()
            .find(|input| input.name == name)
        {
            input.value.clone()
        } else {
            get_form_state().values.push(InputState::new(name));
            get_set_state(get_form_state().clone());
            "".to_string()
        };
        attrs! {At::Type => "text", At::Value => text_input_value}
    }

    fn events<T: Into<String>>(&self, name: T) -> Vec<seed::events::Listener<Ms>> {
        let name = name.into();
        let field_name = name.clone();
        let (get_form_state, get_set_state) = use_state(FormState::default());
        let mut listeners = vec![];
        listeners.push(input_ev("input", move |text| {
            log!("here2");
            let mut form_state = get_form_state();

            log!("hereafter");
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
            get_set_state(form_state);
            Ms::default()
        }));

        let field_name = name.clone();
        let closure = self.on_blur_closure.clone();
        if closure.is_some() {
            listeners.push(input_ev("blur", move |text| {
                if let Some(callback) = closure {
                    callback(text);
                };
                Ms::default()
            }))
        }

        let blur_or_input = if self.validate_on_blur {
            "blur"
        } else {
            "input"
        };

        let (get_form_state, get_set_state) = use_state(FormState::default());
        let closure = self.validate_closure.clone();
        if closure.is_some() {
            listeners.push(input_ev(blur_or_input, move |text| {
                log!("here");
                let mut form_state = get_form_state();
                log!(form_state);
                log!("hereafter");
                if let Some(callback) = closure {
                    if let Err(error) = callback(text) {
                        if let Some(input) = form_state
                            .values
                            .iter_mut()
                            .find(|input| input.name == field_name)
                        {
                            input.errors.push(error);
                        }
                        get_set_state(form_state);
                    };
                };
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
        let (get_form_state, _get_set_state) = use_state(FormState::default());
        if let Some(input) = get_form_state().values.iter().find(|inp| inp.name == name) {
            input.errors.iter().map(|err| span![err]).collect::<_>()
        } else {
            vec![]
        }
    }

    pub fn text<T: Into<String>>(&self, name: T) -> TextInputBuilder<Ms> {
        TextInputBuilder::new(name)
    }
}

pub fn use_form_state<Ms: Default>() -> (Arc<dyn Fn() -> FormState>, FormControl<Ms>) {
    use_form_state_builder::<Ms>().build()
}

#[derive(Default)]
pub struct StateFormBuilder<Ms> {
    _phantom: PhantomData<Ms>,
    on_touched_closure: Option<Arc<dyn Fn(FormState) -> ()>>,
    validate_on_blur: bool,
}

impl<Ms> StateFormBuilder<Ms>
where
    Ms: Default,
{
    pub fn build(&self) -> (Arc<dyn Fn() -> FormState>, FormControl<Ms>) {
        let (get_form_state, _get_set_state) = use_state(FormState::default());
        (
            get_form_state,
            FormControl {
                _phantom: PhantomData,
                id: topo::Id::current(),
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
