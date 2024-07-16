mod utils;

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Coin, Uint128};
    use cw_multi_test::Executor;
    use elixir_dydx_integration::{
        dydx::msg::{OrderSide, OrderTimeInForce},
        error::ContractError,
        execute::{market_make::NewOrder, USDC_COIN_TYPE},
        msg::ExecuteMsg,
    };

    use crate::utils::{
        fetch_response_events, instantiate_contract_with_trader_and_vault, mint_native, test_setup,
    };

    const CLIENT_ID: u32 = 101;
    const CLOB_PAIR_ID: u32 = 0;
    const SUBACCOUNT_NUMBER: u32 = 0;
    const BLOCK_TIME: u32 = 1720791702;

    fn new_order() -> NewOrder {
        NewOrder {
            client_id: CLIENT_ID,
            side: OrderSide::Buy,
            quantums: 1000000,
            subticks: 100000,
            good_til_block_time: BLOCK_TIME,
            time_in_force: OrderTimeInForce::Unspecified,
            reduce_only: false,
            client_metadata: 0,
            conditional_order_trigger_subticks: 0,
        }
    }

    #[test]
    fn trader_can_place_order() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let deposit_amount = 10_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![new_order()],
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID))
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap is not the trader"
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

        let _place_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![new_order()],
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
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
        let deposit_amount = 10_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![new_order()],
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));

        let _cancel_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![],
                    cancel_client_ids: vec![CLIENT_ID],
                    cancel_good_til_block: BLOCK_TIME,
                },
                &[],
            )
            .unwrap();

        assert!(!app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap is not the trader"
    )]
    fn non_traders_cannot_cancel_orders() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 10_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![new_order()],
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));

        let _cancel_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![],
                    cancel_client_ids: vec![CLIENT_ID],
                    cancel_good_til_block: BLOCK_TIME,
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn trader_can_place_multiple_orders_at_once() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let deposit_amount = 10_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let mut new_orders = vec![new_order(), new_order(), new_order()];
        new_orders[1].client_id += 1;
        new_orders[2].client_id += 2;

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders,
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));
        assert!(app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 1));
        assert!(app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 2));
    }

    #[test]
    #[should_panic(expected = "Trader can only place at most 3 bids and 3 asks at a time")]
    fn trader_can_only_place_3_orders_per_side() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let deposit_amount = 10_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let mut new_orders = vec![new_order(), new_order(), new_order(), new_order()];
        new_orders[1].client_id += 1;
        new_orders[2].client_id += 2;
        new_orders[3].client_id += 3;

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders,
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn trader_can_cancel_multiple_orders_at_once() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let deposit_amount = 10_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let mut new_orders = vec![new_order(), new_order(), new_order()];
        new_orders[1].client_id += 1;
        new_orders[2].client_id += 2;

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders,
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));
        assert!(app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 1));
        assert!(app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 2));

        let _cancel_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![],
                    cancel_client_ids: vec![CLIENT_ID, CLIENT_ID + 1],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(!app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));
        assert!(!app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 1));
        assert!(app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 2));
    }

    #[test]
    fn placing_orders_fails_if_it_would_increase_leverage_over_1x() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let deposit_amount = 1_999_999;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount + 1,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let place_response = app.execute_contract(
            user1.clone(),
            app_addr.clone(),
            &ExecuteMsg::MarketMake {
                subaccount_number: SUBACCOUNT_NUMBER,
                clob_pair_id: CLOB_PAIR_ID,
                new_orders: vec![new_order()],
                cancel_client_ids: vec![],
                cancel_good_til_block: 0,
            },
            &[],
        );

        // fails because order is worth $1 and we only have $1.999999... in deposits -> leverage increases over 1x
        assert!(place_response.is_err());
        if let Some(error) = place_response.unwrap_err().downcast_ref::<ContractError>() {
            assert_eq!(
                error,
                &ContractError::NewOrderWouldIncreaseLeverageTooMuch {
                    perp_id: 0,
                    new_order: new_order()
                }
            );
        } else {
            panic!("Expected ContractError::NewOrderWouldIncreaseLeverageTooMuch");
        }

        // deposit 1E-6 more USDC and the order can be placed (at 1x leverage)
        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(1),
                }],
            )
            .unwrap();

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![new_order()],
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();
        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));
    }

    #[test]
    fn trader_can_place_and_cancel_in_one_msg() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let deposit_amount = 10_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user1.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let mut orders1 = vec![new_order(), new_order(), new_order(), new_order()];
        let mut orders2 = vec![new_order()];
        orders1[1].client_id += 1;
        orders1[2].client_id += 2;
        orders2[0].client_id += 3;

        let _place_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: vec![new_order()],
                    cancel_client_ids: vec![],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));

        let place2_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::MarketMake {
                    subaccount_number: SUBACCOUNT_NUMBER,
                    clob_pair_id: CLOB_PAIR_ID,
                    new_orders: orders2,
                    cancel_client_ids: vec![CLIENT_ID, CLIENT_ID + 1, CLIENT_ID + 2],
                    cancel_good_til_block: 0,
                },
                &[],
            )
            .unwrap();

        assert!(!app.router().custom.has_order(SUBACCOUNT_NUMBER, CLIENT_ID));
        assert!(!app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 1));
        assert!(!app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 2));
        assert!(app
            .router()
            .custom
            .has_order(SUBACCOUNT_NUMBER, CLIENT_ID + 3));

        let place_events = fetch_response_events(&place2_response, "placed_order".to_string());
        assert!(place_events.len() == 1);
        assert!(place_events[0].ty == "wasm-placed_order");
        assert!(place_events[0].attributes.len() == 7);
        assert!(place_events[0].attributes[1].key == "perp_id");
        assert!(place_events[0].attributes[1].value == "0");
        assert!(place_events[0].attributes[2].key == "client_id");
        assert!(place_events[0].attributes[2].value == "104");
        assert!(place_events[0].attributes[3].key == "clob_pair_id");
        assert!(place_events[0].attributes[3].value == "0");
        assert!(place_events[0].attributes[4].key == "side");
        assert!(place_events[0].attributes[4].value == "Buy");
        assert!(place_events[0].attributes[5].key == "quantums");
        assert!(place_events[0].attributes[5].value == "1000000");
        assert!(place_events[0].attributes[6].key == "subticks");
        assert!(place_events[0].attributes[6].value == "100000");

        let cancelled_events =
            fetch_response_events(&place2_response, "cancelled_order".to_string());
        assert!(cancelled_events.len() == 3);
        assert!(cancelled_events[0].attributes.len() == 5);
        assert!(cancelled_events[0].attributes[1].key == "perp_id");
        assert!(cancelled_events[0].attributes[1].value == "0");
        assert!(cancelled_events[0].attributes[2].key == "client_id");
        assert!(cancelled_events[0].attributes[2].value == "101");
        assert!(cancelled_events[0].attributes[3].key == "clob_pair_id");
        assert!(cancelled_events[0].attributes[3].value == "0");
        assert!(cancelled_events[0].attributes[4].key == "cancel_good_til_block");
        assert!(cancelled_events[0].attributes[4].value == "0");

        assert!(cancelled_events[1].attributes[1].key == "perp_id");
        assert!(cancelled_events[1].attributes[1].value == "0");
        assert!(cancelled_events[1].attributes[2].key == "client_id");
        assert!(cancelled_events[1].attributes[2].value == "102");
        assert!(cancelled_events[1].attributes[3].key == "clob_pair_id");
        assert!(cancelled_events[1].attributes[3].value == "0");
        assert!(cancelled_events[1].attributes[4].key == "cancel_good_til_block");
        assert!(cancelled_events[1].attributes[4].value == "0");

        assert!(cancelled_events[2].attributes[1].key == "perp_id");
        assert!(cancelled_events[2].attributes[1].value == "0");
        assert!(cancelled_events[2].attributes[2].key == "client_id");
        assert!(cancelled_events[2].attributes[2].value == "103");
        assert!(cancelled_events[2].attributes[3].key == "clob_pair_id");
        assert!(cancelled_events[2].attributes[3].value == "0");
        assert!(cancelled_events[2].attributes[4].key == "cancel_good_til_block");
        assert!(cancelled_events[2].attributes[4].value == "0");
    }
}
