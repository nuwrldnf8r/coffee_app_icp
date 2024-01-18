use ic_cdk::{
    query, update
};
//use ic_cdk::api::Principal;
use candid::CandidType;
use candid::Deserialize;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::vec::Vec;
use digest::Digest;
use sha2::Sha256;


type FarmsStore = BTreeMap<[u8; 32], Farm>;
type WorkersStore = BTreeMap<[u8; 32], BTreeMap<String, Person>>;
type FarmLookup = BTreeMap<String,[u8; 32]>;

type DataStore = BTreeMap<[u8; 32],Data>;
type FarmDataIndex = BTreeMap<[u8; 32],BTreeMap<i64,[u8; 32]>>;


//'Farmer','Farm Manager','Field Manager','Factory Manager','Receiving Manager','Harvester'
#[derive(Clone, Debug, CandidType)]
struct Farm {
    pub name: String,
    pub metadata: String,
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
    Harvester,
    Scout,
}

#[derive(Clone, Debug, CandidType)]
struct Data {
    pub id: String,
    pub farm_id: [u8; 32],
    pub ts: i64,
    pub metadata: String,
}

thread_local! {
    static FARMS_STORE: RefCell<FarmsStore> = RefCell::default();
    static WORKERS_STORE: RefCell<WorkersStore> = RefCell::default();
    static FARM_LOOKUP: RefCell<FarmLookup> = RefCell::default();
    static DATA_STORE: RefCell<DataStore> = RefCell::default();
    static FARM_DATA_INDEX: RefCell<FarmDataIndex> = RefCell::default();
}

fn get_id(s_id: &String) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(s_id.as_bytes());
    let result = hasher.finalize();
    let mut hash = [0; 32];
    hash.copy_from_slice(&result);
    hash
}

fn get_farmer_id(farm_name: &String) -> String{
    if let Some(farm) = get_farm(farm_name.clone()){
        farm.farmer
    } else {
        panic!("Farm does not exist");
    }
}

fn get_farm_from_id(id: &[u8; 32]) -> Option <Farm>{
    FARMS_STORE.with(|farms_store|{
        farms_store
            .borrow()
            .get(id).cloned()
    })
}

#[update]
fn add_farm(name: String, metadata: String) {
    match get_farm(name.clone()){
        Some(_) => {
            panic!("{} already exists",name);
        }
        None => {
            let farmer_id = ic_cdk::api::caller().to_string();
            let key = name.clone();
            let new_farm = Farm{
                name,
                metadata,
                farmer: farmer_id,
            };
            FARMS_STORE.with(|farms_store| {
                farms_store
                    .borrow_mut()
                    .insert(get_id(&key),new_farm);
            });
        }
    } 
}

#[update]
fn update_worker(farm: String, name: String, id: String, role: Role, image_cid: String) {
    let farm_id = get_id(&farm);
    let caller_id = ic_cdk::api::caller().to_string();
    let farmer_id = get_farmer_id(&farm);
    let mut is_new = false;
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
        
        if workers_store_mut.contains_key(&farm_id){
            let worker_map = workers_store_mut.get_mut(&farm_id).unwrap();
            if worker_map.contains_key(&id){
                let worker = worker_map.get_mut(&id).unwrap();
                worker.name = name;
                worker.role = role;
                worker.image_cid = image_cid;
                worker.approved = caller_id == farmer_id;
                let _worker = worker.clone();
                worker_map.insert(id.clone(),_worker);
            } else {
                let worker = Person {
                    name,
                    id: id.clone(),
                    role,
                    image_cid,
                    approved: caller_id == farmer_id,
                };
                worker_map.insert(id.clone(),worker);
                is_new = true;
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
            workers_store_mut.insert(farm_id.clone(), worker_map);
            is_new = true;
        }
    });
    if is_new {
        FARM_LOOKUP.with(|farm_lookup|{
            farm_lookup.borrow_mut().insert(id,farm_id);
        })
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
    get_farm_from_id(&get_id(&name))
}


#[query]
fn get_farm_from_workerid(id: String) -> Option <Farm> {
    FARM_LOOKUP.with(|farm_lookup|{
        match farm_lookup.borrow().get(&id)  {
            Some(farm_id) =>{
                get_farm_from_id(&farm_id)
            }
            None => {
                return None;
            }
        }
    })
}

#[query]
fn get_workers_from_workerid(id: String) -> Vec<Person> {
    let farm = get_farm_from_workerid(id).unwrap();
    get_workers(farm.name)
}


