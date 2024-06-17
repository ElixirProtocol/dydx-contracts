use cosmwasm_std::{testing::MockApi, Addr, Attribute, Empty, Event};
use cw_multi_test::{custom_app, AppResponse, BasicApp, Contract, ContractWrapper, Executor};
use elixir_dydx_integration::{dydx::query::DydxQueryWrapper, msg::InstantiateMsg};

pub fn test_setup() -> (BasicApp<Empty, DydxQueryWrapper>, u64, Vec<Addr>) {
    let contract = ContractWrapper::new(
        elixir_dydx_integration::contract::execute,
        elixir_dydx_integration::contract::instantiate,
        elixir_dydx_integration::contract::query,
    );
    let b: Box<dyn Contract<_, DydxQueryWrapper>> = Box::new(contract);
    let mut app = custom_app::<_, DydxQueryWrapper, _>(|_, _, _| {});
    let code_id = app.store_code(b);

    let mock_api = MockApi::default();
    let owner = mock_api.addr_make("owner");
    let user1 = mock_api.addr_make("user1");
    let user2 = mock_api.addr_make("user2");

    (app, code_id, vec![owner, user1, user2])
}

pub fn instantiate_contract(
    app: &mut BasicApp<Empty, DydxQueryWrapper>,
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
