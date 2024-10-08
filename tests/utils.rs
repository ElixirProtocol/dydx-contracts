use std::cell::RefCell;
use std::collections::HashMap;

use cosmwasm_std::{
    coin,
    testing::{MockApi, MockStorage},
    to_json_binary, Addr, Api, Attribute, Binary, BlockInfo, CustomMsg, CustomQuery, Event,
    Querier, Storage,
};
use cw_multi_test::{
    error::{bail, AnyResult},
    App, AppBuilder, AppResponse, BankKeeper, Contract, ContractWrapper, CosmosRouter,
    DistributionKeeper, Executor, GovFailingModule, IbcFailingModule, Module, StakeKeeper,
    StargateFailingModule, SudoMsg, WasmKeeper,
};
use elixir_dydx_integration::dydx::proto_structs::{
    AssetPosition, PerpetualPosition, SubaccountId,
};
use elixir_dydx_integration::dydx::query::{DydxQuery, DydxQueryWrapper};
use elixir_dydx_integration::{
    dydx::{
        msg::DydxMsg,
        proto_structs::{
            ClobPair, MarketPrice, Metadata, Perpetual, PerpetualClobDetails,
            PerpetualClobMetadata, PerpetualMarketType, PerpetualParams, Status, Subaccount,
        },
        serializable_int::SerializableInt,
    },
    msg::{ExecuteMsg, InstantiateMsg},
};
use num_bigint::BigInt;
use num_traits::Zero;
use serde::de::DeserializeOwned;

pub const TEST_CONTRACT_ADDR: &str = "contract0";

pub type ElixirTestApp = App<
    BankKeeper,
    MockApi,
    MockStorage,
    TestDydx,
    WasmKeeper<DydxMsg, DydxQueryWrapper>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    StargateFailingModule,
>;

pub fn test_setup() -> (ElixirTestApp, u64, Vec<Addr>) {
    let contract = ContractWrapper::new(
        elixir_dydx_integration::contract::execute,
        elixir_dydx_integration::contract::instantiate,
        elixir_dydx_integration::contract::query,
    );
    let b: Box<dyn Contract<DydxMsg, DydxQueryWrapper>> = Box::new(contract);

    let test_dydx = TestDydx::new();
    // let wasm_keeper = DydxWasmKeeper::new(EXECUTE_MSG, QUERY_MSG, SUDO_MSG);
    let app_builder = AppBuilder::new_custom();

    let mut app = app_builder.with_custom(test_dydx).build(|router, _, _| {
        router.custom.mock_subaccounts.borrow_mut().insert(
            0,
            Subaccount {
                id: Some(SubaccountId {
                    owner: TEST_CONTRACT_ADDR.to_string(),
                    number: 0,
                }),
                asset_positions: vec![],
                perpetual_positions: vec![],
                margin_enabled: true,
            },
        );
    });
    let code_id = app.store_code(b);

    let mock_api = MockApi::default();
    let owner = mock_api.addr_make("owner");
    let user1 = mock_api.addr_make("user1");
    let user2 = mock_api.addr_make("user2");
    let user3 = mock_api.addr_make("user3");
    let user4 = mock_api.addr_make("user4");

    (app, code_id, vec![owner, user1, user2, user3, user4])
}

#[allow(dead_code)]
pub fn instantiate_contract(app: &mut ElixirTestApp, code_id: u64, owner: Addr) -> Addr {
    app.instantiate_contract(
        code_id,
        owner.clone(),
        &InstantiateMsg {
            owner: owner.to_string(),
        },
        &[],
        "Contract",
        None,
    )
    .unwrap()
}

#[allow(dead_code)]
pub fn instantiate_contract_with_trader_and_vault(
    app: &mut ElixirTestApp,
    code_id: u64,
    owner: Addr,
    trader: Addr,
) -> Addr {
    let app_addr = app
        .instantiate_contract(
            code_id,
            owner.clone(),
            &InstantiateMsg {
                owner: owner.to_string(),
            },
            &[],
            "Contract",
            None,
        )
        .unwrap();

    let _ = app
        .execute_contract(
            owner.clone(),
            app_addr.clone(),
            &ExecuteMsg::SetTrader {
                new_trader: trader.to_string(),
            },
            &[],
        )
        .unwrap();

    let _ = app
        .execute_contract(
            trader.clone(),
            app_addr.clone(),
            &ExecuteMsg::CreateVault { perp_id: 0 },
            &[],
        )
        .unwrap();

    app_addr
}

