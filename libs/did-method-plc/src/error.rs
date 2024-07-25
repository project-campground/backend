#[derive(Debug, thiserror::Error)]
pub enum PLCError {
    #[error("Http {0}: {1}")]
    Http(u16, String),

    #[error("Misordered operation")]
    MisorderedOperation,

    #[error("Recovery too late")]
    LateRecovery,

    #[error("Signature is invalid")]
    InvalidSignature,

    #[error("Operation is invalid")]
    InvalidOperation,

    #[error("Key is invalid")]
    InvalidKey,

    #[error("Key is malformed")]
    MalformedKey,

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}