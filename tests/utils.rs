use std::fmt::Debug;
use std::marker::PhantomData;

use cosmwasm_std::{
    testing::{mock_env, MockApi, MockStorage}, Addr, Api, Attribute, Binary, BlockInfo, CustomMsg, CustomQuery, Empty, Event, Querier, Record, Storage, WasmMsg, WasmQuery
};
use cw_multi_test::{error::{bail, AnyResult}, App, AppBuilder, AppResponse, BankKeeper, Contract, ContractData, ContractWrapper, CosmosRouter, DistributionKeeper, Executor, GovFailingModule, IbcFailingModule, Module, StakeKeeper, StargateFailingModule, Wasm, WasmKeeper, WasmSudo
};
use elixir_dydx_integration::{
    dydx::{
        msg::DydxMsg,
        query::DydxQueryWrapper,
    },
    msg::InstantiateMsg,
};
use serde::de::DeserializeOwned;

pub type ElixirTestApp = App<BankKeeper, MockApi, MockStorage, TestDydx, WasmKeeper<DydxMsg, DydxQueryWrapper>, StakeKeeper, DistributionKeeper, IbcFailingModule, GovFailingModule, StargateFailingModule>;

pub fn test_setup() -> (ElixirTestApp, u64, Vec<Addr>) {
    let contract = ContractWrapper::new(
        elixir_dydx_integration::contract::execute,
        elixir_dydx_integration::contract::instantiate,
        elixir_dydx_integration::contract::query,
    );
    let b: Box<dyn Contract<DydxMsg, DydxQueryWrapper>> = Box::new(contract);

    let test_dydx = TestDydx {};
    // let wasm_keeper = DydxWasmKeeper::new(EXECUTE_MSG, QUERY_MSG, SUDO_MSG);
    let app_builder = AppBuilder::new_custom();

    let mut app = app_builder
        .with_custom(test_dydx)
        .build(|_, _, _| {});
    let code_id = app.store_code(b);

    let mock_api = MockApi::default();
    let owner = mock_api.addr_make("owner");
    let user1 = mock_api.addr_make("user1");
    let user2 = mock_api.addr_make("user2");

    (app, code_id, vec![owner, user1, user2])
}

pub fn instantiate_contract(
    app: &mut ElixirTestApp,
    code_id: u64,
    owner: Addr,
) -> Addr {
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

pub fn fetch_attributes(resp: &AppResponse, key: String) -> Vec<Attribute> {
    let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
    wasm.attributes
        .iter()
        .cloned()
        .filter(|attr| attr.key == key)
        .collect()
}

pub fn fetch_response_events(resp: &AppResponse, event_name: String) -> Vec<Event> {
    resp.events
        .iter()
        .cloned()
        .filter(|ev| ev.ty == format!("wasm-{event_name}"))
        .collect()
}

pub struct TestDydx {}

impl Module for TestDydx
{
    type ExecT = DydxMsg;
    type QueryT = DydxQueryWrapper;
    type SudoT = Empty;

    /// Runs any [ExecT](Self::ExecT) message, always returns a default response.
    fn execute<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _sender: Addr,
        _msg: Self::ExecT,
    ) -> AnyResult<AppResponse>        where
    ExecC: CustomMsg + DeserializeOwned + 'static,
    QueryC: CustomQuery + DeserializeOwned + 'static, {
        Ok(AppResponse::default())
    }

    fn query(
        &self,
        _api: &dyn Api,
        _storage: &dyn Storage,
        _querier: &dyn Querier,
        _block: &BlockInfo,
        _request: Self::QueryT,
    ) -> AnyResult<Binary> {
        bail!("query not implemented for CustomHandler")
    }

    fn sudo<ExecC, QueryC>(
        &self,
        _api: &dyn Api,
        _storage: &mut dyn Storage,
        _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        _block: &BlockInfo,
        _msg: Self::SudoT,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        bail!("sudo not implemented for CustomHandler")
    }
}

// const CODE_ID: u64 = 154;
// const EXECUTE_MSG: &str = "wasm execute called";
// const QUERY_MSG: &str = "wasm query called";
// const SUDO_MSG: &str = "wasm sudo called";
// const DUPLICATE_CODE_MSG: &str = "wasm duplicate code called";
// const CONTRACT_DATA_MSG: &str = "wasm contract data called";
// static WASM_RAW: Lazy<Vec<Record>> = Lazy::new(|| vec![(vec![154u8], vec![155u8])]);

// type DydxWasmKeeper = DydxKeeper<DydxMsg, DydxQueryWrapper, Empty>;

