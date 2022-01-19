
use hdk::prelude::*;
use std::fmt;
use crate::utils::{get_sourcechain_balance,get_other_sc_balance};




#[hdk_entry(id = "transaction", 
required_validations = 6, 
required_validation_type = "sub_chain" )]
#[derive(Clone)]
pub struct Transaction{
    pub sender: AgentPubKey,
    pub receiver: AgentPubKey,
    pub amount: f32,
    pub sender_balance: f32,
    pub receiver_balance: f32
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}---{}--->{}", self.sender.to_string(),self.amount, self.receiver.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TxInput{
    receiver: AgentPubKey,
    amount:f32
}






#[hdk_extern]
pub fn countersign_tx(tx_in:TxInput) -> ExternResult<HeaderHash>{
    let self_id = agent_info()?.agent_latest_pubkey;
    let self_pubkey: AgentPubKey = AgentPubKey::from(self_id);

    //let latest_tx = get_latest_sc_tx()?; //latest tx on sender's source chain

    // balance calculation is incorrect. when calculating balance, we dont take into account received credits AND we use sender's balance which in the case of tx reception is not the current author
    
    let self_balance = get_sourcechain_balance(())?;
    let other_balance = get_other_sc_balance(tx_in.receiver.clone())?;

    let entry = Transaction {
        sender: self_pubkey,
        receiver: tx_in.receiver,
        amount: tx_in.amount,
        sender_balance: self_balance - tx_in.amount,
        receiver_balance: other_balance + tx_in.amount
    };

    //  debug!("transaction started {:?}",entry);

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
        entry.receiver.clone(),
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
            (tx.sender.clone() ,vec![]),
            (tx.receiver.clone() ,vec![])
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


    //tx.receiver_balance = 99.9;

    //Optional counterparty validation. Not needed due to peer validation, currently 
    //validate_tx(cp_preflight_resp.clone(),tx.clone())?;

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

