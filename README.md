# bitcoincore-zmq
A ZeroMQ pubsequence listener.
Allows keeping a synchronized copy of a Bitcoin Core node's mempool.
Example:

```
use anyhow::Result;
use bitcoincore_zmq::ZmqSeqListener;
use ctrlc;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use url::Url;

fn main() -> Result<()> {
    let stop_th = Arc::new(AtomicBool::new(false));
    let stop_th2 = stop_th.clone();
    ctrlc::set_handler(move || stop_th2.store(true, Ordering::SeqCst))?;
    let zmqseqlistener = ZmqSeqListener::start(&Url::from_str("tcp://192.168.3.2:29000")?)?;
    while !stop_th.load(Ordering::SeqCst) {
        let zmq_seq = zmqseqlistener.receiver().recv()?;
        println!("{:?}", zmq_seq);
    }
    Ok(())
}
```
