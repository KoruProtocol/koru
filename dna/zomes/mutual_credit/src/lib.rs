use hdk::prelude::*;
use std::fmt;
use std::collections::HashMap;
//use hdk::prelude::holo_hash::*;


#[hdk_entry(id = "transaction", 
            required_validations = 20, 
            required_validation_type = "sub_chain" )]
#[derive(Clone)]
pub struct Transaction{
    originater: AgentPubKey,
    recepient: AgentPubKey,
    amount: f32,
    timestamp: i64
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}---{}--->{} @ {}", self.originater,self.amount, self.recepient,self.timestamp)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxInput{
    recepient: AgentPubKey,
    amount:f32
}

entry_defs![
    Transaction::entry_def(),
    Anchor::entry_def()
];

const CREDIT_LIMIT: f32 = -1000.0; // credit limit is hardcoded for now


#[hdk_extern]
pub fn validate_create_entry_transaction(v:ValidateData) -> ExternResult<ValidateCallbackResult>{


    //unwrap to be validated entry
    let curr_elem = v.element.entry().as_option().ok_or(WasmError::Guest("failed to fetch entry from element".into()))?;

    let curr_tx = extract_tx_from_cs_entry(curr_elem.clone())?;

    
    // unwrap validation package and calculate balance for all transactions
    let val_pck = v.validation_package.ok_or(
                                    WasmError::Guest(String::from("Error fetching validation package")))?;

    let elems = val_pck.0;



    let mut sums:HashMap<AgentPubKey,f32> = HashMap::new();
    //let mut contents: Vec<Transaction> = vec![];
    for e in elems {
        let ent= e.entry().as_option();
        let countersign = ent.ok_or(
            WasmError::Guest("unable to unwrap entry".into()))?;
        

        let tx = extract_tx_from_cs_entry(countersign.clone())?;

        let origin = sums.entry(tx.originater.clone()).or_insert(0.0);
        *origin -= tx.amount.clone();

        let recip = sums.entry(tx.recepient.clone()).or_insert(0.0);
        *recip += tx.amount.clone();


        
    }
    
    //debug !("validating with data:{:?}",sums);



    if !sums.is_empty() {
        let temp = sums.get(&curr_tx.originater).ok_or(WasmError::Guest).map_err(|_| WasmError::Guest("error fetching sums from hashmap".into()))?;
        if ( temp - curr_tx.amount) < CREDIT_LIMIT {
            info!("{}",temp - curr_tx.amount);
            return Ok(ValidateCallbackResult::Invalid("Sender's credit limit exceeded".into()))
        }

    }

    

    Ok(ValidateCallbackResult::Valid)
    

   

}


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
pub fn countersign_tx(tx_in:TxInput) -> ExternResult<HeaderHash>{
    let self_id = agent_info()?.agent_latest_pubkey;
    let self_pubkey: AgentPubKey = AgentPubKey::from(self_id);

    
    let timestamp: i64 = hdk::time::sys_time()?.as_millis();
    let entry = Transaction{ //should rename to tx
        originater: self_pubkey,
        recepient: tx_in.recepient,
        amount: tx_in.amount,
        timestamp: timestamp
    };

    //debug!("transaction started {:?}",entry);

    //debug!("building preflight");
    let preflight_req = build_preflight(entry.clone())?;

    // sender locks the source chain
    //debug!("sender locking source chain");
    let my_response = match accept_countersigning_preflight_request(preflight_req)? {
        PreflightRequestAcceptance::Accepted(response) => Ok(response),
        PreflightRequestAcceptance::UnacceptableFutureStart => Err(WasmError::Guest("Start time too far into the future".into())),
        PreflightRequestAcceptance::UnacceptableAgentNotFound => Err(WasmError::Guest("Countersigning agent not found".into())),
        PreflightRequestAcceptance::Invalid(e) => Err(WasmError::Guest(format!("Invalid preflight {}",e)))
    }?;


    let call_remote_result = call_remote(
        entry.recepient.clone(),
        zome_info()?.name,
        FunctionName("handle_preflight_req".into()),
        None,
        my_response.clone(),
    )?;

    //debug!("received response");

    match call_remote_result {
        ZomeCallResponse::Ok(z_response) => match z_response.decode::<PreflightResponse>()?        
        {
             cs_response => { 
                info!("creating countersigned entry");

                let headhash = create_countersign_tx(entry, vec![my_response,cs_response])?;


                Ok(headhash)
            
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

fn build_preflight(tx:Transaction) -> Result<PreflightRequest,WasmError>{

   
    let ehash = hash_entry(&tx)?;
    let times = session_times_from_millis(1000)?;

    let entry_type = EntryType::App(AppEntryType::new(
        EntryDefIndex::from(0),
        zome_info()?.id,
        EntryVisibility::Public,
    ));
    let header_base = HeaderBase::Create(CreateBase::new(entry_type));

    let countersign_agents = vec![
            (tx.originater.clone() ,vec![]),
            (tx.recepient.clone() ,vec![])
            ];

    let bytes = SerializedBytes::try_from(tx.clone())?;
    let preflight_bytes = PreflightBytes(bytes.bytes().to_vec());
    


    let preflight_request = PreflightRequest::try_new(
        ehash,
        countersign_agents,
        Some(0),
        times,
        header_base,
        preflight_bytes,
    )
    .map_err(|err| WasmError::Guest(format!("Could not create preflight request: {:?}", err)))?;


    Ok(preflight_request)
}
#[hdk_extern]
pub fn handle_preflight_req(cp_preflight_resp: PreflightResponse) -> ExternResult<PreflightResponse> {
   
    //debug!("preflight request received, validating...");

    let req = cp_preflight_resp.request();


    let tx: Transaction = SerializedBytes::from(UnsafeBytes::from(req.preflight_bytes().0.clone())).try_into()?;

    //validate tx will raise error if something is wrong
    validate_tx(cp_preflight_resp.clone(),tx.clone())?;

    // need to check if hash is outdated?

    let self_response = match accept_countersigning_preflight_request(req.clone())?{
        PreflightRequestAcceptance::Accepted(response) => Ok(response),
        PreflightRequestAcceptance::UnacceptableFutureStart => Err(WasmError::Guest("Start time too far into the future".into())),
        PreflightRequestAcceptance::UnacceptableAgentNotFound => Err(WasmError::Guest("Countersigning agent not found".into())),
        PreflightRequestAcceptance::Invalid(e) => Err(WasmError::Guest(format!("Invalid preflight {}",e)))
    }?;


    let responses = vec![cp_preflight_resp, self_response.clone()];

    create_countersign_tx(tx, responses)?;

    Ok(self_response)
}


pub fn create_countersign_tx(tx:Transaction,responses:Vec<PreflightResponse>) -> ExternResult<HeaderHash> {
    //create countersigned entry
    let session_data = CounterSigningSessionData::try_from_responses(responses).map_err(
        |cs_err| WasmError::Guest(cs_err.to_string()))?;
    let entry = Entry::CounterSign(Box::new(session_data),tx.clone().try_into()?);

    let res = HDK.with(|h| {
        h.borrow().create(CreateInput::new(
            (&tx).into(),
            entry.clone(),
            // Countersigned entries MUST have strict ordering.
            ChainTopOrdering::Strict,
        ))
    })?;


    Ok(res)
}

fn validate_tx(preflight:PreflightResponse,_tx:Transaction) -> ExternResult<bool> {

    let a_state= preflight.agent_state();
    let mut headhash = a_state.chain_top();


    //for debugging
    let mut cs_source_txs :Vec<Transaction> = vec![];

    let opt_elem = get(headhash.clone(),GetOptions::latest())?;

    // opt_elem provides None for InitZomeComplete header, shouldnt it at least return a header instead of returning None?
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
                        if  author == tx.originater {
                            credit_sum -= tx.amount;
                        }
                        else if author == tx.recepient {
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
/*
fn recursive_walk_sourcechain(in_vector:&mut Vec<Transaction>, headhash:HeaderHash) -> ExternResult<()> {

    debug!("calling recursive walk");
    let opt_elem = get(headhash.clone(),GetOptions::latest())?;
    let elem = opt_elem.ok_or(WasmError::Guest("Error fetching entry from countersign state chain_top hash".into()))?;

    let entry = elem.entry().as_option().ok_or(WasmError::Guest("failed to fetch entry from element".into()))?;


    let tx = extract_tx_from_cs_entry(entry.clone());

    match tx {
        Ok(tx) => in_vector.push(tx),
        _ => debug!("tried to extract tx from non countersign entry")
    }   

    let prev_head = elem.header().prev_header().ok_or(WasmError::Guest("error fetching previous header".into()))?;

    recursive_walk_sourcechain(in_vector, prev_head.clone())
}

*/

fn extract_tx_from_cs_entry(cs_entry: Entry) -> ExternResult<Transaction> {
    
    match cs_entry {
            Entry::CounterSign(_cs_data,cs_app) => {
               Ok(Transaction::try_from(SerializedBytes::from(cs_app.to_owned()))?)

            },
            _ => Err(WasmError::Guest("Error extracting tx from countersign entry: not of type Entry::CounterSign".into())),
        }
}
/*

// may need to use this for post countersign linking
#[hdk_extern(infallible)]
pub fn link_tx(schedule: Option<Schedule>) -> Option<Schedule> {

}
*/