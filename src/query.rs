use crate::{
    dydx::{
        querier::DydxQuerier,
        query::{DydxQueryWrapper, LiquidityTiersResponse, PerpetualClobDetailsResponse},
    },
    error::{ContractError, ContractResult},
    execute::{USDC_DENOM, USDC_ID},
    msg::{
        DydxSubaccountResponse, LpTokenBalanceResponse, TokenInfoResponse, TraderResponse,
        VaultOwnershipResponse, VaultsResponse, WithdrawalResponse, WithdrawalsResponse,
    },
    state::{LP_BALANCES, LP_TOKENS, STATE, VAULTS_BY_PERP_ID, WITHDRAWAL_QUEUES},
};
use cosmwasm_std::{Deps, Int256, Order, SignedDecimal, SignedDecimal256, StdResult};
use num_traits::{Signed, ToPrimitive};

pub fn perp_clob_details(
    deps: Deps<DydxQueryWrapper>,
    perp_id: u32,
) -> StdResult<PerpetualClobDetailsResponse> {
    let querier = DydxQuerier::new(&deps.querier);
    querier.query_perpetual_clob_details(perp_id)
}

pub fn liquidity_tiers(deps: Deps<DydxQueryWrapper>) -> StdResult<LiquidityTiersResponse> {
    let querier = DydxQuerier::new(&deps.querier);
    querier.query_liquidity_tiers()
}

pub fn trader(deps: Deps<DydxQueryWrapper>) -> StdResult<TraderResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(TraderResponse {
        trader: state.trader,
    })
}

pub fn withdrawals(deps: Deps<DydxQueryWrapper>, perp_id: u32) -> StdResult<WithdrawalsResponse> {
    let q = WITHDRAWAL_QUEUES
        .may_load(deps.storage, perp_id)?
        .ok_or(ContractError::MissingWithdrawalQueue { perp_id })
        .unwrap();
    let pos = query_dydx_position(deps, perp_id).unwrap();
    let subaccount_value = pos.asset_usdc_value + pos.perp_usdc_value;

    let lp_token_info = lp_token_info(deps, perp_id)?;
    let outstanding_lp_tokens = Int256::try_from(lp_token_info.total_supply).unwrap();

    let withdrawals: Vec<WithdrawalResponse> = q
        .into_iter()
        .map(|w| {
            let lp_fraction = SignedDecimal256::from_ratio(w.lp_tokens, outstanding_lp_tokens);

            WithdrawalResponse {
                recipient_addr: w.recipient_addr,
                lp_tokens: w.lp_tokens,
                usdc_equivalent: SignedDecimal256::from(subaccount_value) * lp_fraction,
            }
        })
        .collect();
    Ok(WithdrawalsResponse {
        withdrawal_queue: withdrawals,
    })
}

/// Queries all existing vaults and returns the `perp_id` of the underlying dYdX market.
pub fn vaults(deps: Deps<DydxQueryWrapper>) -> StdResult<VaultsResponse> {
    let vault = VAULTS_BY_PERP_ID.keys(deps.storage, None, None, Order::Ascending);
    let vaults: Vec<u32> = vault.map(|i| i.unwrap()).collect();
    Ok(VaultsResponse { vaults })
}

/// Queries a depositor's share of the vault with the provided `perp_id`.
pub fn vault_ownership(
    deps: Deps<DydxQueryWrapper>,
    perp_id: u32,
    depositor: String,
) -> StdResult<VaultOwnershipResponse> {
    let state = STATE.load(deps.storage)?;
    let pos = query_dydx_position(deps, perp_id).unwrap();

    let raw_depositor_balance = lp_balance(deps, perp_id, depositor)?;
    let lp_token_info = lp_token_info(deps, perp_id)?;

    Ok(VaultOwnershipResponse {
        subaccount_owner: state.contract.to_string(),
        subaccount_number: perp_id,
        asset_usdc_value: pos.asset_usdc_value,
        perp_usdc_value: pos.perp_usdc_value,
        depositor_lp_tokens: raw_depositor_balance.balance,
        outstanding_lp_tokens: lp_token_info.total_supply,
    })
}

