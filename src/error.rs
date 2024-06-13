use cosmwasm_std::{Addr, StdError};
use thiserror::Error;

pub type ContractResult<T> = Result<T, ContractError>;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("Provided owner: {owner} does not match the sender")]
    InvalidOwnerDuringInstantiation { owner: Addr },
    #[error("{sender} does not have permission to modify traders")]
    SenderIsNotAdmin { sender: Addr },
    #[error("Trade permissions cannot be revoked from the contract deployer")]
    CannotRemoveContractDeployerAsTrader,
}
