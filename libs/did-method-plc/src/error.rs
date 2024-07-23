#[derive(Debug, thiserror::Error)]
pub enum PLCError {
    #[error("Failed to create PLC: {0}")]
    Create(u16),

    #[error("Failed to deactivate PLC: {0}")]
    Deactivated(String),

    #[error("Failed to update PLC: {0}")]
    Update(String),
    
    #[error("Failed to recover PLC: {0}")]
    Recover(String),

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

    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}