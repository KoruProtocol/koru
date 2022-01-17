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