#[query]
fn get_worker(farm: String, id: String) -> Option <Person>  {
    WORKERS_STORE.with(|workers_store|{
        workers_store
            .borrow()
            .get(&get_id(&farm))
            .and_then(|persons|{
                persons.get(&id).cloned()
            })
    })
}

#[query]
fn get_workers(farm: String) -> Vec<Person> {
    WORKERS_STORE.with(|workers_store|{
        if let Some(persons) = workers_store.borrow().get(&get_id(&farm)){
            persons.values().cloned().collect()
        } else {
            Vec::new()
        }
    })
}

#[query]
fn id() -> String{
    ic_cdk::api::caller().to_string()
}

#[update]
fn delete_worker(id: String) {
    match get_farm_from_workerid(id.clone()){
        Some(farm) => {
            let caller_id = ic_cdk::api::caller().to_string();
            if caller_id != farm.farmer {
                panic!("Only farmer can remove workers");
            }
            let farm_id = get_id(&farm.name);
            WORKERS_STORE.with(|workers_store|{
                let mut workers_store_mut = workers_store.borrow_mut();
                if workers_store_mut.contains_key(&farm_id){
                    let worker_map = workers_store_mut.get_mut(&farm_id).unwrap();
                    worker_map.remove(&id);
                }
            })
        }
        None => {}
    }
}

#[update]
fn delete_farm(name: String) {
    let farm_id = get_id(&name);
    let caller_id = ic_cdk::api::caller().to_string();
    match get_farm_from_id(&farm_id){
        Some(farm) => {
            if caller_id != farm.farmer {
                panic!("Only farmer can delete farm");
            }
        }
        None => return
    }
    FARMS_STORE.with(|farms_store|{
        farms_store
            .borrow_mut().remove(&farm_id);
        WORKERS_STORE.with(|workers_store|{
            workers_store.borrow_mut().remove(&farm_id);
        })          
    })
}

fn index_data(id: [u8; 32], ts: i64, farm_id: [u8; 32]) {  
    FARM_DATA_INDEX.with(|farm_data_index|{
        let mut mut_index = farm_data_index.borrow_mut();
        if mut_index.contains_key(&farm_id){
            let ts_map = mut_index.get_mut(&farm_id).unwrap();
            ts_map.insert(ts,id);
        } else {
            let mut ts_map: BTreeMap<i64, [u8; 32]> = BTreeMap::new();
            ts_map.insert(ts,id);
            mut_index.insert(farm_id,ts_map);
        }

    })
}

#[update]
fn add_data(id: String, ts: i64, farm: String, metadata: String) {
    let caller_id = ic_cdk::api::caller().to_string();
    let _farm = get_farm_from_workerid(caller_id);
    if _farm.unwrap().name != farm.clone() {
        panic!("Only workers on the correct farm can add data"); 
        //later we look up role and filter data type into who can update
    }
    
    DATA_STORE.with(|data_store|{
        let mut mut_data_store = data_store.borrow_mut();
        let _id = get_id(&id);
        let farm_id = get_id(&farm);
        if !mut_data_store.contains_key(&_id){
            let data = Data {
                id: id.clone(),
                farm_id: farm_id.clone(),
                ts: ts.clone(),
                metadata,
            };
            mut_data_store.insert(_id.clone(), data);
            index_data(_id,ts,farm_id);
        } else {
            panic!("Data for this id already exists");
        }
    })
}

#[query]
fn get_data_by_id(id: String) -> Data{
    let _id = get_id(&id);
    DATA_STORE.with(|data_store|{
        data_store.borrow().get(&_id).cloned().unwrap()
    })
}

#[query]
fn get_data_by_farm(farm: String, ts_start: i64, ts_end: i64) -> Vec<Data>{
    let farm_id = get_id(&farm);
    FARM_DATA_INDEX.with(|farm_data_index|{
        let farm_index = farm_data_index.borrow();
        if farm_index.contains_key(&farm_id){
            let map = farm_index.get(&farm_id);
            let range_query: Vec<Data> = map.unwrap()
            .range(ts_start..=ts_end)
            .map(|(_,&v)| {
                DATA_STORE.with(|data_store|{
                    data_store.borrow().get(&v).cloned().unwrap()
                })
            })
            .collect();
            range_query
        } else {
            let empty: Vec<Data> = vec![];
            empty
        }
    })
}


ic_cdk::export_candid!();

//https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.icp0.io/?id=nqvbb-mqaaa-aaaak-afhsq-cai