
import { Orchestrator, Player, Cell } from "@holochain/tryorama";
import { config, installation, sleep } from '../utils';

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

    const postContents = "My Post";


  //single tx

  console.log(bob_happ.agent)
  
  // Alice pays Bob
  const postHash = await alice.call(
        "mutual_credit",
        "countersign_tx",
        {
          recepient: bob_happ.agent,
          amount: 10.0
        }
    );
    t.ok();

    await sleep(10);

  

    console.log(bob_happ.agent)
  
    // Alice pays Bob
    const postHash2 = await alice.call(
          "mutual_credit",
          "countersign_tx",
          {
            recepient: bob_happ.agent,
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
              recepient: bob_happ.agent,
              amount: 10.0
            }
        );
        t.ok();
        await sleep(10);
  
    // Bob gets the created post
    //const post = await bob.call("mutual_credit", "get_post", postHash);
    ///t.equal(post, postContents);
});
