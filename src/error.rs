use cosmwasm_std::{Addr, StdError};
use thiserror::Error;

pub type ContractResult<T> = Result<T, ContractError>;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    StdError(#[from] StdError),
    #[error("Provided owner: {owner} does not match the sender")]
    InvalidOwnerDuringInstantiation { owner: Addr },
    #[error("{sender} does not have permission to modify trader")]
    SenderCannotModifyTrader { sender: Addr },
    #[error("The new trader must be different from the old one")]
    NewTraderMustNotBeCurrentTrader,
    #[error("An invalid address was provided: {addr}")]
    InvalidAddress { addr: String },
    #[error("{addr} cannot place/cancel trades")]
    SenderIsNotTrader { addr: String },
    #[error("{sender} does not have permission to create vaults")]
    SenderCannotCreateVault { sender: Addr },
    #[error("{sender} does not have permission to freeze the vault")]
    SenderCannotFreezeVault { sender: Addr },
    #[error("{sender} does not have permission to thaw the vault")]
    SenderCannotThawVault { sender: Addr },
    #[error("{sender} does not have permission to process withdrawals")]
    SenderCannotProcessWithdrawals { sender: Addr },
    #[error("Tried to set {new_trader} as trader of vault: {perp_id}, but they do not have permission to trade")]
    NewVaultTraderMustBeApproved { new_trader: Addr, perp_id: u32 },
    #[error("{sender} does not have permission to place trades. Only {expected} can place trades  perp_id: {perp_id}")]
    SenderCannotPlaceTrade {
        sender: Addr,
        expected: String,
        perp_id: u32,
    },
    #[error("Trade permissions cannot be revoked from the contract deployer")]
    CannotRemoveContractDeployerAsTrader,

    #[error("Vault with perp_id: {perp_id} must be open to place a trade")]
    VaultIsNotOpen { perp_id: u32 },

    #[error("Vault already initialized for perp_id: {perp_id}")]
    VaultAlreadyInitialized { perp_id: u32 },
    #[error("Vault with perp_id: {perp_id} is not initialized")]
    VaultNotInitialized { perp_id: u32 },
    #[error("Vault already frozen for perp_id: {perp_id}")]
    VaultAlreadyFrozen { perp_id: u32 },
    #[error("Vault already open for perp_id: {perp_id}")]
    VaultAlreadyOpen { perp_id: u32 },

    #[error("The subaccount for vault with perp_id: {perp_id} has more that one perp position")]
    VaultSubaccountHasMoreThanOnePerpPosition { perp_id: u32 },

    #[error("The subaccount for vault with perp_id: {perp_id} has more that one asset position")]
    VaultSubaccountHasMoreThanOneAssetPosition { perp_id: u32 },

    #[error("The subaccount of the order id must be owned by this smart contract")]
    InvalidOrderIdSubaccountOwner,

    #[error("Parsed an invalid exponent for oracle price: {exponent} for market with perp_id: {perp_id}")]
    InvalidPriceExponent { exponent: i32, perp_id: u32 },

    #[error("Parsed an invalid exponent: {exponent} for market with perp_id: {perp_id}")]
    InvalidPerpExponent { exponent: i32, perp_id: u32 },

    #[error("Market with id: {perp_id} is not configured")]
    InvalidMarket { perp_id: u32 },

    #[error("Tried to deposit an invalid coin: {coin_type}. Only USDC is accepted")]
    InvalidCoin { coin_type: String },

    #[error("Tried to deposit an invalid amount of: {coin_type}, {amount}")]
    InvalidDepositAmount { coin_type: String, amount: u128 },

    #[error("Tried to withdraw an invalid amount of: {coin_type}, {amount}")]
    InvalidWithdrawalAmount { coin_type: String, amount: u128 },

    #[error("Could not find LP tokens with perp_id: {perp_id} for {user}")]
    LpTokensNotFound { user: Addr, perp_id: u32 },

    #[error("Only one coin type can be deposited at a time")]
    CanOnlyDepositOneCointype {},

    // Token errors
    #[error("Unauthorized")]
    Unauthorized {},
    #[error("Minting cannot exceed the cap")]
    CannotExceedCap {},
    #[error("could not find LP token for vault with perp_id: {perp_id}")]
    MissingLpToken { perp_id: u32 },

    #[error("could not find withdrawal_queue for vault with perp_id: {perp_id}")]
    MissingWithdrawalQueue { perp_id: u32 },
}
