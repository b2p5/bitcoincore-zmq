use anyhow::Result;
use bitcoincore_zmq::ZmqSeqListener;
use ctrlc;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use url::Url;
use std::time::Duration;
use bitcoincore_zmqsequence::check::{ClientConfig, NodeChecker};

fn main() -> Result<()> {
    let checker = NodeChecker::new(&ClientConfig {
        cookie_auth_path: None,
        ip_addr: "localhost".to_string(),
        user: "anon".to_string(),
        passwd: "anon".to_string(),
    })?;

    println!("Waiting to node Ok");
    checker.wait_till_node_ok(2, Duration::from_secs(5))?;
    println!("Node Ok");

    let stop_th = Arc::new(AtomicBool::new(false));
    let stop_th2 = stop_th.clone();
    ctrlc::set_handler(move || stop_th2.store(true, Ordering::SeqCst))?;
    let zmqseqlistener = ZmqSeqListener::start(&Url::from_str("tcp://localhost:28332")?)?;
    
    let kk_1 = zmqseqlistener.rx.recv()?;
    println!("{:?}", kk_1);

    while !stop_th.load(Ordering::SeqCst) {
        let kk = zmqseqlistener.rx.recv()?;
        println!("{:?}", kk);

        //Get txid
        let txid = match kk {
            bitcoincore_zmq::MempoolSequence::TxAdded { txid, .. } => txid,
            _ => continue,
        };

        println!("txid: {:?}", txid);
        println!("\n");

    }

    Ok(())

}





    // let has_index = checker.check_tx_index()?;
    // if !has_index {
    //     return Err(anyhow!(
    //         "bitcoind must have transactions index enabled, add txindex=1 to bitcoin.conf file"
    //     ));
    // }




            // // Dirección y puerto del nodo Bitcoin Core con ZMQ habilitado
        // let zmq_endpoint = "tcp://127.0.0.1:28332"; // Ajusta esto según tu configuración

        // // Dirección de Bitcoin para la cual deseas obtener datos de transacciones en la mempool
        // let mempool_address = txid;

        // // Crear un contexto ZMQ
        // let context = Context::new();

        // // Crear un socket ZMQ de tipo SUB para suscribirse a eventos
        // let socket: Socket = context.socket(SUB)
        //     .expect("Error al crear el socket ZMQ");
        // socket.connect(zmq_endpoint)
        //     .expect("Error al conectar al nodo Bitcoin Core");

        // // Suscribirse a eventos relacionados con la dirección de Bitcoin en la mempool
        // socket.set_subscribe(format!("mempool {}", mempool_address).as_bytes())
        //     .expect("Error al establecer la suscripción");

        // println!("Esperando eventos para la dirección en la mempool: {}", mempool_address);

        // // Esperar y procesar eventos
        // // Esperar a recibir un mensaje
        // let msg = socket.recv_msg(0)
        //     .expect("Error al recibir el mensaje");
        // // Convertir el mensaje a texto (asumiendo que es UTF-8)
        // // let msg_str = std::str::from_utf8(&msg.as_bytes())
        // //     .expect("Error al convertir el mensaje a texto");
        // // Imprimir el mensaje recibido
        // println!("Mensaje ZMQ recibido desde la mempool: {}", msg.as_str().unwrap());