// impl<ExecT, QueryT> Wasm<ExecT, QueryT> for DydxWasmKeeper {
//     fn execute(
//         &self,
//         _api: &dyn Api,
//         _storage: &mut dyn Storage,
//         _router: &dyn CosmosRouter<ExecC = ExecT, QueryC = QueryT>,
//         _block: &BlockInfo,
//         _sender: Addr,
//         _msg: WasmMsg,
//     ) -> AnyResult<AppResponse> {
//         bail!(self.1);
//     }

//     fn query(
//         &self,
//         _api: &dyn Api,
//         _storage: &dyn Storage,
//         _querier: &dyn Querier,
//         _block: &BlockInfo,
//         _request: WasmQuery,
//     ) -> AnyResult<Binary> {
//         bail!(self.2);
//     }

//     fn sudo(
//         &self,
//         _api: &dyn Api,
//         _storage: &mut dyn Storage,
//         _router: &dyn CosmosRouter<ExecC = ExecT, QueryC = QueryT>,
//         _block: &BlockInfo,
//         _msg: WasmSudo,
//     ) -> AnyResult<AppResponse> {
//         bail!(self.3);
//     }

//     fn store_code(&mut self, _creator: Addr, _code: Box<dyn Contract<ExecT, QueryT>>) -> u64 {
//         CODE_ID
//     }

//     fn store_code_with_id(
//         &mut self,
//         _creator: Addr,
//         code_id: u64,
//         _code: Box<dyn Contract<ExecT, QueryT>>,
//     ) -> AnyResult<u64> {
//         Ok(code_id)
//     }

//     fn duplicate_code(&mut self, _code_id: u64) -> AnyResult<u64> {
//         bail!(DUPLICATE_CODE_MSG);
//     }

//     fn contract_data(&self, _storage: &dyn Storage, _address: &Addr) -> AnyResult<ContractData> {
//         bail!(CONTRACT_DATA_MSG);
//     }

//     fn dump_wasm_raw(&self, _storage: &dyn Storage, _address: &Addr) -> Vec<Record> {
//         WASM_RAW.clone()
//     }
// }



// pub struct DydxKeeper<ExecT, QueryT, SudoT>(
//     PhantomData<(ExecT, QueryT, SudoT)>,
//     &'static str,
//     &'static str,
//     &'static str,
// );

// impl<ExecT, QueryT, SudoT> DydxKeeper<ExecT, QueryT, SudoT> {
//     fn new(execute_msg: &'static str, query_msg: &'static str, sudo_msg: &'static str) -> Self {
//         Self(Default::default(), execute_msg, query_msg, sudo_msg)
//     }
// }

// impl Module for DydxWasmKeeper
// {
//     type ExecT = DydxMsg;
//     type QueryT = DydxQueryWrapper;
//     type SudoT = Empty;

//     fn execute<ExecC, QueryC>(
//         &self,
//         _api: &dyn Api,
//         _storage: &mut dyn Storage,
//         _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
//         _block: &BlockInfo,
//         _sender: Addr,
//         msg: Self::ExecT,
//     ) -> AnyResult<AppResponse>
//     where
//         ExecC: CustomMsg + DeserializeOwned + 'static,
//         QueryC: CustomQuery + DeserializeOwned + 'static,
//     {
//         match msg {
//             DydxMsg::DepositToSubaccount { recipient, asset_id, quantums } => todo!(),
//             DydxMsg::WithdrawFromSubaccount { subaccount_number, recipient, asset_id, quantums } => todo!(),
//             DydxMsg::PlaceOrder { subaccount_number, client_id, order_flags, clob_pair_id, side, quantums, subticks, good_til_block_time, time_in_force, reduce_only, client_metadata, condition_type, conditional_order_trigger_subticks } => todo!(),
//             DydxMsg::CancelOrder { subaccount_number, client_id, order_flags, clob_pair_id, good_til_block_time } => todo!(),
//             _ => todo!(),
//         }
//         bail!(self.1);
//     }

//     fn query(
//         &self,
//         _api: &dyn Api,
//         _storage: &dyn Storage,
//         _querier: &dyn Querier,
//         _block: &BlockInfo,
//         _request: Self::QueryT,
//     ) -> AnyResult<Binary> {
//         bail!(self.2);
//     }

//     fn sudo<ExecC, QueryC>(
//         &self,
//         _api: &dyn Api,
//         _storage: &mut dyn Storage,
//         _router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
//         _block: &BlockInfo,
//         _msg: Self::SudoT,
//     ) -> AnyResult<AppResponse>
//     where
//         ExecC: CustomMsg + DeserializeOwned + 'static,
//         QueryC: CustomQuery + DeserializeOwned + 'static,
//     {
//         bail!(self.3);
//     }
// }
