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

        // let trader_added_events = fetch_response_events(&create_vault_response, "trader_added".to_string());
        // assert!(trader_added_events.len() == 1);
        // assert!(trader_added_events[0].ty == "wasm-trader_added");

        let method_attributes = fetch_attributes(&create_vault_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "create_vault");

        // let count_attributes = fetch_attributes(&add_response, "added_count".to_string());
        // assert!(count_attributes.len() == 1);
        // assert!(count_attributes[0].key == "added_count");
        // assert!(count_attributes[0].value == "1");

        // TODO: events
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
