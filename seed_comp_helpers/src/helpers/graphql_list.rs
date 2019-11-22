use crate::use_fetch_helper;
pub use crate::use_fetch_helper::UseFetchJson;
pub use crate::use_fetch_helper::{UseFetchJsonStatusTrait, UseFetchStatus};
use comp_state::{use_list, List, ListControl, StateAccess};
use seed::*;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::Deserialize;
use std::fmt::Display;
use std::str::FromStr;
// enum DataResponseEnum<T> {

pub trait IntoList<I> {
    fn items(&self) -> Vec<I>;
}
#[derive(Clone)]
pub struct GraphQLListControl<I>
where
    I: Clone + 'static,
{
    pub list: ListControl<I>,
    pub fetcher: StateAccess<UseFetchJson<ArrayResponse<I>>>,
}

impl<I> GraphQLListControl<I>
where
    I: Clone + 'static,
    StateAccess<UseFetchJson<ArrayResponse<I>>>: UseFetchJsonStatusTrait,
{
    pub fn get_list(&self) -> List<I> {
        self.list.get_list()
    }

    pub fn dispatch<Ms, Mdl>(&self)
    where
        Ms: Clone + Default + 'static,
        Mdl: 'static,
    {
        self.fetcher.dispatch::<Ms, Mdl>();
    }

    pub fn status(&self) -> UseFetchStatus {
        self.fetcher.hard_get().status
    }
}

pub fn use_graphql_list<I: Clone + std::fmt::Debug + DeserializeOwned>(
    query: &str,
    url: &str,
    container_name: &str,
) -> (List<I>, GraphQLListControl<I>) {
    topo::call!({
        //create a blank list to be used later
        let (_list, list_control) = use_list(|| vec![]);

        // intialize fetch objects and control
        let (fetched, fetch_control) = use_fetch_helper::use_fetch_with_json::<ArrayResponse<I>>(
            url,
            Method::Post,
            query,
            Some(container_name),
        );
        // if fetched is returned as Some then
        // load list_control
        if let Some(fetched) = fetched {
            comp_state::do_once({
                || {
                    for item in fetched.items() {
                        list_control.push(item.clone());
                    }
                }
            })
        }

        let graphql_list_control = GraphQLListControl::<I> {
            list: list_control,
            fetcher: fetch_control,
        };
        // return list, list_cotnrol, and fetch_cotnrol
        (graphql_list_control.get_list(), graphql_list_control)
    })
}

// type LoadAllStrands<T> = Vec<T>;

#[derive(Clone, Debug, Deserialize)]
pub struct ArrayResponse<I> {
    pub data: DataResponseEnum<I>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum DataResponseEnum<I> {
    UseFetchJsonItems(Vec<I>),
}

impl<I> IntoList<I> for ArrayResponse<I>
where
    I: Clone,
{
    fn items(&self) -> Vec<I> {
        match &self.data {
            DataResponseEnum::UseFetchJsonItems(d) => d.to_vec(),
        }
    }
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}
