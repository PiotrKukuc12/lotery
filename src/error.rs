use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Player already is participating in this game")]
    PlayerAlreadyInGame {},

    #[error("Game is finished")]
    FinishedGame {},

    #[error("Number is out of range")]
    NumberOutOfRange {},
}
