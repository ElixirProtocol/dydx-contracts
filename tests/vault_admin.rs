mod utils;

#[cfg(test)]
mod tests {
    use cw_multi_test::Executor;
    use elixir_dydx_integration::{
        msg::{ExecuteMsg, QueryMsg, TradersResp, VaultStateResp},
        state::VaultStatus,
    };

    use crate::utils::{fetch_attributes, fetch_response_events, instantiate_contract, test_setup};

    #[test]
    fn trader_can_create_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
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

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 2);
        assert!(traders_resp.traders[1].1.markets[0] == 1);

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

        // TODO: assert callthrough to dYdX
        // TODO: events
    }

    #[test]
    fn trader_can_create_multiple_vaults() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
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

        let traders_resp: TradersResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Traders {})
            .unwrap();

        assert!(traders_resp.traders.len() == 2);
        assert!(traders_resp.traders[1].1.markets[0] == 1);
        assert!(traders_resp.traders[1].1.markets[1] == 2);
    }

    #[test]
    #[should_panic(expected = "Vault already initialized for perp_id: 1")]
    fn trader_cannot_create_multiple_vaults_with_same_market_id() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let same_market_id = 1;

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
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
        expected = "cosmwasm1pgzph9rze2j2xxavx4n7pdhxlkgsq7rak245x0vk7mgh3j4le6gqmlwcfu does not have permission to create vaults"
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

    #[test]
    fn trader_can_freeze_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let _create_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let freeze_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::FreezeVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let freeze_events =
            fetch_response_events(&freeze_vault_response, "vault_frozen".to_string());
        assert!(freeze_events.len() == 1);
        assert!(freeze_events[0].attributes[1].key == "id");
        assert!(freeze_events[0].attributes[1].value == "1");

        let method_attributes = fetch_attributes(&freeze_vault_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "freeze_vault");
    }

    #[test]
    #[should_panic(expected = "Vault already frozen for perp_id: 1")]
    fn trader_can_only_freeze_vault_once() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let _create_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let _freeze_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::FreezeVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let _freeze_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::FreezeVault { perp_id: 1 },
                &[],
            )
            .unwrap();
    }

    // TODO: Trader can only freeze/thaw their own vaults

    // TODO: Trader can only freeze/thaw vaults that exist

    #[test]
    fn trader_can_thaw_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let _create_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let _freeze_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::FreezeVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let thaw_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::ThawVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let thawed_events = fetch_response_events(&thaw_vault_response, "vault_thawed".to_string());
        assert!(thawed_events.len() == 1);
        assert!(thawed_events[0].attributes[1].key == "id");
        assert!(thawed_events[0].attributes[1].value == "1");

        let method_attributes = fetch_attributes(&thaw_vault_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "thaw_vault");
    }

    #[test]
    fn trader_can_set_new_vault_trader() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_traders = vec![user1.to_string(), user2.to_string()];
        let _add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddTraders { new_traders },
                &[],
            )
            .unwrap();

        let _create_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::CreateVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let _freeze_vault_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::FreezeVault { perp_id: 1 },
                &[],
            )
            .unwrap();

        let vault_resp: VaultStateResp = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::VaultState { perp_id: 1 })
            .unwrap();

        assert!(vault_resp.status == VaultStatus::Frozen);
        assert!(vault_resp.subaccount_owner == user1.to_string());
        assert!(vault_resp.subaccount_number == 0);

        let set_trader_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::ChangeVaultTrader {
                    perp_id: 1,
                    new_trader: user2.to_string(),
                },
                &[],
            )
            .unwrap();

        let vault_resp: VaultStateResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::VaultState { perp_id: 1 })
            .unwrap();

        assert!(vault_resp.status == VaultStatus::Frozen);
        assert!(vault_resp.subaccount_owner == user2.to_string());
        assert!(vault_resp.subaccount_number == 0);

        // let freeze_events =
        //     fetch_response_events(&set_trader_response, "vault_frozen".to_string());
        // assert!(freeze_events.len() == 1);
        // assert!(freeze_events[0].attributes[1].key == "id");
        // assert!(freeze_events[0].attributes[1].value == "1");

        let method_attributes = fetch_attributes(&set_trader_response, "method".to_string());
        assert!(method_attributes.len() == 1);
        assert!(method_attributes[0].key == "method");
        assert!(method_attributes[0].value == "change_vault_trader");
    }
}
