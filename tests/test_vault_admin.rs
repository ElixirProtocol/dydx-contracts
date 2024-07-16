mod utils;

#[cfg(test)]
mod tests {
    use cw_multi_test::Executor;
    use elixir_dydx_integration::msg::{ExecuteMsg, QueryMsg, TraderResponse, VaultsResponse};

    use crate::utils::{fetch_attributes, fetch_response_events, instantiate_contract, test_setup};

    #[test]
    fn trader_can_create_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let set_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::SetTrader {
                    new_trader: user1.to_string(),
                },
                &[],
            )
            .unwrap();

        let create_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let trader_resp: TraderResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Trader {})
            .unwrap();

        let vault_resp: VaultsResponse = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Vaults {})
            .unwrap();

        assert!(trader_resp.trader == user1);
        assert!(vault_resp.vaults.len() == 1);
        assert!(vault_resp.vaults[0] == 1);

        let trader_events = fetch_response_events(&set_response, "new_trader".to_string());
        assert!(trader_events.len() == 1);
        assert!(trader_events[0].ty == "wasm-new_trader");
        assert!(trader_events[0].attributes.len() == 3);
        assert!(trader_events[0].attributes[1].key == "old");
        assert!(
            trader_events[0].attributes[1].value
                == "cosmwasm1fsgzj6t7udv8zhf6zj32mkqhcjcpv52yph5qsdcl0qt94jgdckqs2g053y"
        );
        assert!(trader_events[0].attributes[2].key == "new");
        assert!(
            trader_events[0].attributes[2].value
                == "cosmwasm1pgzph9rze2j2xxavx4n7pdhxlkgsq7rak245x0vk7mgh3j4le6gqmlwcfu"
        );

        let method_attributes = fetch_attributes(&create_vault_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "create_vault");

        let vault_events = fetch_response_events(&create_vault_response, "new_vault".to_string());
        assert!(vault_events.len() == 1);
        assert!(vault_events[0].ty == "wasm-new_vault");
        assert!(vault_events[0].attributes.len() == 4);
        assert!(vault_events[0].attributes[1].key == "perp_id");
        assert!(vault_events[0].attributes[1].value == "1");
        assert!(vault_events[0].attributes[2].key == "lp_name");
        assert!(vault_events[0].attributes[2].value == "Elixir LP Token: dYdX-1");
        assert!(vault_events[0].attributes[3].key == "lp_symbol");
        assert!(vault_events[0].attributes[3].value == "ELXR-LP-dYdX-1");
    }

    #[test]
    fn trader_can_create_multiple_vaults() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let _set_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::SetTrader {
                    new_trader: user1.to_string(),
                },
                &[],
            )
            .unwrap();

        let _cv1 = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let _cv2 = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault { perp_id: 2 },
                &[],
            )
            .unwrap();

        let vault_resp: VaultsResponse = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Vaults {})
            .unwrap();

        assert!(vault_resp.vaults.len() == 2);
        assert!(vault_resp.vaults[0] == 1);
        assert!(vault_resp.vaults[1] == 2);
    }

    #[test]
    #[should_panic(expected = "Vault already initialized for perp_id: 1")]
    fn trader_cannot_create_multiple_vaults_with_same_market_id() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let same_market_id = 1;

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let _set_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::SetTrader {
                    new_trader: user1.to_string(),
                },
                &[],
            )
            .unwrap();

        let _cv1 = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault {
                    perp_id: same_market_id,
                },
                &[],
            )
            .unwrap();

        let _cv2 = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault {
                    perp_id: same_market_id,
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1pgzph9rze2j2xxavx4n7pdhxlkgsq7rak245x0vk7mgh3j4le6gqmlwcfu is not the trader"
    )]
    fn need_permissions_to_create_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let same_market_id = 1;

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let _cv1 = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault {
                    perp_id: same_market_id,
                },
                &[],
            )
            .unwrap();
    }
}
