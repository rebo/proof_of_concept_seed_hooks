// If you are going to use UseFetch you need the following in your base g::

//FetchString(topo::Id, String, Method),
//FetchedString(topo::Id, String),

use comp_state::{update_state_with_topo_id, use_state, StateAccess};
use enclose::enclose;
use futures::{Async, Future, Poll};
use seed::{prelude::*, *};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use wasm_bindgen_futures::spawn_local;

use wasm_bindgen_futures::JsFuture;

// Code + docs: https://rustwasm.github.io/wasm-bindgen/api/wasm_bindgen_futures/

/// A future that becomes ready after a tick of the micro task queue.
pub struct NextTick {
    inner: JsFuture,
}

impl NextTick {
    /// Construct a new `NextTick` future.
    pub fn new() -> NextTick {
        // Create a resolved promise that will run its callbacks on the next
        // tick of the micro task queue.
        let promise = js_sys::Promise::resolve(&JsValue::NULL);
        // Convert the promise into a `JsFuture`.
        let inner = JsFuture::from(promise);
        NextTick { inner }
    }
}

impl Default for NextTick {
    fn default() -> Self {
        Self::new()
    }
}

impl Future for NextTick {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<(), ()> {
        // Polling a `NextTick` just forwards to polling if the inner promise is
        // ready.
        match self.inner.poll() {
            Ok(Async::Ready(_)) => Ok(Async::Ready(())),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => unreachable!(
                "We only create NextTick with a resolved inner promise, never \
                 a rejected one, so we can't get an error here"
            ),
        }
    }
}

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
pub fn use_fetch_with_json<T: Clone + DeserializeOwned + std::fmt::Debug + 'static>(
    url: &str,
    _method: Method,
    json: &str,
    container_name: Option<&str>,
) -> (Option<T>, StateAccess<UseFetchJson<T>>) {
    topo::call!({
        let (state, state_access) = use_state(|| UseFetchJson::<T>::new(url, json));

        let possible_type: Option<T> = match (state.status, state.string_response) {
            (UseFetchStatus::Complete, Some(mut response)) => {
                if let Some(container_name) = container_name {
                    response = response.replace(&container_name, "UseFetchJsonItems");
                }
                let result: Result<T, _> = serde_json::from_str(&response);
                let poss = result.unwrap();
                Some(poss)
            }
            _ => None,
        };
        (possible_type, state_access)
    })
}

#[derive(Clone, Debug)]
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
    pub fail_reason: Option<seed::fetch::FailReason<String>>,
    pub url: String,
    pub method: Method,
}

