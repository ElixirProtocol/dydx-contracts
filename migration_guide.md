## Why do we need to migrate?
Currently, the integration smart contract is the "sender" of messages that place/cancel orders for a subaccount.
This means that an `ExecuteMsg` call is necessary any time an Elixir-owned subaccount needs to manage its orders.
Unfortunately, the gas costs associated with `ExecuteMsg` are much higher than a wallet-signed place or cancel. As such, dYdX will implement rate-limits that restrict how often an Elixir trader would be able to send orders.

However, dYdX is planning to add a feature called "permissioned keys" that will allow an appointed signer to place and cancel orders for a given subaccount. If the Elixir smart contract used this feature, it would be able to avoid the rate-limits on `ExecuteMsg` thus allowing Elixir to trade more frequently and on more markets. Migration is necessary to use a new version of the smart contract that supports permissioned keys.

## What will the migration do?

Migration copies state from the old version of smart contract to the new version. For the dYdX integration contract this includes everything in the `state.rs` file that tracks user positions and handles withdrawals. The smart contract address stays the same throughout the migration, but the contract's code id will change. All subaccounts will remain unchanged.

## Migration steps

1. Compile the updated smart contract using `cargo wasm`. Before compiling, make sure that the version in the `Cargo.toml` is strictly greater than the previously deployed version.
2. Deploy the new contract. Do not instantiate the new contract, otherwise you will need to re-deploy.
3. Using `wasmd query wasm list-code`, determine which code id the new contract has.
4. Migrate the contract using the following command: `wasmd tx wasm migrate <old-contract-address> "<code-id>" "{}" ...`. All state will be preserved and the entrypoints will reflect whatever is in the new contract.
Observe that the contract has migrating by running `wasmd query wasm list-contract-by-code "<code-id>"`. There will be nothing at the old id and the smart contract with it's original address at the new id.

**The simplicity of this migration relies upon the fact that the state in the new and old contracts is identical. If any difference in state is required, please refer to the resources below to properly set up the migration.**

- https://medium.com/cosmwasm/cosmwasm-for-ctos-ii-advanced-usage-ee04ce95d1d0
- https://github.com/CosmWasm/cosmwasm/blob/a0cf296c43aa092b81457d96a9c6bc2ab223f6d3/contracts/hackatom/src/contract.rs#L37-L48
- https://docs.archway.io/developers/cosmwasm-documentation/smart-contracts/migration