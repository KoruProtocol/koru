use hdk::prelude::*;
//use hdk::prelude::holo_hash::*;


pub mod countersign;
pub mod validation;
pub mod utils;



entry_defs![
    countersign::Transaction::entry_def(),
    Anchor::entry_def()
];








#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {

    // grant unrestricted access to accept_cap_claim so other agents can send us claims
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.name, "handle_preflight_req".into()));
    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;


    Ok(InitCallbackResult::Pass)
}




#[hdk_extern]
pub fn get_dht_entry(hash:HeaderHash) -> ExternResult<Element> {
    must_get_valid_element(hash)
}




