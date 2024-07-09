mod utils;

#[cfg(test)]
mod tests {
    use crate::utils::{
        instantiate_contract_with_trader_and_vault, mint_native, test_setup, TEST_CONTRACT_ADDR,
    };
    use cosmwasm_std::{Coin, Decimal, Uint128};
    use cw_multi_test::Executor;
    use elixir_dydx_integration::{
        execute::USDC_COIN_TYPE,
        msg::{DydxSubaccountResponse, ExecuteMsg, QueryMsg, VaultOwnershipResponse},
    };

    #[test]
    fn can_mint_coin() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        // mint coins to user2
        let _app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(&mut app, user1.to_string(), USDC_COIN_TYPE.to_string(), 10);
    }

    #[test]
    fn any_user_can_deposit_into_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 1_000_000;

        // mint coins to user2
        mint_native(
            &mut app,
            user2.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let _deposit_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let account_resp: DydxSubaccountResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::DydxSubaccount {
                    owner: app_addr.to_string(),
                    number: 0,
                },
            )
            .unwrap();

        let subaccount = account_resp.subaccount;
        let subaccount_id = subaccount.id.unwrap();

        assert!(subaccount_id.number == 0);
        assert!(subaccount_id.owner == TEST_CONTRACT_ADDR.to_string());
        assert!(subaccount.asset_positions.len() == 1);
        assert!(subaccount.asset_positions[0].asset_id == 0);
        assert!(subaccount.asset_positions[0].quantums.i == deposit_amount.into());

        let vault_resp: VaultOwnershipResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::VaultOwnership {
                    perp_id: 0,
                    depositor: user2.to_string(),
                },
            )
            .unwrap();
        assert!(vault_resp.subaccount_owner == TEST_CONTRACT_ADDR.to_string());
        assert!(vault_resp.subaccount_number == 0);
        assert!(vault_resp.asset_usdc_value == Decimal::one());
        assert!(vault_resp.perp_usdc_value == Decimal::zero());
        assert!(vault_resp.depositor_lp_tokens == Uint128::new(deposit_amount));
        assert!(vault_resp.outstanding_lp_tokens == Uint128::new(deposit_amount));
        // TODO: fix
    }

    #[test]
    #[should_panic]
    fn deposit_into_vault_fails_with_no_coins() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 1_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let _deposit_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Tried to deposit an invalid coin: ibc/1234. Only USDC is accepted")]
    fn deposit_into_vault_fails_if_not_usdc() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 1_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        // mint NON-USDC coins to user2
        mint_native(
            &mut app,
            user2.to_string(),
            "ibc/1234".to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: "ibc/1234".to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Cannot transfer empty coins amount")]
    fn deposit_into_vault_fails_if_amount_is_zero() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 1_000_000;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        // mint NON-USDC coins to user2
        mint_native(
            &mut app,
            user2.to_string(),
            "ibc/1234".to_string(),
            deposit_amount,
        );

        let _deposit_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: "ibc/1234".to_string(),
                    amount: Uint128::new(0),
                }],
            )
            .unwrap();
    }

    #[test]
    fn depositors_can_withdraw_from_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 1_000_000;
        let withdraw_amount = 1_000u128;
        let remainder = deposit_amount - withdraw_amount;

        // mint coins to user2
        mint_native(
            &mut app,
            user2.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let _deposit_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let _withdraw_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::WithdrawFromVault {
                    perp_id: 0,
                    amount: withdraw_amount as u64,
                },
                &[],
            )
            .unwrap();

        let account_resp: DydxSubaccountResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::DydxSubaccount {
                    owner: app_addr.to_string(),
                    number: 0,
                },
            )
            .unwrap();

        let subaccount = account_resp.subaccount;
        let subaccount_id = subaccount.id.unwrap();

        assert!(subaccount_id.number == 0);
        assert!(subaccount_id.owner == TEST_CONTRACT_ADDR.to_string());
        assert!(subaccount.asset_positions.len() == 1);
        assert!(subaccount.asset_positions[0].asset_id == 0);
        println!("{:?}", subaccount.asset_positions[0]);
        assert!(subaccount.asset_positions[0].quantums.i == remainder.into());

        let vault_resp: VaultOwnershipResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::VaultOwnership {
                    perp_id: 0,
                    depositor: user2.to_string(),
                },
            )
            .unwrap();
        assert!(vault_resp.subaccount_owner == TEST_CONTRACT_ADDR.to_string());
        assert!(vault_resp.subaccount_number == 0);
        assert!(
            vault_resp.asset_usdc_value == Decimal::from_atomics(Uint128::new(999), 3).unwrap()
        );
        assert!(vault_resp.perp_usdc_value == Decimal::zero());
        assert!(vault_resp.depositor_lp_tokens == Uint128::new(998999));
        assert!(vault_resp.outstanding_lp_tokens == Uint128::new(998999)); // TODO: fix
    }

    #[test]
    fn withdrawing_with_amount_as_0_withdraws_everything() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 1_000_000;
        let withdraw_amount = 0u128;

        // mint coins to user2
        mint_native(
            &mut app,
            user2.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let _deposit_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let _withdraw_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::WithdrawFromVault {
                    perp_id: 0,
                    amount: withdraw_amount as u64,
                },
                &[],
            )
            .unwrap();

        let account_resp: DydxSubaccountResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::DydxSubaccount {
                    owner: app_addr.to_string(),
                    number: 0,
                },
            )
            .unwrap();

        let subaccount = account_resp.subaccount;
        let subaccount_id = subaccount.id.unwrap();

        assert!(subaccount_id.number == 0);
        assert!(subaccount_id.owner == TEST_CONTRACT_ADDR.to_string());
        assert!(subaccount.asset_positions.len() == 1);
        assert!(subaccount.asset_positions[0].asset_id == 0);
        assert!(subaccount.asset_positions[0].quantums.i == 0.into());

        let vault_resp: VaultOwnershipResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::VaultOwnership {
                    perp_id: 0,
                    depositor: user1.to_string(),
                },
            )
            .unwrap();
        assert!(vault_resp.subaccount_owner == TEST_CONTRACT_ADDR.to_string());
        assert!(vault_resp.subaccount_number == 0);
        assert!(vault_resp.asset_usdc_value == Decimal::from_atomics(Uint128::zero(), 3).unwrap());
        assert!(vault_resp.perp_usdc_value == Decimal::zero());
        assert!(vault_resp.depositor_lp_tokens == Uint128::zero());
        assert!(vault_resp.outstanding_lp_tokens == Uint128::zero()); // TODO: fix
    }

    #[test]
    #[should_panic(
        expected = "Could not find LP tokens with perp_id: 0 for cosmwasm1tps04uptd0rzy2a94jjjx4s0pcmyenvtv7lwfph730muq82f9n9s2w0guk"
    )]
    fn only_users_with_deposits_can_withdraw_from_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let user3 = users[3].clone();
        let deposit_amount = 1_000_000;
        let withdraw_amount = 1_000u128;

        // mint coins to user2
        mint_native(
            &mut app,
            user2.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
        );

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        let _deposit_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                &[Coin {
                    denom: USDC_COIN_TYPE.to_string(),
                    amount: Uint128::new(deposit_amount),
                }],
            )
            .unwrap();

        let _withdraw_response = app
            .execute_contract(
                user3.clone(),
                app_addr.clone(),
                &ExecuteMsg::WithdrawFromVault {
                    perp_id: 0,
                    amount: withdraw_amount as u64,
                },
                &[],
            )
            .unwrap();
    }
}
