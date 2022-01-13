const fs = require('fs')

async function main(){
  let { AdminWebsocket, AppWebsocket, InstalledAppInfo } = await import('@holochain/conductor-api');
  let { Base64 } = await import('js-base64')

  let a_ports = get_admin_ports();
  admins_sockets = []
  admin_cells = []
  let aws_list = []
  let i = 0
  for (admin_port of a_ports){
    //console.log(admin_port)
    let admin_socket = await AdminWebsocket.connect('ws://localhost:'+ admin_port);
    admins_sockets.push(admin_socket)

    let cell = await admin_socket.listCellIds();
    admin_cells.push(cell)

    try {
      let aws_temp = await admin_socket.attachAppInterface({port:8880+i});

      let aws_connection = await AppWebsocket.connect(`ws://localhost:`+(8880+i))
      aws_list.push(aws_connection)
    }catch(e){
      //console.log(e) // address is already in use, connect instead
      let aws_connection = await AppWebsocket.connect(`ws://localhost:`+(8880+i))
      aws_list.push(aws_connection)
    }

    console.log(aws_list)
    i+=1;
  }


  console.log("transacting")
  
  n_trials = 100
  for (let i = 0 ; i < n_trials; i++){
    let a_i = between(0,aws_list.length-1);
    let b_i = between(0,aws_list.length-1)
    let amt = between(0,100)
    console.log("%i ---> %i ---> %i",a_i,amt,b_i)
    let res = await call_transaction(aws_list[a_i],admin_cells[a_i][0],admin_cells[b_i],amt)
    
    
    await sleep(1000)
  }


}



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

function between(min, max) {  
  return Math.floor(
    Math.random() * (max - min + 1) + min
  )
}
async function call_transaction(aws,originator_cell,recipient_cell,amt) {

  try{
    res = await aws.callZome({
      cap: null,
      cell_id: originator_cell,
      zome_name: 'mutual_credit',
      fn_name: 'countersign_tx',
      payload: {
        recepient: recipient_cell[0][1],
        amount: amt
      },
      provenance: originator_cell[1],
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

main()