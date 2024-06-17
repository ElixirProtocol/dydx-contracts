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
    SenderIsNotOwner { sender: Addr },
    #[error("{sender} does not have permission to create vaults")]
    SenderCannotCreateVault { sender: Addr },
    #[error("{sender} does not have permission to change the vault trader")]
    SenderCannotChangeVaultTrader { sender: Addr },
    #[error("{sender} does not have permission to freeze the vault")]
    SenderCannotFreezeVault { sender: Addr },
    #[error("{sender} does not have permission to thaw the vault")]
    SenderCannotThawVault { sender: Addr },
    #[error("Tried to set {new_trader} as trader of vault: {perp_id}, but they do not have permission to trade")]
    NewVaultTraderMustBeApproved { new_trader: Addr, perp_id: u32 },
    #[error("The new trader of vault: {perp_id} must be different from the old one")]
    NewVaultTraderMustNotBeCurrentTrader { perp_id: u32 },
    #[error("{sender} does not have permission to place trades. Only {expected} can place trades  perp_id: {perp_id}")]
    SenderCannotPlaceTrade {
        sender: Addr,
        expected: String,
        perp_id: u32,
    },
    #[error("Trade permissions cannot be revoked from the contract deployer")]
    CannotRemoveContractDeployerAsTrader,

    #[error("Vault must be halted to change trader")]
    VaultMustBeHaltedToChangeTrader,

    #[error("Vault already initialized for perp_id: {perp_id}")]
    VaultAlreadyInitialized { perp_id: u32 },
    #[error("Vault with perp_id: {perp_id} is not initialized")]
    VaultNotInitialized { perp_id: u32 },
    #[error("Vault already frozen for perp_id: {perp_id}")]
    VaultAlreadyFrozen { perp_id: u32 },
    #[error("Vault already open for perp_id: {perp_id}")]
    VaultAlreadyOpen { perp_id: u32 },

    #[error("Market with id: {perp_id} is not configured")]
    InvalidMarket { perp_id: u32 },
}
