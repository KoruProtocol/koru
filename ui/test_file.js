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

function randomChoice(arr) {
  return arr[Math.floor(Math.random() * arr.length)];
}

function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}
function deserializeHash(hash){
  return Base64.toUint8Array(hash.slice(1));
}

function serializeHash(hash) {
  return `u${Base64.fromUint8Array(hash, true)}`;
}

async function call_transaction(aws,cell_id,originator_cell,recipient_cell,amt) {

  try{
    res = await aws.callZome({
      cap: null,
      cell_id: cell_id,
      zome_name: 'mutual_credit',
      fn_name: 'countersign_tx',
      payload: {
        recepient: recipient_cell[0][1],
        amount: amt
      },
      provenance: cell_id[1],
    });
    
    return res
  }
  catch (e)
  {
    console.log("ERROR:")
    console.log(e)
    return e
  }
}

async function main(){
  let { AdminWebsocket, AppWebsocket, InstalledAppInfo } = await import('@holochain/conductor-api');
  let { Base64 } = await import('js-base64')


  let aws = await AppWebsocket.connect(`ws://localhost:8888`);

  let a_ports = get_admin_ports();

  console.log(a_ports)
  admins_sockets = []
  admin_cells = []

  for (admin_port of a_ports){
    //console.log(admin_port)
    let admin_socket = await AdminWebsocket.connect('ws://localhost:'+ admin_port);
    admins_sockets.push(admin_socket)

    let cell = await admin_socket.listCellIds();
    admin_cells.push(cell)
  }


  //console.log(admin_cells);



  let info = await aws.appInfo({
    installed_app_id: 'test-app',
  });
  const cell_id = info.cell_data[0].cell_id;


  let res = await call_transaction(aws,cell_id,cell_id,admin_cells[1],10)
  

  /*
  n_trials = 1
  for (let i = 0 ; i < n_trials; i++){
    let a = randomChoice(admin_cells)
    let b = randomChoice(admin_cells)

    console.log(a[0])
    console.log(cell_id)
    let res = await call_transaction(aws,cell_id,b,10);

    await sleep(5000)
  }

  */

}



main()