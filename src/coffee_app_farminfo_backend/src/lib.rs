use ic_cdk::{
    query, update
};
//use ic_cdk::api::Principal;
use candid::CandidType;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::vec::Vec;
type FarmsStore = BTreeMap<String, Farm>;

#[derive(Clone, Debug, Default, CandidType)]
struct Farm {
    pub name: String,
    pub geohash: String,
    pub farmer: String,
}

thread_local! {
    static FARMS_STORE: RefCell<FarmsStore> = RefCell::default();
}

#[update]
fn add_farm(name: String, geohash: String) {
    match get_farm(name.clone()){
        Some(_) => {
            panic!("{} already exists",name);
        }
        None => {
            let farmer_id = ic_cdk::api::caller().to_string();
            let key = name.clone();
            let new_farm = Farm{
                name,
                geohash,
                farmer: farmer_id,
            };
            FARMS_STORE.with(|farms_store| {
                farms_store
                    .borrow_mut()
                    .insert(key,new_farm);
            });
        }
    } 
}

#[query]
fn get_farms() -> Vec<Farm>{
    //let mut farms: Vec<Farm> = Vec::new();
    FARMS_STORE.with(|farms_store| {
        farms_store.borrow().values().cloned().collect()
    })
}

#[query]
fn get_farm(name: String) -> Option <Farm> {
    FARMS_STORE.with(|farms_store|{
        farms_store
            .borrow()
            .get(&name).cloned()
    })
}


ic_cdk::export_candid!();