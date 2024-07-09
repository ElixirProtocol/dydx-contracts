mod utils;

#[cfg(test)]
mod tests {
    use cw_multi_test::Executor;
    use elixir_dydx_integration::{
        dydx::msg::{OrderConditionType, OrderSide, OrderTimeInForce},
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    };

    use crate::utils::{
        fetch_response_events, instantiate_contract_with_trader_and_vault, test_setup,
    };

    const CLIENT_ID: u32 = 101;
    const ORDER_FLAGS: u32 = 64;
    const CLOB_PAIR_ID: u32 = 0;
    const SUBACCOUNT_NUMBER: u32 = 0;
    const BLOCK_TIME: u32 = 1720791702;

    #[test]
    fn trader_can_place_order() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::PlaceOrder {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    client_id: CLIENT_ID,
                    order_flags: ORDER_FLAGS,
                    clob_pair_id: CLOB_PAIR_ID,
                    side: OrderSide::Buy,
                    quantums: 1000000,
                    subticks: 100000,
                    good_til_block_time: BLOCK_TIME,
                    time_in_force: OrderTimeInForce::Unspecified,
                    reduce_only: false,
                    client_metadata: 0,
                    conditional_order_trigger_subticks: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID))

        // Event::new("placed_order")
        // .add_attribute("owner", self.order_id.subaccount_id.owner.clone())
        // .add_attribute("subaccount_number", self.order_id.subaccount_id.number.to_string())
        // .add_attribute("side", self.side.to_string())
        // .add_attribute("quantums", self.quantums.to_string())
        // .add_attribute("subticks", self.subticks.to_string())
        // .add_attribute("time_in_force", self.time_in_force.to_string())
        // .add_attribute("reduce_only", self.reduce_only.to_string())
        // .add_attribute("client_metadata", self.client_metadata.to_string())
        // .add_attribute("condition_type", self.condition_type.to_string())
        // .add_attribute("conditional_order_trigger_subticks", self.conditional_order_trigger_subticks.to_string())

        // let trader_added_events = fetch_response_events(&add_response, "trader_added".to_string());
        // assert!(trader_added_events.len() == 1);
        // assert!(trader_added_events[0].ty == "wasm-trader_added");

        // let method_attributes = fetch_attributes(&add_response, "method".to_string());
        // assert!(method_attributes.len() == 1);
        // assert!(method_attributes[0].key == "method");
        // assert!(method_attributes[0].value == "add_traders");

        // let count_attributes = fetch_attributes(&add_response, "added_count".to_string());
        // assert!(count_attributes.len() == 1);
        // assert!(count_attributes[0].key == "added_count");
        // assert!(count_attributes[0].value == "1");
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap cannot place/cancel trades"
    )]
    fn non_traders_cannot_place_order() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let place_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::PlaceOrder {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    client_id: CLIENT_ID,
                    order_flags: ORDER_FLAGS,
                    clob_pair_id: CLOB_PAIR_ID,
                    side: OrderSide::Buy,
                    quantums: 1000000,
                    subticks: 100000,
                    good_til_block_time: BLOCK_TIME,
                    time_in_force: OrderTimeInForce::Unspecified,
                    reduce_only: false,
                    client_metadata: 0,
                    conditional_order_trigger_subticks: 0,
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn trader_can_cancel_order() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::PlaceOrder {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    client_id: CLIENT_ID,
                    order_flags: ORDER_FLAGS,
                    clob_pair_id: CLOB_PAIR_ID,
                    side: OrderSide::Buy,
                    quantums: 1000000,
                    subticks: 100000,
                    good_til_block_time: BLOCK_TIME,
                    time_in_force: OrderTimeInForce::Unspecified,
                    reduce_only: false,
                    client_metadata: 0,
                    conditional_order_trigger_subticks: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));

        let _cancel_response = app
            .execute_contract(
                user1,
                app_addr,
                &ExecuteMsg::CancelOrder {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    client_id: CLIENT_ID,
                    order_flags: ORDER_FLAGS,
                    clob_pair_id: CLOB_PAIR_ID,
                    good_til_block_time: BLOCK_TIME,
                },
                &[],
            )
            .unwrap();

        assert!(!app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap cannot place/cancel trades"
    )]
    fn non_traders_cannot_cancel_orders() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::PlaceOrder {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    client_id: CLIENT_ID,
                    order_flags: ORDER_FLAGS,
                    clob_pair_id: CLOB_PAIR_ID,
                    side: OrderSide::Buy,
                    quantums: 1000000,
                    subticks: 100000,
                    good_til_block_time: BLOCK_TIME,
                    time_in_force: OrderTimeInForce::Unspecified,
                    reduce_only: false,
                    client_metadata: 0,
                    conditional_order_trigger_subticks: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));

        let _cancel_response = app
            .execute_contract(
                user2,
                app_addr,
                &ExecuteMsg::CancelOrder {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    client_id: CLIENT_ID,
                    order_flags: ORDER_FLAGS,
                    clob_pair_id: CLOB_PAIR_ID,
                    good_til_block_time: BLOCK_TIME,
                },
                &[],
            )
            .unwrap();
    }
}