pub fn dydx_subaccount(
    deps: Deps<DydxQueryWrapper>,
    owner: String,
    number: u32,
) -> StdResult<DydxSubaccountResponse> {
    let querier = DydxQuerier::new(&deps.querier);
    let subaccount = querier.query_subaccount(owner.clone(), number)?.subaccount;
    Ok(DydxSubaccountResponse { subaccount })
}

/// Queries an address' balance of the LP token for the specified perp market.
pub fn lp_balance(
    deps: Deps<DydxQueryWrapper>,
    perp_id: u32,
    address: String,
) -> StdResult<LpTokenBalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = LP_BALANCES
        .may_load(deps.storage, (perp_id, &address))?
        .unwrap_or_default();
    Ok(LpTokenBalanceResponse { perp_id, balance })
}

/// Queries the metadata of the LP token for the specified perp market.
/// This includes the total token supply.
pub fn lp_token_info(deps: Deps<DydxQueryWrapper>, perp_id: u32) -> StdResult<TokenInfoResponse> {
    let info = LP_TOKENS
        .may_load(deps.storage, perp_id)?
        .ok_or(ContractError::MissingLpToken { perp_id })
        .unwrap();
    let res = TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    };
    Ok(res)
}

/// Contains USDC value of a subaccount, split into perp and spot (USDC) components.
/// Ignores the value of open orders.
pub struct DydxPosition {
    pub asset_usdc_value: SignedDecimal,
    pub perp_usdc_value: SignedDecimal,
}

/// Queries dYdX for the asset and perp values of the subaccount.
/// Utilizes the market price to value the perp position.
/// Assumes that the subaccount only holds USDC and a single perp position in the provided market.
pub fn query_dydx_position(
    deps: Deps<DydxQueryWrapper>,
    perp_id: u32,
) -> ContractResult<DydxPosition> {
    let state = STATE.load(deps.storage)?;
    let querier = DydxQuerier::new(&deps.querier);

    // query subaccount + price state from dYdX
    let clob_resp = querier.query_perpetual_clob_details(perp_id)?;
    let perp_params = clob_resp.perpetual_clob_details.perpetual.params;
    let market_price_resp = querier.query_market_price(perp_params.market_id)?;
    let subaccount_resp = querier.query_subaccount(state.contract.to_string(), perp_id)?;
    let subaccount = subaccount_resp.subaccount;

    if market_price_resp.market_price.exponent > 0 {
        return Err(ContractError::InvalidPriceExponent {
            exponent: market_price_resp.market_price.exponent,
            perp_id,
        });
    };
    let price_exponent = (-1 * market_price_resp.market_price.exponent) as u32;
    let price =
        SignedDecimal::from_atomics(market_price_resp.market_price.price, price_exponent).unwrap();

    let usdc_position = subaccount
        .asset_positions
        .iter()
        .find(|p| p.asset_id == USDC_ID);
    let asset_usdc_value = match usdc_position {
        Some(p) => {
            let quantums: i128 = p.quantums.to_big_int().to_i128().unwrap_or(0);
            SignedDecimal::from_atomics(quantums, USDC_DENOM).unwrap()
        }
        None => SignedDecimal::zero(),
    };

    if perp_params.atomic_resolution > 0 {
        return Err(ContractError::InvalidPerpExponent {
            exponent: perp_params.atomic_resolution,
            perp_id,
        });
    };
    let perp_exponent = (-1 * perp_params.atomic_resolution) as u32;
    let perp_position = subaccount
        .perpetual_positions
        .iter()
        .find(|p| p.perpetual_id == perp_id);
    let perp_usdc_value = match perp_position {
        Some(p) => {
            let big_int = p.quantums.to_big_int();
            let is_neg = big_int.is_negative();
            let quantums: i128 = big_int.to_i128().unwrap_or(0);
            let position = SignedDecimal::from_atomics(quantums, perp_exponent).unwrap();
            if is_neg {
                position * price * SignedDecimal::negative_one()
            } else {
                position * price
            }
        }
        None => SignedDecimal::zero(),
    };

    let position = DydxPosition {
        asset_usdc_value,
        perp_usdc_value,
    };

    Ok(position)
}
