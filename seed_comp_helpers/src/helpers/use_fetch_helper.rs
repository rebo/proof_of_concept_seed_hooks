// If you are going to use UseFetch you need the following in your base g::

//FetchString(topo::Id, String, Method),
//FetchedString(topo::Id, String),

use comp_state::{update_state_with_topo_id, use_state, StateAccess};
use futures::prelude::*;
use seed::{prelude::*, *};
use serde::de::DeserializeOwned;

pub fn use_fetch<T: Clone + DeserializeOwned>(
    url: String,
    method: Method,
) -> (Option<T>, impl UseFetchStatusTrait) {
    topo::call!({
        let (state, state_access) = use_state(|| UseFetch::new(url, method));

        let possible_type: Option<T> = match (state.status, state.string_response) {
            (UseFetchStatus::Complete, Some(response)) => {
                let result: Result<T, _> = serde_json::from_str(&response);
                let poss = result.unwrap();
                Some(poss)
            }
            _ => None,
        };
        (possible_type, state_access)
    })
}

#[derive(Clone)]
pub enum UseFetchStatus {
    Initialized,
    Loading,
    Failed,
    Complete,
}

impl Default for UseFetchStatus {
    fn default() -> Self {
        UseFetchStatus::Initialized
    }
}
use std::default::Default;

#[derive(Clone)]
pub struct UseFetch {
    pub status: UseFetchStatus,
    pub string_response: Option<String>,
    pub url: String,
    pub method: Method,
}

impl UseFetch {
    fn new(url: String, method: Method) -> UseFetch {
        UseFetch {
            status: UseFetchStatus::Initialized,
            string_response: None,
            url,
            method,
        }
    }
}

pub trait UseFetchStatusTrait: Clone {
    fn status(&self) -> UseFetchStatus;
    fn dispatch<Ms: UseFetchMsgTrait + 'static, Mdl: 'static>(&self);
}

pub trait UseFetchMsgTrait {
    fn fetch_message(id: topo::Id, url: String, method: Method) -> Self;
    fn fetched_message(id: topo::Id, response: String) -> Self;
}

impl UseFetchStatusTrait for StateAccess<UseFetch> {
    fn status(&self) -> UseFetchStatus {
        self.get().unwrap().status
    }

    fn dispatch<Ms: UseFetchMsgTrait + 'static, Mdl: 'static>(&self) {
        let use_fetch = self.get().unwrap();
        self.update(|state| state.status = UseFetchStatus::Loading);
        let url = use_fetch.url.clone();
        let method = use_fetch.method;
        let id = self.id;
        let boxed_fn = {
            // clone_all!(url, state_access);d
            Box::new(move || {
                if let Some(app) = topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>() {
                    app.update(Ms::fetch_message(id, url.clone(), method));
                }
            })
        };

        // let (once, once_access) = use_state(|| false);
        // if !once {
        seed::set_timeout(boxed_fn, 0);
        // }
    }
}

pub fn fetch_string<Ms: UseFetchMsgTrait + 'static>(
    id: topo::Id,
    url: String,
    method: Method,
) -> impl Future<Item = Ms, Error = Ms> {
    seed::fetch::Request::new(url)
        .method(method)
        .fetch_string(move |f| Ms::fetched_message(id, f.response().unwrap().data))
}

pub fn update_fetch<Ms: UseFetchMsgTrait + 'static>(
    orders: &mut impl Orders<Ms>,
    id: topo::Id,
    url: String,
    method: Method,
) {
    orders.perform_cmd(fetch_string::<Ms>(id, url, method));
}

pub fn update_fetched(id: topo::Id, string_response: String) {
    update_state_with_topo_id::<UseFetch, _>(id, |u| {
        u.status = UseFetchStatus::Complete;
        u.string_response = Some(string_response.clone());
    })
}
