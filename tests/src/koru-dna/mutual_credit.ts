
import { Orchestrator, Player, Cell} from "@holochain/tryorama";
import { Element,HeaderHash,Entry,EntryContent,AgentPubKeyB64 } from "@holochain-open-dev/core-types";
import { config, installation, sleep } from '../utils';
import { fromPairs } from "lodash";
import * as msgpack from "@msgpack/msgpack";




interface Transaction {
  sender: Buffer
  receiver: Buffer,
  amount: number,
  balance: number
}
/*
class Transaction implements ITransaction {
  sender: AgentPubKeyB64,
  receive: AgentPubKeyB64,
  amount: number,
  balance: number

  constructor(sender: AgentPubKeyB64,
    receive: AgentPubKeyB64,
    amount: number,
    balance: number ) {
    this.sender = sender
}

  static isCar(unknownObject: unknown): boolean {
    const carKeys = Object.keys(new Transaction({ model: 'test', brand: 'test' }));

    if (typeof unknownObject !== 'object') return false;

    for (const unknownKey in unknownObject) {
      const hasKey = carKeys.some((k) => k === unknownKey);

      if (!hasKey) return false;
    }

    return true;
  }
}
*/
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



  //single tx
  // custom data we want back from the hApp
  // Alice pays Bob
  const headhash: HeaderHash = await alice.call(
        "mutual_credit",
        "countersign_tx",
        { 
          receiver: bob_happ.agent,
          amount: 10.0
        }
    );
  let elem:Element = await alice.call(
    "mutual_credit",
    "get_dht_entry",
    headhash
  );
  
  let tx = get_tx(elem)
  if (tx){
    t.equal(Buffer.compare(alice_happ.agent,tx.sender),0)
    t.equal(Buffer.compare(bob_happ.agent,tx.receiver),0)
    t.equal(tx.amount,10)
  }
  
  

  
  t.ok();
  /*

    console.log(bob_happ.agent)
  
    // Alice pays Bob
    const postHash2 = await alice.call(
          "mutual_credit",
          "countersign_tx",
          {
            receiver: bob_happ.agent,
            amount: 10.0
          }
      );
      t.ok();
      await sleep(10);
      console.log(bob_happ.agent)
  
      // Alice pays Bob
      const postHash3 = await alice.call(
            "mutual_credit",
            "countersign_tx",
            {
              receiver: bob_happ.agent,
              amount: 10.0
            }
        );
        t.ok();
        await sleep(10);
  
    // Bob gets the created post
    //const post = await bob.call("mutual_credit", "get_post", postHash);
    ///t.equal(post, postContents);
  */
  });
