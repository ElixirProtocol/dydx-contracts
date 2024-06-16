mod utils;

#[cfg(test)]
mod tests {
    use cw_multi_test::Executor;
    use elixir_dydx_integration::{
        msg::{ExecuteMsg, InstantiateMsg, QueryMsg, TradersResp},
        state::Trader,
    };

    use crate::utils::{fetch_attributes, fetch_response_events, instantiate_contract, test_setup};

    #[test]
    fn can_instantiate_contract() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Traders {})
            .unwrap();
        assert_eq!(
            resp,
            TradersResp {
                traders: vec![(owner, Trader { num_markets: 0 })]
            }
        );
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
    fn owner_can_add_traders() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 2);
        assert!(traders_resp.traders.contains(&(owner, Trader::default())));
        assert!(traders_resp.traders.contains(&(user1, Trader::default())));

        let trader_added_events = fetch_response_events(&add_response, "trader_added".to_string());
        assert!(trader_added_events.len() == 1);
        assert!(trader_added_events[0].ty == "wasm-trader_added");

        let method_attributes = fetch_attributes(&add_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "add_traders");

        let count_attributes = fetch_attributes(&add_response, "added_count".to_string());
        assert!(count_attributes.len() == 1);
        assert!(count_attributes[0].key == "added_count");
        assert!(count_attributes[0].value == "1");
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1pgzph9rze2j2xxavx4n7pdhxlkgsq7rak245x0vk7mgh3j4le6gqmlwcfu does not have permission to modify traders"
    )]
    fn only_owner_can_add_traders() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn can_add_multiple_admins() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string(), user2.to_string()];
        let add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 3);
        assert!(traders_resp.traders.contains(&(owner, Trader::default())));
        assert!(traders_resp.traders.contains(&(user1, Trader::default())));
        assert!(traders_resp.traders.contains(&(user2, Trader::default())));

        let trader_added_events = fetch_response_events(&add_response, "trader_added".to_string());
        assert!(trader_added_events.len() == 2);

        let method_attributes = fetch_attributes(&add_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "add_traders");

        let count_attributes = fetch_attributes(&add_response, "added_count".to_string());
        assert!(count_attributes.len() == 1);
        assert!(count_attributes[0].key == "added_count");
        assert!(count_attributes[0].value == "2");
    }

    #[test]
    fn add_traders_is_idempotent() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders {
                    new_traders: new_traders.clone(),
                },
                &[],
            )
            .unwrap();

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 2);
        assert!(traders_resp
            .traders
            .contains(&(owner.clone(), Trader::default())));
        assert!(traders_resp
            .traders
            .contains(&(user1.clone(), Trader::default())));

        let add_response2 = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let traders_resp2: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp2.traders.len() == 2);
        assert!(traders_resp.traders.contains(&(owner, Trader::default())));
        assert!(traders_resp.traders.contains(&(user1, Trader::default())));

        let trader_added_events = fetch_response_events(&add_response2, "trader_added".to_string());
        assert!(trader_added_events.len() == 0);

        let count_attributes = fetch_attributes(&add_response2, "added_count".to_string());
        assert!(count_attributes.len() == 1);
        assert!(count_attributes[0].key == "added_count");
        assert!(count_attributes[0].value == "0");
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap does not have permission to modify traders"
    )]
    fn only_owner_can_remove_traders() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let traders_to_remove = new_traders.clone();
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let _remove_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::RemoveTraders { traders_to_remove },
                &[],
            )
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Trade permissions cannot be revoked from the contract deployer")]
    fn cannot_remove_owner_as_admin() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());
        let traders_to_remove = vec![owner.to_string()];

        let _remove_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::RemoveTraders { traders_to_remove },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn can_remove_multiple_admins() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string(), user2.to_string()];
        let traders_to_remove = new_traders.clone();
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 3);
        assert!(traders_resp
            .traders
            .contains(&(owner.clone(), Trader::default())));
        assert!(traders_resp
            .traders
            .contains(&(user1.clone(), Trader::default())));
        assert!(traders_resp
            .traders
            .contains(&(user2.clone(), Trader::default())));

        let remove_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::RemoveTraders { traders_to_remove },
                &[],
            )
            .unwrap();

        let trader_removed_events =
            fetch_response_events(&remove_response, "trader_removed".to_string());
        assert!(trader_removed_events.len() == 2);

        let method_attributes = fetch_attributes(&remove_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "remove_traders");

        let count_attributes = fetch_attributes(&remove_response, "removed_count".to_string());
        assert!(count_attributes.len() == 1);
        assert!(count_attributes[0].key == "removed_count");
        assert!(count_attributes[0].value == "2");

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 1);
        assert!(traders_resp
            .traders
            .contains(&(owner.clone(), Trader::default())));
    }

    #[test]
    fn remove_traders_is_idempotent() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let traders_to_remove = new_traders.clone();
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders {
                    new_traders: new_traders.clone(),
                },
                &[],
            )
            .unwrap();

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 2);
        assert!(traders_resp
            .traders
            .contains(&(owner.clone(), Trader::default())));
        assert!(traders_resp
            .traders
            .contains(&(user1.clone(), Trader::default())));

        let _remove_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::RemoveTraders {
                    traders_to_remove: traders_to_remove.clone(),
                },
                &[],
            )
            .unwrap();

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 1);
        assert!(traders_resp
            .traders
            .contains(&(owner.clone(), Trader::default())));

        let remove_response2 = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::RemoveTraders { traders_to_remove },
                &[],
            )
            .unwrap();

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 1);
        assert!(traders_resp
            .traders
            .contains(&(owner.clone(), Trader::default())));

        let trader_added_events =
            fetch_response_events(&remove_response2, "trader_removed".to_string());
        assert!(trader_added_events.len() == 0);

        let count_attributes = fetch_attributes(&remove_response2, "removed_count".to_string());
        assert!(count_attributes.len() == 1);
        assert!(count_attributes[0].key == "removed_count");
        assert!(count_attributes[0].value == "0");
    }
}
