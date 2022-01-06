const fs = require('fs')

function get_admin_ports() {
  let ports = [];
  try {
    const data = fs.readFileSync('../.hc', 'utf8')
    n_agents = data.toString().split('\n').length - 1
    
    for(var i = 0; i < n_agents ;i++){
      let port = fs.readFileSync('../.hc_live_'+ i, 'utf8')
      ports.push(parseInt(port));
    }
    return ports
  } catch (err) {
    console.error(err)
  }
}

async function main(){
  let { AdminWebsocket, AppWebsocket, InstalledAppInfo } = await import('@holochain/conductor-api');
  let { Base64 } = await import('js-base64')

  function deserializeHash(hash){
    return Base64.toUint8Array(hash.slice(1));
  }

  function serializeHash(hash) {
    return `u${Base64.fromUint8Array(hash, true)}`;
  }
  
let aws = await AppWebsocket.connect(`ws://localhost:8888`);

let a_ports = get_admin_ports();

let admin1 = await AdminWebsocket.connect('ws://localhost:'+ a_ports[0]);
let admin2 = await AdminWebsocket.connect('ws://localhost:'+ a_ports[1]);




let cell1 = await admin1.listCellIds();
let cell2 = await admin2.listCellIds();



let info = await aws.appInfo({
  installed_app_id: 'test-app',
});
const cell_id = info.cell_data[0].cell_id;

try{
  res = await aws.callZome({
    cap: null,
    cell_id: cell_id,
    zome_name: 'mutual_credit',
    fn_name: 'countersign_tx',
    payload: {
      recepient: cell2[0][1],
      amount: 0.1 
    },
    provenance: cell_id[1],
  });
  
  console.log(res)
}
catch (e)
{
  console.log("ERROR:")
  console.log(e)
}
}


main()