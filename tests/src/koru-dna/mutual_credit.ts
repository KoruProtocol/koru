
import { Orchestrator, Player, Cell} from "@holochain/tryorama";
import { Element,HeaderHash,Entry,EntryContent,AgentPubKeyB64, Dictionary, serializeHash } from "@holochain-open-dev/core-types";
import { config, installation, sleep } from '../utils';
import { fromPairs, kebabCase } from "lodash";
import * as msgpack from "@msgpack/msgpack";




interface Transaction {
  sender: Buffer
  receiver: Buffer,
  amount: number,
  sender_balance: number
}


async function transact(sender,receiver,amount): Promise<Transaction | undefined>{
  let headhash: HeaderHash = await sender.call(
    "mutual_credit",
    "countersign_tx",
    { 
      receiver: receiver.cellId[1],
      amount: amount
    }
  );
  let elem:Element = await sender.call( //when fetching dht entry from receiver or other, cant get valid element?
    "mutual_credit",
    "get_dht_entry",
    headhash
  );
  let tx = get_tx(elem)

  if (tx){return tx}
}


function get_tx(elem:Element): Transaction | undefined {
  if(elem){
    if (elem['entry']) {
      let entry: Entry = elem['entry'];
      let cont = entry['Present']['entry'][0]['preflight_request']['preflight_bytes']
      let tx: Transaction = <Transaction>msgpack.decode(cont) //maybe should validate before casting
      return tx
    }
    

  }
}


async function all_tx_from_sc(cell_arr:Array<Cell>): Promise<Object> {


  let out_dict = {};

  for (var cell of cell_arr){

    let statedump = await cell.stateDump()
    statedump[0]['source_chain_dump']['elements'].forEach(function (curr) {
      let head = curr['header']['type']
      let ent = curr['entry']
  
      if (head == 'Create' && ent != null){
        
        if (ent['entry_type'] == "CounterSign"){
  
          let cont = curr['entry']['entry'][0]['preflight_request']['preflight_bytes']
          let tx: Transaction = <Transaction>msgpack.decode(cont)
          out_dict[serializeHash(curr['header']['entry_hash'])] = tx
        }
        
      }
    })
  
  }
  return out_dict;

}


function calc_balances(tx_arr): Object {
  let out_dict:Object = {};

  for (var tx of tx_arr){

    let sender = serializeHash(tx.sender)
    let sender_check = out_dict[sender]
    let receiver = serializeHash(tx.receiver)
    let receiver_check = out_dict[receiver]

    if (sender_check == undefined) {
      out_dict[sender] = 0
    }
    if (receiver_check == undefined){
      out_dict[receiver] = 0
    }
    
    out_dict[sender] -= tx.amount
    out_dict[receiver] += tx.amount
  }


  return out_dict
}


function human_balances(balances,humans): Object {

  let human_balance = {}
  for (var key in balances){
    let name = humans[key]
    human_balance[name] = balances[key]
  }
  return human_balance
}
export default (orchestrator: Orchestrator<any>) => 
  orchestrator.registerScenario("mutual_credit tests", async (s, t) => {
    // Declare two players using the previously specified config, nicknaming them "alice" and "bob"
    // note that the first argument to players is just an array conductor configs that that will
    // be used to spin up the conductor processes which are returned in a matching array.
    const [alice_player, bob_player, ben_player]: Player[] = await s.players([config, config, config]);

    // install your happs into the conductors and destructuring the returned happ data using the same
    // array structure as you created in your installation array.
    const [[alice_happ]] = await alice_player.installAgentsHapps(installation);
    const [[bob_happ]] = await bob_player.installAgentsHapps(installation);
    const [[ben_happ]] = await ben_player.installAgentsHapps(installation);

    await s.shareAllNodes([alice_player, bob_player, ben_player]);

    const alice = alice_happ.cells.find(cell => cell.cellRole.includes('/koru-dna.dna')) as Cell;
    const bob = bob_happ.cells.find(cell => cell.cellRole.includes('/koru-dna.dna')) as Cell;
    const ben = ben_happ.cells.find(cell => cell.cellRole.includes('/koru-dna.dna')) as Cell;

  

    let id_to_name = {}
    id_to_name[serializeHash(alice_happ.agent)] = "Alice"
    id_to_name[serializeHash(bob_happ.agent)] = "Bob"
    id_to_name[serializeHash(ben_happ.agent)] = "Ben"

  
  // single transactions
  // Alice pays Bob
  let tx = await transact(alice,bob,10)
    console.log(tx)

  t.equal(tx!=undefined,true)
  if(tx) {
    t.equal(Buffer.compare(alice_happ.agent,tx.sender),0)
    t.equal(Buffer.compare(bob_happ.agent,tx.receiver),0)
    t.equal(tx.amount,10)
    t.equal(tx.sender_balance,-10)
  }
  
  
  // Ben pays Alice
  let tx2 = await transact(ben,alice,20);
  t.equal(tx2!=undefined,true)
  if (tx2){
    console.log(tx2)
    t.equal(tx2.sender_balance,-20)
  }



  let tx3 = await transact(alice,ben,20);
  
  let tx_list = await all_tx_from_sc([alice,ben,bob])
  let balances = calc_balances(Object.values(tx_list))
  let h_bal = human_balances(balances,id_to_name)
  console.log(h_bal)

  /*
  let tx = await transact(alice,bob,10,ben)
  if (tx){
  t.equal(Buffer.compare(alice_happ.agent,tx.sender),0)
  t.equal(Buffer.compare(bob_happ.agent,tx.receiver),0)
  t.equal(tx.amount,10)
  }
  */

  });
