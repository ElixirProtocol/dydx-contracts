# dydx-contracts

[Getting Started](#startup) <br />
[Contract Design](#design) <br />
[Integration Tests](testing) 

<a name="startup"></a>
<h2 align="center">Getting Started</h2>

To setup the elixir-dydx contracts:
1. Ensure that [Rust is installed](https://www.rust-lang.org/tools/install)
2. Install the  Wasm Rust compiler backend using: `rustup target add wasm32-unknown-unknown`
3. Build using `cargo wasm`
4. Run unit tests using `cargo test`

For more details on how CosmWasm environments are configured, see the [CosmWasm Book](https://book.cosmwasm.com/setting-up-env.html).

<a name="design"></a>
<h2 align="center">Contract Design</h2>
The smart contract contained in this repository is intended to enable the Elixir protocol to engage in market-making activities with user funds in a permissionless manner. Specifically the contract must:

* Enable Elixir to trade on behalf of users. Trading should be done independently on multiple markets.
* Allow users to deposit and withdraw their funds (subject to restrictions based on Elixir's trading strategy).
* Track account value and user deposits such that changes in the PnL of the account are reflected when a user withdraws their funds. For example if PnL is positive, the user should be able to withdraw more than they deposited.

<h3 align="left">dYdX Subaccounts</h3>
Before going into more detail about the integration contract, it is useful to uderstand some details about how dYdX subaccounts work. dYdX subaccounts are cross-margin by default and USDC is the preferred collateral. They only accept messages (e.g deposit, place_order) sent by their `owner`. For dYdX's CosmWasm integration the creator of a subaccount is always the [smart contract who sent the message] (https://github.com/dydxprotocol/v4-chain/blob/0cb3e8d3d8ed1ea33df4f739041ca13b769f9edb/protocol/wasmbinding/msg_plugin.go#L45). Note that for dYdX a deposit results in subaccount creation.

<h3 align="left">Permissions</h3>

<h4 align="left">Trader</h4>
    The smart contract will always have an address with trading permissions. Typically this accounts/address is referred to as the `Trader`. The `Trader`:
    
* Is initialized as the contract deployer (but can be modified). Only the current `Trader` can set a new `Trader`. 
* Can initialize a `Vault` (which includes a contract-owned dYdX subaccount).
* Is the only address allowed to call the `market_make` entrypoint.
* Has permission to trade on all vaults/perp markets.
* Should always be an Elixir owned account.

Since the underlying dYdX subaccount is owned by the smart contract itself, the `Trader` does not have permission to withdraw user funds. For the same reason, the smart contract's `market_make` endpoint must be called to place/cancel orders.


<h4 align="left">Vaults</h4>
A `Vault` is the concept that the smart contract uses to coordinate tracking user deposits and trading. Each `Vault`:

1. Corresponds to a dYdx perp market. See `perp_id`.
2. Has one contract-owned subaccount associated with it.
3. Has a unique LP token that is minted when users deposit into the `Vault` and burned when users withdraw. The LP token is used to determine a user's share in the `Vault`.
4. Has a withdrawal queue associated with it.
5. Can only be created by a `Trader`

Despite the fact that dYdX subaccounts are cross-margined by default, 1 and 2 implies that each `Vault` is isolated to its associated market. Due to this, `perp_id` and `subaccount_number` are interchangeable.

<h3 align="left">User Deposits</h3>
Users may only deposit and withdraw USDC. 
As stated above, the contract uses the minting and burning of LP tokens to keep track of deposits. LP tokens are managed according to the invariant:

```(user LP tokens / total LP tokens) = (user deposit-or-withdraw value USDC / vault value USDC)```.

As a simple example, if a user deposited $10 USDC into the `Vault` and the USDC value of the `Vault` was $100 as a result, the depositor would own 10% of all outstanding LP tokens. If a user owns 10% of all outstanding LP tokens, they are entitled to withdraw 10% of the USDC value of the `Vault`. This mechanism ensures that withdrawals properly reflect the changes in `Vault` value during the lifetime of a user's deposit. Users can deposit at any time, but withdrawals are queued and later fulfilled by the `Trader`. This is done to prevent withdrawals from disrupting Elixir's trading.

<h3 align="left">Trading</h3>
All trading is done by the `Trader` using the `market_make` entrypoint. `market_make` sends multiple `PlaceOrderV1` and `CancelOrderV1` messages for the specified subaccount/perp market (again `perp_id` and `subaccount_number` are interchangeable). Due to gas considerations, dYdX has restricted the amount of orders placed to be at most 3 bids and 3 asks. The `market_make` entrypoint also has a check to keep leverage <= 1x. If leverage is already over 1x due to market movements, the check will just enforce that any new orders woulld decrease leverage.

<a name="testing"></a>
<h2 align="center">Integration Testing</h2>
<br />

Since the smart contract is intended to be run on dYdX chain, the unit tests require heavy mocking. Fortunately, dYdX provides a dockerized version of their blockchain that can be used to run integration tests. To setup this environment:

1. `git clone https://github.com/dydxprotocol/v4-chain`
2. `git checkout feature/cosmwasm` or ensure that the branch has Wasm support (typically by seeing if `/protocol/wasmbinding` is present)
3. Run the chain locally using the `README.md` from dYdX's repo
4. Use the messages in `example_messages.md` from this repo, but replace `wasmd` with `./build/dydxprotocold`