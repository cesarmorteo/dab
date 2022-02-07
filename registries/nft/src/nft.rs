use ic_cdk::export::candid::{CandidType, Principal};
use ic_kit::macros::*;
use ic_kit::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::validate_url;

pub struct Controller(pub Principal);

impl Default for Controller {
    fn default() -> Self {
        panic!()
    }
}

#[init]
fn init() {
    ic::store(Controller(ic::caller()));
}

fn is_controller(account: &Principal) -> bool {
    account == &ic::get::<Controller>().0
}

#[update]
fn set_controller(new_controller: Principal) -> Result<(), OperationError> {
    if is_controller(&ic::caller()) {
        ic::store(Controller(new_controller));
        return Ok(());
    }
    Err(OperationError::NotAuthorized)
}

#[derive(CandidType, Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum DetailValue {
    True,
    False,
    U64(u64),
    I64(i64),
    Float(f64),
    Text(String),
    Principal(Principal),
    #[serde(with = "serde_bytes")]
    Slice(Vec<u8>),
    Vec(Vec<DetailValue>),
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq)]
pub struct NftCanister {
    pub name: String,
    pub description: String,
    pub thumbnail: String,
    pub frontend: Option<String>,
    pub principal_id: Principal,
    pub details: Vec<(String, DetailValue)>,
}

#[derive(Default)]
pub struct Registry(HashMap<Principal, NftCanister>);

impl Registry {
    pub fn archive(&mut self) -> Vec<(Principal, NftCanister)> {
        let map = std::mem::replace(&mut self.0, HashMap::new());
        map.into_iter().collect()
    }

    pub fn load(&mut self, archive: Vec<(Principal, NftCanister)>) {
        assert!(self.0.is_empty());
        self.0 = archive.into_iter().collect();
    }

    pub fn add(&mut self, canister_info: NftCanister) -> Result<(), OperationError> {
        self.0.insert(canister_info.principal_id, canister_info);
        Ok(())
    }

    pub fn remove(&mut self, principal_id: &Principal) -> Result<(), OperationError> {
        if self.0.remove(&principal_id).is_some() {
            return Ok(());
        }

        Err(OperationError::NonExistentItem)
    }

    pub fn get(&self, principal_id: &Principal) -> Option<&NftCanister> {
        self.0.get(principal_id)
    }

    pub fn get_all(&self) -> Vec<&NftCanister> {
        self.0.values().collect()
    }
}

#[query]
fn name() -> String {
    String::from("NFT Registry Canister")
}

#[derive(CandidType)]
pub enum OperationError {
    NotAuthorized,
    NonExistentItem,
    BadParameters,
    Unknown(String),
}

#[update]
fn add(canister_info: NftCanister) -> Result<(), OperationError> {
    if !is_controller(&ic::caller()) {
        return Err(OperationError::NotAuthorized);
    } else if !validate_url(&canister_info.thumbnail) {
        return Err(OperationError::BadParameters);
    } else if canister_info.frontend.is_some()
        && !validate_url(&canister_info.frontend.clone().unwrap())
    {
        return Err(OperationError::BadParameters);
    } else if canister_info.details[0].0 != String::from("standard") {
        return Err(OperationError::BadParameters);
    } else if canister_info.details.len() != 1 {
        return Err(OperationError::BadParameters);
    }

    let name = canister_info.name.clone();
    if name.len() <= 120 && &canister_info.description.len() <= &1200 {
        let db = ic::get_mut::<Registry>();
        return db.add(canister_info);
    }

    Err(OperationError::BadParameters)
}

#[update]
fn remove(principal_id: Principal) -> Result<(), OperationError> {
    if !is_controller(&ic::caller()) {
        return Err(OperationError::NotAuthorized);
    }

    let db = ic::get_mut::<Registry>();
    db.remove(&principal_id)
}

#[query]
fn get(principal_id: Principal) -> Option<&'static NftCanister> {
    let db = ic::get_mut::<Registry>();
    db.get(&principal_id)
}

#[query]
fn get_all() -> Vec<&'static NftCanister> {
    let db = ic::get_mut::<Registry>();
    db.get_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller() {
        // alice is the controller
        let ctx = MockContext::new()
            .with_caller(mock_principals::alice())
            .inject();

        init();

        let canister_info = NftCanister {
            name: String::from("xtc"),
            principal_id: mock_principals::xtc(),
            description: String::from("XTC is your cycles wallet."),
            thumbnail: String::from("https://google.com"),
            frontend: None,
            details: vec![(
                String::from("standard"),
                DetailValue::Text(String::from("Dank")),
            )],
        };

        let mut addition = add(canister_info.clone());
        assert!(addition.is_ok());

        let remove_operation = remove(mock_principals::xtc());
        assert!(remove_operation.is_ok());

        ctx.update_caller(mock_principals::bob());
        addition = add(canister_info);
        assert!(addition.is_err());
    }

    #[test]
    fn test_add() {
        MockContext::new()
            .with_caller(mock_principals::alice())
            .with_data(Controller(mock_principals::alice()))
            .inject();

        let canister_info = NftCanister {
            name: String::from("xtc"),
            principal_id: mock_principals::xtc(),
            description: String::from("XTC is your cycles wallet."),
            thumbnail: String::from("https://google.com"),
            frontend: None,
            details: vec![(
                String::from("standard"),
                DetailValue::Text(String::from("Dank")),
            )],
        };

        assert!(add(canister_info).is_ok());
    }

    #[test]
    fn test_remove() {
        MockContext::new()
            .with_caller(mock_principals::alice())
            .with_data(Controller(mock_principals::alice()))
            .inject();

        let canister_info = NftCanister {
            name: String::from("xtc"),
            principal_id: mock_principals::xtc(),
            description: String::from("XTC is your cycles wallet."),
            thumbnail: String::from("https://google.com"),
            frontend: None,
            details: vec![(
                String::from("standard"),
                DetailValue::Text(String::from("Dank")),
            )],
        };

        assert!(add(canister_info).is_ok());

        assert!(remove(mock_principals::xtc()).is_ok());
    }

    #[test]
    fn test_get() {
        MockContext::new()
            .with_caller(mock_principals::alice())
            .with_data(Controller(mock_principals::alice()))
            .inject();

        let canister_info = NftCanister {
            name: String::from("xtc"),
            principal_id: mock_principals::xtc(),
            description: String::from("XTC is your cycles wallet."),
            thumbnail: String::from("https://google.com"),
            frontend: None,
            details: vec![(
                String::from("standard"),
                DetailValue::Text(String::from("Dank")),
            )],
        };

        assert!(add(canister_info.clone()).is_ok());

        assert_eq!(
            get(mock_principals::xtc()).unwrap().name,
            canister_info.name
        );
        assert!(get(mock_principals::alice()).is_none());
    }

    #[test]
    fn test_get_all() {
        MockContext::new()
            .with_caller(mock_principals::alice())
            .with_data(Controller(mock_principals::alice()))
            .inject();

        let canister_info = NftCanister {
            name: String::from("xtc"),
            principal_id: mock_principals::xtc(),
            description: String::from("XTC is your cycles wallet."),
            thumbnail: String::from("https://google.com"),
            frontend: None,
            details: vec![(
                String::from("standard"),
                DetailValue::Text(String::from("Dank")),
            )],
        };

        assert!(add(canister_info.clone()).is_ok());
        assert_eq!(get_all()[0].name, canister_info.name);
    }
}
