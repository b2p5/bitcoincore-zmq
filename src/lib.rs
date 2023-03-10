use crate::errors::ZMQSeqListenerError;
use std::any::Any;
use std::str;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Barrier};
use std::thread;
use std::thread::JoinHandle;

use url::Url;

mod errors;

type Msg = Vec<Vec<u8>>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MempoolSequence {
    BlockConnection {
        block_hash: String,
        zmq_seq: u32,
    },
    BlockDisconnection {
        block_hash: String,
        zmq_seq: u32,
    },
    TxRemoved {
        txid: String,
        mp_seq_num: u64,
        zmq_seq: u32,
    },
    TxAdded {
        txid: String,
        mp_seq_num: u64,
        zmq_seq: u32,
    },
    SeqError {
        error: ZMQSeqListenerError,
    },
}

impl TryFrom<Msg> for MempoolSequence {
    type Error = ZMQSeqListenerError;

    fn try_from(msg: Vec<Vec<u8>>) -> Result<Self, Self::Error> {
        //Safe to do next three unwraps
        if msg.len() < 3 {
            return Err(ZMQSeqListenerError::MsgError("msg.len() < 3".to_string()));
        };
        let topic = str::from_utf8(msg.get(0).unwrap())?;
        let body = msg.get(1).unwrap();
        let zmqseq = msg.get(2).unwrap();

        if topic.ne("sequence") {
            return Err(ZMQSeqListenerError::TopicError(topic.to_string()));
        }
        let zmq_seq = parse_zmq_seq_num(zmqseq)?;
        let char = *body.get(32).ok_or(ZMQSeqListenerError::MsgError(
            "Not enough size, expected [u8;33]".to_string(),
        ))?;
        match char {
            // "C"
            67 => Ok(MempoolSequence::BlockConnection {
                block_hash: hex::encode(&body[..32]),
                zmq_seq,
            }),
            // "D"
            68 => Ok(MempoolSequence::BlockDisconnection {
                block_hash: hex::encode(&body[..32]),
                zmq_seq,
            }),
            // "R"
            82 => Ok(MempoolSequence::TxRemoved {
                txid: hex::encode(&body[..32]),
                mp_seq_num: parse_mp_seq_num(body)?,
                zmq_seq,
            }),
            // "A"
            65 => Ok(MempoolSequence::TxAdded {
                txid: hex::encode(&body[..32]),
                mp_seq_num: parse_mp_seq_num(body)?,
                zmq_seq,
            }),
            ch => Err(ZMQSeqListenerError::CharCodeError(ch.to_string())),
        }
    }
}

fn parse_mp_seq_num(value: &[u8]) -> Result<u64, ZMQSeqListenerError> {
    let ch: [u8; 8] = match value[33..41].try_into() {
        Ok(val) => Ok(val),
        Err(_) => Err(ZMQSeqListenerError::MsgError(
            "Not enough size, expected [u8;41]".to_string(),
        )),
    }?;
    Ok(u64::from_le_bytes(ch))
}

fn parse_zmq_seq_num(value: &[u8]) -> Result<u32, ZMQSeqListenerError> {
    let ch: [u8; 4] = match value[..4].try_into() {
        Ok(val) => Ok(val),
        Err(_) => Err(ZMQSeqListenerError::MsgError(
            "Not enough size, expected [u8;4]".to_string(),
        )),
    }?;
    Ok(u32::from_le_bytes(ch))
}

pub struct ZmqSeqListener {
    rx: Receiver<MempoolSequence>,
    stop: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

impl ZmqSeqListener {
    pub fn start(zmq_address: &Url) -> Result<Self, ZMQSeqListenerError> {
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB)?;
        subscriber.connect(zmq_address.as_str())?;
        subscriber.set_subscribe(b"sequence")?;
        let stop_th = Arc::new(AtomicBool::new(false));
        let stop = stop_th.clone();
        let (tx, rx) = channel();
        //Use a barrier, Zmq is "slow joiner"
        let barrier = Arc::new(Barrier::new(2));
        let barrierc = barrier.clone();
        let thread = thread::spawn(move || {
            let mut wait = true;
            while !stop_th.load(Ordering::SeqCst) {
                let mpsq = match receive_mpsq(&subscriber, &mut wait, &barrier) {
                    Ok(mpsq) => mpsq,
                    Err(e) => MempoolSequence::SeqError { error: e },
                };
                tx.send(mpsq).unwrap();
            }
        });
        barrierc.wait();
        Ok(ZmqSeqListener { rx, stop, thread })
    }

    pub fn stop(self) -> Result<(), Box<dyn Any + Send + 'static>> {
        self.stop.store(true, Ordering::SeqCst);
        self.thread.join()
    }

    pub fn receiver(&self) -> &Receiver<MempoolSequence> {
        &self.rx
    }
}

fn receive_mpsq(
    subscriber: &zmq::Socket,
    wait: &mut bool,
    barrier: &Arc<Barrier>,
) -> Result<MempoolSequence, ZMQSeqListenerError> {
    let res = {
        let msg = subscriber.recv_multipart(0)?;
        if *wait {
            barrier.wait();
            *wait = false;
        }

        let mpsq = MempoolSequence::try_from(msg)?;
        Ok(mpsq)
    };
    res
}
