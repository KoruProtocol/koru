
import { Orchestrator } from "@holochain/tryorama";

import mutual_credit from './koru-dna/mutual_credit';


let orchestrator: Orchestrator<any>;

orchestrator = new Orchestrator();
mutual_credit(orchestrator);
orchestrator.run();





