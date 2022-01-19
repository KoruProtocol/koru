use hdk::prelude::*;
use std::collections::HashMap;
use crate::utils::extract_tx_from_cs_entry;



const CREDIT_LIMIT: f32 = -10000.0; // credit limit is hardcoded for now




#[hdk_extern]
pub fn validate_create_entry_transaction(v:ValidateData) -> ExternResult<ValidateCallbackResult>{


    //unwrap to be validated entry
    let curr_elem = v.element.entry().as_option().ok_or(WasmError::Guest("failed to fetch entry from element".into()))?;

    let curr_tx = extract_tx_from_cs_entry(curr_elem.clone())?;

    
    // unwrap validation package and calculate balance for all transactions
    let val_pck = v.validation_package.ok_or(
                                    WasmError::Guest(String::from("Error fetching validation package")))?;

    let elems = val_pck.0;
    
    //whose source chain are we looking at?
    let author = v.element.header().author().clone();
    


    debug!("validating with {} elements",elems.len());
    //validation type: sub_chain provides entry authors source chain entries.
    let mut sums:HashMap<AgentPubKey,f32> = HashMap::new();
    //let mut contents: Vec<Transaction> = vec![];
    let mut  _i = 0;
    for e in elems {
        let ent= e.entry().as_option();
        let countersign = ent.ok_or(
            WasmError::Guest("unable to unwrap entry".into()))?;
        

        let tx = extract_tx_from_cs_entry(countersign.clone())?;

        //debug!("validation data@{}:{:?}",i,tx);

        let origin = sums.entry(tx.sender.clone()).or_insert(0.0);
        *origin -= tx.amount.clone();

        let recip = sums.entry(tx.receiver.clone()).or_insert(0.0);
        *recip += tx.amount.clone();
        _i+=1;
    }




    if !sums.is_empty() {

        // receiver never transacted with sender, no prior history. So here we assume the sender has a balance of 0. Peer validation is done 2x in this case for recipient and sender.
        let source_sum = match sums.get(&author) {
            Some(val) => val.clone(),
            None => 0.0
        };



        //debug !("validating for author: {:?} with a balance of {}",temp,sender_sum - curr_tx.amount);
        if author == curr_tx.sender {

            let peer_balance = source_sum - curr_tx.amount;
            if peer_balance != curr_tx.sender_balance {


                debug!("Sender balance mismatch for tx:{:?}. \n Peer balance:{}  \n Validating agent {}",curr_tx,peer_balance, author);
                return Ok(ValidateCallbackResult::Invalid(format!("Sender balance doesn't match peers (peer {} != tx {})",
                    peer_balance,curr_tx.sender_balance
                    )))

            }

            if peer_balance < CREDIT_LIMIT {
                return Ok(ValidateCallbackResult::Invalid(format!("Sender's credit limit of {} exceeded: {}",CREDIT_LIMIT, source_sum - curr_tx.amount)))
            }
        } else if author == curr_tx.receiver {
            let peer_balance = source_sum + curr_tx.amount;

            if peer_balance != curr_tx.receiver_balance {
                debug!("Receiver balance mismatch for tx:{:?} \n Peer balance: {} \n Validating agent {}",curr_tx,peer_balance, author);
                return Ok(ValidateCallbackResult::Invalid(format!("Receiver balance doesn't match peers (peer {} != tx {})",
                    peer_balance,curr_tx.receiver_balance
                    )))
            }
        }
     


    } 

    

    Ok(ValidateCallbackResult::Valid)
    
    

   

}

/*
fn validate_tx(preflight:PreflightResponse,_tx:Transaction) -> ExternResult<bool> { //countersigning validation

    let a_state= preflight.agent_state();
    let mut headhash = a_state.chain_top();


    //for debugging
    let mut cs_source_txs :Vec<Transaction> = vec![];

    let opt_elem = get(headhash.clone(),GetOptions::latest())?;

    // opt_elem provides None for InitZomeComplete header, shouldnt it at least return a header instead of returning None?
    // initzomecomplete created on remote zome calls. Can make arbitrary call to trigger initzome.
    // this validation function makes too many get calls, leave validation to peer validation (instead of countersign)
    // hashbound sourcechain query - upcoming
    let mut elem = opt_elem.ok_or(WasmError::Guest("Error fetching entry from countersign state chain_top hash".into()))?;

    let mut credit_sum:f32 = 0.0;

    while elem.header().header_type() != HeaderType::Dna {


        match elem.header() {
            Header::Create(_) => {
                let entry = elem.entry().as_option().ok_or(WasmError::Guest("failed to fetch entry from element".into()))?;
                let tx = extract_tx_from_cs_entry(entry.clone());

                match tx {
                    Ok(tx) => {
                        
                        let author = elem.header().author().clone();
                        if  author == tx.sender {
                            credit_sum -= tx.amount;
                        }
                        else if author == tx.receiver {
                             credit_sum += tx.amount;
                        }

                        cs_source_txs.push(tx);
                    },
                    _ => ()
                };   

            },
            _ => (),
        };

        headhash = elem.header().prev_header().ok_or(WasmError::Guest("error fetching previous header".into()))?;
        let opt_elem = get(headhash.clone(),GetOptions::latest())?;
        elem = opt_elem.ok_or(WasmError::Guest("Error fetching entry from countersign state chain_top hash".into()))?;
            
    }


    debug!("Balance is at {}",credit_sum);

    if credit_sum < CREDIT_LIMIT {

        debug!("sender surpasses credit limit in cs validate");
        return Ok(false)
    }

    // check tx timestamp is within countersign session timestamp
    Ok(true)

}
*/