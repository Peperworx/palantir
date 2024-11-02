use thiserror::Error;



#[derive(Debug, Error)]
pub enum PalantirError {
    #[error("error initializing server endpoint")]
    UnableToInitializeServer,
}