use hdk::prelude::*;
use crate::countersign::Transaction;


pub fn extract_tx_from_cs_entry(cs_entry: Entry) -> ExternResult<Transaction> {
    
    match cs_entry {
            Entry::CounterSign(_cs_data,cs_app) => {
            Ok(Transaction::try_from(SerializedBytes::from(cs_app.to_owned()))?)

            },
            _ => Err(WasmError::Guest("Error extracting tx from countersign entry: not of type Entry::CounterSign".into())),
        }
}

pub fn get_latest_sc_tx() -> ExternResult<Option<Transaction>> {
    // get the most recent transaction to compute the new account balance
    let filter = ChainQueryFilter::new()
    .include_entries(true)
    .entry_type(EntryType::App(AppEntryType::new(
        entry_def_index!(Transaction)?,
        zome_info()?.id,
        EntryVisibility::Public,
    )));

    let mut res = query(filter)?;
    //info!("{:?}",res);
    let temp = res.pop();
    match temp {
        Some(elem) => {
            let elem_entry = elem.entry();
            let temp = elem_entry.as_option().ok_or(WasmError::Guest(format!("Error unwrapping element into entry: {:?}",&elem)))?;
            let tx = extract_tx_from_cs_entry(temp.clone())?;
            Ok(Some(tx))},
        None => Ok(None)
    }

}
#[hdk_extern]
pub fn get_sourcechain_balance(_:()) -> ExternResult<f32> {
    
    let self_id = agent_info()?.agent_latest_pubkey;
    let self_pubkey: AgentPubKey = AgentPubKey::from(self_id);
    
    let filter = ChainQueryFilter::new()
    .include_entries(true)
    .entry_type(EntryType::App(AppEntryType::new(
        entry_def_index!(Transaction)?,
        zome_info()?.id,
        EntryVisibility::Public,
    )));

    let res = query(filter)?;
    
    let mut balance: f32 = 0.0;

    for ele in res {
        let elem_entry = ele.entry();
        let temp = elem_entry.as_option().ok_or(WasmError::Guest(format!("Error unwrapping element into entry: {:?}",&ele)))?;
        let tx = extract_tx_from_cs_entry(temp.clone())?;

        if tx.sender == self_pubkey {
            balance -= tx.amount;
        } else if tx.receiver == self_pubkey {
            balance += tx.amount;
        }
    }

    Ok(balance)
}

pub fn get_other_sc_balance(agent:AgentPubKey) -> ExternResult<f32> {

    let call_remote_result = call_remote(
        agent,
        zome_info()?.name,
        FunctionName("get_sourcechain_balance".into()),
        None,
        (),
    )?;

    //debug!("received response");

    match call_remote_result {
        ZomeCallResponse::Ok(z_response) => match z_response.decode::<f32>()?        
        {
             sc_balance => { 
                Ok(sc_balance)
            
            }
        },
        ZomeCallResponse::Unauthorized(cell,zome,func,agent) => {
            Err(WasmError::Guest(format!("{} is unauthorized for calling {} in {}:{}", agent,func,zome,cell)))
        },
        ZomeCallResponse::CountersigningSession(e) => {
            Err(WasmError::Guest(format!("remote call for countersign failed: {}", e)))
        },
        ZomeCallResponse::NetworkError(e) => {
            Err(WasmError::Guest(format!("network error during remote call: {}", e)))
        }
    }

}