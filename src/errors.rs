use std::str::Utf8Error;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, Clone)]
pub enum ZMQSeqListenerError {
    #[error("Error from ZMQ library")]
    ZMQError(#[from] zmq::Error),
    #[error("Error parsing msg")]
    ParseError(#[from] Utf8Error),
    #[error("Message error: {0}")]
    MsgError(String),
    #[error("Invalid char code: {0} in mempoolseq message")]
    CharCodeError(String),
    #[error("Topic error, topic should be \"secuence\", but its: {0}")]
    TopicError(String),
}
