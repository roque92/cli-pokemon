use thiserror::Error;

#[derive(Debug, Error)]
pub enum PetcliError {
    #[error("state file not found or corrupted: {0}")]
    StateLoad(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("sprite fetch failed: {0}")]
    SpriteFetch(String),

    #[error("sprite decode failed: {0}")]
    SpriteDecode(String),

    #[error("no active pet in party")]
    NoPet,

    #[error("party is full (max 6 pokemon)")]
    PartyFull,

    #[error("offline mode active — sprite unavailable")]
    Offline,
}