#[allow(dead_code)]
pub fn mint_native(app: &mut ElixirTestApp, beneficiary: String, denom: String, amount: u128) {
    app.sudo(cw_multi_test::SudoMsg::Bank(
        cw_multi_test::BankSudo::Mint {
            to_address: beneficiary,
            amount: vec![coin(amount, denom)],
        },
    ))
    .unwrap();
}

#[allow(dead_code)]
pub fn fetch_attributes(resp: &AppResponse, key: String) -> Vec<Attribute> {
    let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    wasm.attributes
        .iter()
        .cloned()
        .filter(|attr| attr.key == key)
        .collect()
}

#[allow(dead_code)]
pub fn fetch_response_events(resp: &AppResponse, event_name: String) -> Vec<Event> {
    resp.events
        .iter()
        .cloned()
        .filter(|ev| ev.ty == format!("wasm-{event_name}"))
        .collect()
}

pub struct TestDydx {
    pub bank: BankKeeper,
    /// Mock meant to mimic subaccount state on dYdX chain
    mock_subaccounts: RefCell<HashMap<u32, Subaccount>>,
    /// Mock of orders for a subaccount. Keyed on subaccount number, value is client order id
    mock_orders: RefCell<HashMap<u32, Vec<u32>>>,
}

impl TestDydx {
    pub fn new() -> Self {
        TestDydx {
            bank: BankKeeper::new(),
            mock_orders: RefCell::new(HashMap::new()),
            mock_subaccounts: RefCell::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    pub fn has_order(&self, subaccount_number: u32, client_order_id: u32) -> bool {
        let order_map = self.mock_orders.borrow();
        if let Some(orders) = order_map.get(&subaccount_number) {
            orders.contains(&client_order_id)
        } else {
            false
        }
    }

    /// Adds the input perp position to the contract owned subaccount
    #[allow(dead_code)]
    pub fn sudo_add_perp_position(&self, subaccount_number: u32, position: PerpetualPosition) {
        let mut accounts = self.mock_subaccounts.borrow_mut();
        let mut subaccount = accounts.get_mut(&subaccount_number).unwrap().clone();
        subaccount.perpetual_positions.push(position);

        accounts.insert(subaccount_number, subaccount);
    }
}

impl Module for TestDydx {
    type ExecT = DydxMsg;
    type QueryT = DydxQueryWrapper;
    type SudoT = SudoMsg;

    /// Runs any [ExecT](Self::ExecT) message, always returns a default response.
    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _sender: Addr,
        msg: Self::ExecT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg {
            DydxMsg::DepositToSubaccountV1 {
                recipient,
                asset_id,
                quantums,
            } => {
                println!("DepositToSubaccount");
                if recipient.number != 0 || recipient.owner != TEST_CONTRACT_ADDR.to_string() {
                    bail!("tryingto deposit for an unsupported subaccount");
                }

                if asset_id != 0 {
                    bail!("tryingto deposit something other than USDC");
                }

                let mut account_map = self.mock_subaccounts.borrow_mut();

                let subaccount = account_map.get_mut(&0).unwrap();

                if subaccount.asset_positions.len() == 0 {
                    subaccount.asset_positions.push(AssetPosition {
                        asset_id,
                        quantums: SerializableInt::new(quantums.into()),
                        index: 0,
                    })
                } else if subaccount.asset_positions.len() == 1 {
                    let current_amount = subaccount.asset_positions[0].quantums.clone();
                    let new_amount = SerializableInt::new(
                        current_amount.i.checked_add(&quantums.into()).unwrap(),
                    );
                    subaccount.asset_positions[0].quantums = new_amount;
                } else {
                    bail!("subaccount should only have USDC asset");
                }
                Ok(AppResponse::default())
            }
            DydxMsg::WithdrawFromSubaccountV1 {
                subaccount_number,
                recipient,
                asset_id,
                quantums,
            } => {
                println!("WithdrawFromSubaccountV1");
                if subaccount_number != 0 {
                    bail!("tryingto withdraw from an unsupported subaccount");
                }
                if recipient == TEST_CONTRACT_ADDR.to_string() {
                    bail!("tryingto withdraw to the smart contract");
                }
                if asset_id != 0 {
                    bail!("tryingto withdraw something other than USDC");
                }

                let mut account_map = self.mock_subaccounts.borrow_mut();

                let subaccount = account_map.get_mut(&0).unwrap();

                if subaccount.asset_positions.len() == 0 {
                    bail!("tryingto withdraw without any deposits");
                } else if subaccount.asset_positions.len() == 1 {
                    let current_amount = subaccount.asset_positions[0].quantums.clone();
                    let new_amount = SerializableInt::new(
                        current_amount.i.checked_sub(&quantums.into()).unwrap(),
                    );
                    subaccount.asset_positions[0].quantums = new_amount;
                } else {
                    bail!("subaccount should only have USDC asset");
                }
                Ok(AppResponse::default())
            }
            DydxMsg::PlaceOrderV1 {
                subaccount_number,
                client_id,
                order_flags: _,
                clob_pair_id: _,
                side: _,
                quantums: _,
                subticks: _,
                good_til_block_time: _,
                time_in_force: _,
                reduce_only: _,
                client_metadata: _,
                condition_type: _,
                conditional_order_trigger_subticks: _,
            } => {
                println!("PlaceOrderV1");

                // Note that this mock is very simplistic and only handles orders for subaccounts owned by the contract.
                let mut order_map = self.mock_orders.borrow_mut();
                let order_client_ids = order_map.entry(subaccount_number).or_insert(vec![]);

                if !order_client_ids.contains(&client_id) {
                    order_client_ids.push(client_id);
                }

                Ok(AppResponse::default())
            }
            DydxMsg::CancelOrderV1 {
                subaccount_number,
                client_id,
                order_flags: _,
                clob_pair_id: _,
                good_til_block_time: _,
            } => {
                println!("CancelOrderV1");

                let mut order_map = self.mock_orders.borrow_mut();
                let order_client_ids = order_map.entry(subaccount_number).or_insert(vec![]);
                order_client_ids.retain(|&x| x != client_id);

                Ok(AppResponse::default())
            }
            _ => panic!("unknown message"),
        }
    }

    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        request: Self::QueryT,
    ) -> AnyResult<Binary> {
        match request.query_data {
            DydxQuery::MarketPrice { id } => {
                println!("{:?}", "MarketPrice");
                if id != 0 {
                    bail!("only market with id: 0 is supported for testing");
                }
                Ok(to_json_binary(&MarketPrice {
                    id,
                    exponent: -5,
                    price: 6038418054,
                })?)
            }
            DydxQuery::Subaccount { owner, number } => {
                println!("Subaccount {} {}", owner, number);
                if number != 0 || owner != TEST_CONTRACT_ADDR.to_string() {
                    bail!("tryingto query for an unsupported subaccount");
                }

                let subaccount = self.mock_subaccounts.borrow().get(&number).unwrap().clone();
                Ok(to_json_binary(&subaccount)?)
            }
            DydxQuery::PerpetualClobDetails { id } => {
                println!("{:?}", "PerpetualClobDetails");
                if id != 0 {
                    bail!("only market with id: 0 is supported for testing");
                }
                Ok(to_json_binary(&PerpetualClobDetails {
                    perpetual: Perpetual {
                        params: PerpetualParams {
                            id,
                            ticker: "BTC-USD".to_string(),
                            market_id: 0,
                            atomic_resolution: -10,
                            default_funding_ppm: 0,
                            liquidity_tier: 0,
                            market_type: PerpetualMarketType::Cross,
                        },
                        funding_index: SerializableInt::new(BigInt::zero()),
                        open_interest: SerializableInt::new(BigInt::zero()),
                    },
                    clob_pair: ClobPair {
                        id,
                        metadata: Metadata::PerpetualClobMetadata(PerpetualClobMetadata {
                            perpetual_id: 0,
                        }),
                        step_base_quantums: 1000000,
                        subticks_per_tick: 100000,
                        quantum_conversion_exponent: -9,
                        status: Status::Active,
                    },
                })?)
            }
            DydxQuery::LiquidityTiers => {
                unimplemented!()
            }
        }
    }

    fn sudo<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        msg: Self::SudoT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        match msg {
            SudoMsg::Bank(bank_sudo) => self.bank.sudo(api, storage, router, block, bank_sudo),
            SudoMsg::Custom(_) => unimplemented!(),
            SudoMsg::Staking(_) => unimplemented!(),
            SudoMsg::Wasm(_) => unimplemented!(),
        }
    }
}