impl UseFetch {
    fn new(url: String, method: Method) -> UseFetch {
        UseFetch {
            status: UseFetchStatus::Initialized,
            string_response: None,
            fail_reason: None,
            url,
            method,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UseFetchJson<T> {
    pub status: UseFetchStatus,
    pub json_body: String,
    pub string_response: Option<String>,
    pub url: String,
    pub _phantom_data: PhantomData<T>,
}

impl<T> UseFetchJson<T> {
    fn new(url: &str, json_body: &str) -> UseFetchJson<T> {
        UseFetchJson {
            status: UseFetchStatus::Initialized,
            string_response: None,
            url: url.to_string(),
            json_body: json_body.to_string(),
            _phantom_data: PhantomData::default(),
        }
    }
}

pub trait UseFetchStatusTrait: Clone {
    fn status(&self) -> UseFetchStatus;
    fn dispatch<Ms: UseFetchMsgTrait + Default + 'static, Mdl: 'static>(&self);
    fn dispatch_with_seed<Ms: UseFetchMsgTrait + 'static, Mdl: 'static>(&self);
}

pub trait UseFetchJsonStatusTrait: Clone {
    fn status(&self) -> UseFetchStatus;
    fn dispatch<Ms: Default + Clone + 'static, Mdl: 'static>(&self);
    // fn dispatch_with_seed<Ms: UseFetchMsgTrait + 'static, Mdl: 'static>(&self);
}

pub trait UseFetchMsgTrait {
    fn fetch_message(id: topo::Id, url: String, method: Method) -> Self;
    fn fetched_message(id: topo::Id, response: String) -> Self;
}

impl UseFetchStatusTrait for StateAccess<UseFetch> {
    fn status(&self) -> UseFetchStatus {
        self.get().unwrap().status
    }

    fn dispatch_with_seed<Ms: UseFetchMsgTrait + 'static, Mdl: 'static>(&self) {
        let use_fetch = self.get().unwrap();
        self.update(|state| state.status = UseFetchStatus::Loading);
        let url = use_fetch.url.clone();
        let method = use_fetch.method;
        let id = self.id;
        let boxed_fn = {
            Box::new(move || {
                if let Some(app) = topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>() {
                    app.update(Ms::fetch_message(id, url.clone(), method));
                }
            })
        };

        seed::set_timeout(boxed_fn, 0);
    }

    fn dispatch<Ms: UseFetchMsgTrait + 'static + Default, Mdl: 'static>(&self) {
        let use_fetch = self.get().unwrap();
        self.update(|state| state.status = UseFetchStatus::Loading);
        let url = use_fetch.url.clone();
        let method = use_fetch.method;
        let id = self.id;
        let boxed_fn = {
            Box::new(move || {
                if let Some(app) = topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>() {
                    let lazy_schedule_cmd = enclose!((app => s, url) move |_| {
                        let url = url.clone();
                        spawn_local(  {fetch_string::<Ms>(id, url, method).then(move |_| {
                                // let msg_returned_from_effect = res.unwrap_or_else(|err_msg| err_msg);
                                // recursive call which can blow the call stack
                                s.update(Ms::default());
                                Ok(()) })}
                            )
                    });
                    // we need to clear the call stack by NextTick so we don't exceed it's capacity
                    spawn_local(NextTick::new().map(lazy_schedule_cmd));

                    app.update(Ms::default());
                }
            })
        };

        seed::set_timeout(boxed_fn, 0);
    }
}

pub fn fetch_string<Ms: UseFetchMsgTrait + Default + 'static>(
    id: topo::Id,
    url: String,
    method: Method,
) -> impl Future<Item = Ms, Error = Ms> {
    seed::fetch::Request::new(url)
        .method(method)
        .fetch_string(move |f| {
            match f.response() {
                Ok(response) => update_state_with_topo_id::<UseFetch, _>(id, |u| {
                    u.status = UseFetchStatus::Complete;
                    u.string_response = Some(response.data.clone());
                }),
                Err(fail_reason) => update_state_with_topo_id::<UseFetch, _>(id, |u| {
                    u.status = UseFetchStatus::Failed;
                    u.fail_reason = Some(fail_reason);
                    // u.string_response = Some();
                }),
            }
            Ms::default()
        })
}

impl<T> UseFetchJsonStatusTrait for StateAccess<UseFetchJson<T>>
where
    T: Clone + DeserializeOwned + 'static + std::fmt::Debug,
{
    fn status(&self) -> UseFetchStatus {
        self.get().unwrap().status
    }

    fn dispatch<Ms: 'static + Clone + Default, Mdl: 'static>(&self) {
        let use_fetch = self.get().unwrap();
        self.update(|state| state.status = UseFetchStatus::Loading);
        let url = use_fetch.url.clone();
        let id = self.id;
        let boxed_fn = {
            Box::new(move || {
                if let Some(app) = topo::Env::get::<seed::App<Ms, Mdl, Node<Ms>>>() {
                    let lazy_schedule_cmd = enclose!((url, use_fetch) move |_| {
                        let url = url.clone();
                        spawn_local(
                                {fetch_json::<T,Ms,Mdl>(id, url,use_fetch.json_body).then(move |_| {
                                // let msg_returned_from_effect = res.unwrap_or_else(|err_msg| err_msg);
                                // recursive call which can blow the call stack
                                // s.update(Ms::default());
                                Ok(()) })}
                            )

                    });
                    // we need to clear the call stack by NextTick so we don't exceed it's capacity
                    spawn_local(NextTick::new().map(lazy_schedule_cmd));

                    app.update(Ms::default());
                }
            })
        };

        seed::set_timeout(boxed_fn, 0);
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageRequestBody {
    pub query: String,
}

pub fn fetch_json<
    T: DeserializeOwned + std::fmt::Debug + 'static + Clone,
    Ms: Default + Clone + 'static,
    Mdl: 'static,
>(
    id: topo::Id,
    url: String,
    json: String,
) -> impl Future<Item = Ms, Error = Ms> {
    let json_request = &SendMessageRequestBody { query: json };

    seed::fetch::Request::new(url)
        .method(Method::Post)
        .send_json(json_request)
        .fetch_string(move |f| {
            let data = f.response().unwrap().data;
            update_state_with_topo_id::<UseFetchJson<T>, _>(id, |u| {
                u.status = UseFetchStatus::Complete;
                u.string_response = Some(data.clone());
                crate::schedule_update::<Ms, Mdl>(Ms::default());
            });
            Ms::default()
        })
}

pub fn fetch_string_with_seed_msg<Ms: UseFetchMsgTrait + 'static>(
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
    orders.perform_cmd(fetch_string_with_seed_msg::<Ms>(id, url, method));
}

pub fn update_fetched(id: topo::Id, string_response: String) {
    update_state_with_topo_id::<UseFetch, _>(id, |u| {
        u.status = UseFetchStatus::Complete;
        u.string_response = Some(string_response.clone());
    })
}
