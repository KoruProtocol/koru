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

const CREDIT_LIMIT: f32 = -1000.0;


#[hdk_extern]
pub fn validate_create_entry_transaction(v:ValidateData) -> ExternResult<ValidateCallbackResult>{


    //unwrap
    let curr_elem = v.element.entry().as_option().ok_or(WasmError::Guest("failed to fetch entry from element".into()))?;

    let curr_tx = match curr_elem {
        Entry::CounterSign(_cs_data,cs_app) => {
            Transaction::try_from(SerializedBytes::from(cs_app.to_owned())).map_err(|err| WasmError::Guest(format!("Could not deserialize current element: {:?}", err)))
        },
        _ => Err(WasmError::Guest("Failed to open current element while validating".into()))
    }?;





    let val_pck = v.validation_package.ok_or(
                                    WasmError::Guest(String::from("Error fetching validation package")))?;

    let elems = val_pck.0;
    
    info!("validating!");

    let mut sums:HashMap<AgentPubKey,f32> = HashMap::new();
    //let mut contents: Vec<Transaction> = vec![];
    for e in elems {
        let ent= e.entry().as_option();
        let countersign = ent.ok_or(
            WasmError::Guest("unable to unwrap entry".into()))?;
        
        match countersign {
                Entry::CounterSign(_cs_data,cs_app) => {
                    let q: Transaction = Transaction::try_from(SerializedBytes::from(cs_app.to_owned()))?;

                    let origin = sums.entry(q.originater.clone()).or_insert(0.0);
                    *origin -= q.amount.clone();

                    let recip = sums.entry(q.recepient.clone()).or_insert(0.0);
                    *recip += q.amount.clone();
  
                },
                _ => debug!("validating non countersign entries?"),
            }
    

        
    }
    
    info!("{:?}",sums);

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
pub fn transact(tx_in:TxInput) -> ExternResult<HeaderHash>{
    info!("sending {}",tx_in.amount);

    let self_id = agent_info()?.agent_latest_pubkey;
    let self_pubkey: AgentPubKey = AgentPubKey::from(self_id);


    // secure timestamp fetching is in the works. timestamp below is insecure. see https://docs.rs/hdk/0.0.115/hdk/time/fn.sys_time.html
    // this fetches time on the host, therefore a host can easily change system clock to trick the system
    let timestamp: i64 = hdk::time::sys_time()?.as_millis();
    let entry = Transaction{
        originater: self_pubkey,
        recepient: tx_in.recepient,
        amount: tx_in.amount,
        timestamp: timestamp // kinda useless now with countersigning
    };
    create_entry(&entry)
}

#[hdk_extern]
pub fn countersign_tx(tx_in:TxInput) -> ExternResult<HeaderHash>{
    let self_id = agent_info()?.agent_latest_pubkey;
    let self_pubkey: AgentPubKey = AgentPubKey::from(self_id);
    info!("{:?} initiating tx countersign", self_pubkey.clone());
    
    info!("{:?}",tx_in);
    let timestamp: i64 = hdk::time::sys_time()?.as_millis();
    let entry = Transaction{ //should rename to tx
        originater: self_pubkey,
        recepient: tx_in.recepient,
        amount: tx_in.amount,
        timestamp: timestamp
    };

    info!("building preflight");
    let preflight_req = build_preflight(entry.clone())?;

    // sender locks the source chain
    info!("sender locking source chain");
    let my_response = match accept_countersigning_preflight_request(preflight_req)? {
        PreflightRequestAcceptance::Accepted(response) => Ok(response),
        _ => Err(WasmError::Guest(
            "There was an error when building the preflight_request for the transaction".into(),
        )),
    }?;


    let call_remote_result = call_remote(
        entry.recepient.clone(),
        zome_info()?.name,
        FunctionName("handle_preflight_req".into()),
        None,
        my_response.clone(),
    )?;

    info!("received response");

    match call_remote_result {
        ZomeCallResponse::Ok(z_response) => match z_response.decode::<PreflightResponse>()?        
        {
             cs_response => { 
                 info!("creating countersigned entry");

                let headhash = create_countersign_tx(entry, vec![my_response,cs_response])?;


                Ok(headhash)
            
            }
        },
        ZomeCallResponse::Unauthorized(..) => {
            info!("unauthorized zome call, missing cap grant");
            Err(WasmError::Guest("unauthorized due to missing cap grant".into()))
        },
        ZomeCallResponse::CountersigningSession(_err_str) => {
            info!("zome call countersign failed");
            Err(WasmError::Guest("remote call failed".into()))
        },
        ZomeCallResponse::NetworkError(_err_str) => {
            info!("zome call network error failed {}", _err_str);
            Err(WasmError::Guest("remote call failed".into()))
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
   
    info!("preflight request received, validating...");

    let req = cp_preflight_resp.request();

    //how can we have safer decoding?
    let tx: Transaction = SerializedBytes::from(UnsafeBytes::from(req.preflight_bytes().0.clone())).try_into()?;

    //validate tx



    let _validation = validate_tx(cp_preflight_resp.clone(),tx.clone());

    // need to handle validation result. For now its always valid

    // need to check if hash is outdated

    let self_response = match accept_countersigning_preflight_request(req.clone())?{
        PreflightRequestAcceptance::Accepted(response) => Ok(response),
        _ => Err(WasmError::Guest("Error accepting preflight countersign".into()))
    }?; // match to all variants for detailed error handling and debugging


    let responses = vec![cp_preflight_resp, self_response.clone()];

    create_countersign_tx(tx, responses)?;

    Ok(self_response)
}


pub fn create_countersign_tx(tx:Transaction,responses:Vec<PreflightResponse>) -> ExternResult<HeaderHash> {
    //create countersigned entry
    let session_data = CounterSigningSessionData::try_from_responses(responses).map_err(
        |cs_err| WasmError::Guest(cs_err.to_string()))?;
    let entry = Entry::CounterSign(Box::new(session_data),tx.clone().try_into()?);

    let _ehash = hash_entry(entry.clone())?;

    

    

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

fn validate_tx(preflight:PreflightResponse,_tx:Transaction) -> bool {
    // need sender's source chain, self source chain and DHT
    

    let a_s = preflight.agent_state();
    info!("{:?}", a_s.chain_top());


    // check tx timestamp is within countersign session timestamp
    true

}
/*

// may need to use this for post countersign linking
#[hdk_extern(infallible)]
pub fn link_tx(schedule: Option<Schedule>) -> Option<Schedule> {

}
*/