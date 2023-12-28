use ic_cdk::{
    query, update
};
//use ic_cdk::api::Principal;
use candid::CandidType;
use candid::Deserialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::vec::Vec;

type FarmsStore = BTreeMap<String, Farm>;
type WorkersStore = BTreeMap<String, BTreeMap<String, Person>>;

//'Farmer','Farm Manager','Field Manager','Factory Manager','Receiving Manager','Harvester'
#[derive(Clone, Debug, CandidType)]
struct Farm {
    pub name: String,
    pub geohash: String,
    pub farmer: String,
}

#[derive(Clone, Debug, CandidType)]
struct Person {
    pub name: String,
    pub id: String,
    pub role: Role,
    pub image_cid: String,
    pub approved: bool,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
enum Role{
    Farmer,
    FarmManager,
    FieldManager,
    FactoryManager,
    ReceivingManager,
    Harvester
}

thread_local! {
    static FARMS_STORE: RefCell<FarmsStore> = RefCell::default();
    static WORKERS_STORE: RefCell<WorkersStore> = RefCell::default();
}

fn get_farmer_id(farm_name: &String) -> String{
    if let Some(farm) = get_farm(farm_name.clone()){
        farm.farmer
    } else {
        panic!("Farm does not exist");
    }
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

#[update]
fn update_worker(farm: String, name: String, id: String, role: Role, image_cid: String) {
    let caller_id = ic_cdk::api::caller().to_string();
    let farmer_id = get_farmer_id(&farm);
    match role {
        Role::Farmer => {
            if caller_id != farmer_id{
                panic!("Only farmer can update farmer role");
            }
        }
        _ => {}
    }
    WORKERS_STORE.with(|workers_store|{
        let mut workers_store_mut = workers_store.borrow_mut();
        if workers_store_mut.contains_key(&farm){
            let worker_map = workers_store_mut.get_mut(&farm).unwrap();
            if worker_map.contains_key(&id){
                let worker = worker_map.get_mut(&id).unwrap();
                worker.name = name;
                worker.role = role;
                worker.image_cid = image_cid;
                worker.approved = caller_id == farmer_id;
                let _worker = worker.clone();
                worker_map.insert(id,_worker);
            } else {
                let worker = Person {
                    name,
                    id: id.clone(),
                    role,
                    image_cid,
                    approved: caller_id == farmer_id,
                };
                worker_map.insert(id,worker);
            }
        } else {
            let mut worker_map: BTreeMap<String, Person> = BTreeMap::new();
            let worker = Person {
                name,
                id: id.clone(),
                role,
                image_cid,
                approved: caller_id == farmer_id,
            };
            worker_map.insert(id.clone(), worker);
            workers_store_mut.insert(farm.clone(), worker_map);
        }
    });
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


#[query]
fn get_worker(farm: String, id: String) -> Option <Person>  {
    WORKERS_STORE.with(|workers_store|{
        workers_store
            .borrow()
            .get(&farm)
            .and_then(|persons|{
                persons.get(&id).cloned()
            })
    })
}

#[query]
fn get_workers(farm: String) -> Vec<Person> {
    WORKERS_STORE.with(|workers_store|{
        if let Some(persons) = workers_store.borrow().get(&farm){
            persons.values().cloned().collect()
        } else {
            Vec::new()
        }
    })
}



ic_cdk::export_candid!();