mod utils;

#[cfg(test)]
mod tests {
    use crate::utils::{fetch_attributes, fetch_response_events, instantiate_contract, test_setup};
    use cw_multi_test::Executor;
    use elixir_dydx_integration::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TraderResponse};

    #[test]
    fn can_instantiate_contract() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let resp: TraderResponse = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Trader {})
            .unwrap();
        assert_eq!(resp, TraderResponse { trader: owner });
    }

    #[test]
    #[should_panic(
        expected = "Provided owner: cosmwasm1pgzph9rze2j2xxavx4n7pdhxlkgsq7rak245x0vk7mgh3j4le6gqmlwcfu does not match the sender"
    )]
    fn sender_must_be_owner_on_instantiated_contract() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let _app_addr = app
            .instantiate_contract(
                code_id,
                owner.clone(),
                &InstantiateMsg {
                    owner: user1.to_string(),
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();
    }

    #[test]
    fn owner_can_set_trader() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::SetTrader {
                    new_trader: user1.to_string(),
                },
                &[],
            )
            .unwrap();

        let traders_resp: TraderResponse = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Trader {})
            .unwrap();

        assert!(traders_resp.trader == user1);

        let trader_added_events = fetch_response_events(&add_response, "trader_set".to_string());
        assert!(trader_added_events.len() == 1);
        println!("{:?}", trader_added_events);
        assert!(trader_added_events[0].ty == "wasm-trader_set");

        let method_attributes = fetch_attributes(&add_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "set_trader");

        assert!(trader_added_events[0].attributes.len() == 3);
        assert!(trader_added_events[0].attributes[1].key == "old");
        assert!(trader_added_events[0].attributes[1].value == owner.to_string());
        assert!(trader_added_events[0].attributes[2].key == "new");
        assert!(trader_added_events[0].attributes[2].value == user1.to_string());
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1pgzph9rze2j2xxavx4n7pdhxlkgsq7rak245x0vk7mgh3j4le6gqmlwcfu does not have permission to modify trader"
    )]
    fn only_owner_can_set_trader_initially() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let _add_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::SetTrader {
                    new_trader: user1.to_string(),
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn set_trader_is_idempotent() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::SetTrader {
                    new_trader: user1.to_string(),
                },
                &[],
            )
            .unwrap();

        let traders_resp: TraderResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Trader {})
            .unwrap();

        assert!(traders_resp.trader == user1.to_string());

        let add_response2 = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::SetTrader {
                    new_trader: user1.to_string(),
                },
                &[],
            )
            .unwrap();

        let traders_resp2: TraderResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Trader {})
            .unwrap();

        assert!(traders_resp2.trader == user1.to_string());

        let trader_added_events = fetch_response_events(&add_response2, "trader_set".to_string());
        assert!(trader_added_events.len() == 1);

        let method_attributes = fetch_attributes(&add_response2, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "set_trader");
    }
}
