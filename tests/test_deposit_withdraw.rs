mod utils;

#[cfg(test)]
mod tests {
    use crate::utils::{
        fetch_response_events, instantiate_contract_with_trader_and_vault, mint_native, test_setup,
        TEST_CONTRACT_ADDR,
    };
    use cosmwasm_std::{Coin, Decimal, Uint128};
    use cw_multi_test::Executor;
    use elixir_dydx_integration::{
        dydx::{proto_structs::PerpetualPosition, serializable_int::SerializableInt},
        error::ContractError,
        execute::USDC_COIN_TYPE,
        msg::{
            DydxSubaccountResponse, ExecuteMsg, LpTokenBalanceResponse, QueryMsg,
            VaultOwnershipResponse, WithdrawalsResponse,
        },
    };
    use num_bigint::BigInt;

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

        let deposit_response = app
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

        let deposit_events = fetch_response_events(&deposit_response, "new_deposit".to_string());
        assert!(deposit_events.len() == 1);
        assert!(deposit_events[0].ty == "wasm-new_deposit");
        assert!(deposit_events[0].attributes.len() == 6);
        assert!(deposit_events[0].attributes[1].key == "depositor");
        assert!(
            deposit_events[0].attributes[1].value
                == "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap"
        );
        assert!(deposit_events[0].attributes[2].key == "perp_id");
        assert!(deposit_events[0].attributes[2].value == "0");
        assert!(deposit_events[0].attributes[3].key == "usdc_amount");
        assert!(deposit_events[0].attributes[3].value == "1000000");
        assert!(deposit_events[0].attributes[4].key == "minted_lp_tokens");
        assert!(deposit_events[0].attributes[4].value == "1000000");
        assert!(deposit_events[0].attributes[5].key == "total_lp_tokens");
        assert!(deposit_events[0].attributes[5].value == "1000000");
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
    fn depositors_can_request_withdrawal_from_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
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

        let user_lp_before: LpTokenBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::UserLpTokens {
                    perp_id: 0,
                    user: user2.to_string(),
                },
            )
            .unwrap();
        let contract_lp_before: LpTokenBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::UserLpTokens {
                    perp_id: 0,
                    user: TEST_CONTRACT_ADDR.to_string(),
                },
            )
            .unwrap();
        assert!(user_lp_before.balance == Uint128::new(deposit_amount));
        assert!(contract_lp_before.balance == Uint128::zero());

        let request_withdraw_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::RequestWithdrawal {
                    perp_id: 0,
                    usdc_amount: withdraw_amount as u64,
                },
                &[],
            )
            .unwrap();

        let q_resp: WithdrawalsResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Withdrawals { perp_id: 0 })
            .unwrap();

        let user_lp_after: LpTokenBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::UserLpTokens {
                    perp_id: 0,
                    user: user2.to_string(),
                },
            )
            .unwrap();
        let contract_lp_after: LpTokenBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::UserLpTokens {
                    perp_id: 0,
                    user: TEST_CONTRACT_ADDR.to_string(),
                },
            )
            .unwrap();

        let withdrawal_queue = q_resp.withdrawal_queue;
        assert!(withdrawal_queue.len() == 1);
        assert!(withdrawal_queue[0].recipient_addr == user2);
        assert!(withdrawal_queue[0].lp_tokens == Uint128::new(withdraw_amount));
        assert!(withdrawal_queue[0].usdc_equivalent == Decimal::from_atomics(1u128, 3).unwrap());

        // tokens are moved to smart contract temporarily
        assert!(user_lp_after.balance == Uint128::new(deposit_amount - withdraw_amount));
        assert!(contract_lp_after.balance == Uint128::new(withdraw_amount));

        let withdraw_events = fetch_response_events(
            &request_withdraw_response,
            "new_withdrawal_request".to_string(),
        );
        assert!(withdraw_events.len() == 1);
        assert!(withdraw_events[0].ty == "wasm-new_withdrawal_request");
        assert!(withdraw_events[0].attributes.len() == 5);
        assert!(withdraw_events[0].attributes[1].key == "withdrawer");
        assert!(
            withdraw_events[0].attributes[1].value
                == "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap"
        );
        assert!(withdraw_events[0].attributes[2].key == "perp_id");
        assert!(withdraw_events[0].attributes[2].value == "0");
        assert!(withdraw_events[0].attributes[3].key == "usdc_amount");
        assert!(withdraw_events[0].attributes[3].value == "1000");
        assert!(withdraw_events[0].attributes[4].key == "transferred_lp_tokens");
        assert!(withdraw_events[0].attributes[4].value == "1000");
    }

    #[test]
    fn multiple_depositors_can_request_withdrawal_from_vault() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let user3 = users[3].clone();
        let user4 = users[4].clone();
        let deposit_amount = 1_000_000;
        let withdraw_amount = 1_000u128;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        // mint, deposit and request withdraw for all users
        for user in vec![user2.clone(), user3.clone(), user4.clone()] {
            mint_native(
                &mut app,
                user.to_string(),
                USDC_COIN_TYPE.to_string(),
                deposit_amount,
            );

            let _deposit_response = app
                .execute_contract(
                    user.clone(),
                    app_addr.clone(),
                    &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                    &[Coin {
                        denom: USDC_COIN_TYPE.to_string(),
                        amount: Uint128::new(deposit_amount),
                    }],
                )
                .unwrap();

            let _request_withdraw_response = app
                .execute_contract(
                    user.clone(),
                    app_addr.clone(),
                    &ExecuteMsg::RequestWithdrawal {
                        perp_id: 0,
                        usdc_amount: withdraw_amount as u64,
                    },
                    &[],
                )
                .unwrap();
        }

        let q_resp: WithdrawalsResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Withdrawals { perp_id: 0 })
            .unwrap();

        let withdrawal_queue = q_resp.withdrawal_queue;
        assert!(withdrawal_queue.len() == 3);
        assert!(withdrawal_queue[0].recipient_addr == user2);
        assert!(withdrawal_queue[0].lp_tokens == Uint128::new(1000));
        assert!(
            withdrawal_queue[0].usdc_equivalent == Decimal::new(Uint128::new(001000000333333443))
        );
        assert!(withdrawal_queue[1].recipient_addr == user3);
        assert!(withdrawal_queue[1].lp_tokens == Uint128::new(1000));
        assert!(
            withdrawal_queue[1].usdc_equivalent == Decimal::new(Uint128::new(001000000333333443))
        );
        assert!(withdrawal_queue[2].recipient_addr == user4);
        assert!(withdrawal_queue[2].lp_tokens == Uint128::new(1000));
        assert!(
            withdrawal_queue[2].usdc_equivalent == Decimal::new(Uint128::new(001000000333333443))
        );
    }

    #[test]
    fn users_can_cancel_their_withdrawal_requests() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let user3 = users[3].clone();
        let user4 = users[4].clone();
        let deposit_amount = 1_000_000;
        let withdraw_amount = 1_000u128;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        // mint, deposit and request withdraw for all users
        for user in vec![user2.clone(), user3.clone(), user4.clone()] {
            mint_native(
                &mut app,
                user.to_string(),
                USDC_COIN_TYPE.to_string(),
                deposit_amount,
            );

            let _deposit_response = app
                .execute_contract(
                    user.clone(),
                    app_addr.clone(),
                    &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                    &[Coin {
                        denom: USDC_COIN_TYPE.to_string(),
                        amount: Uint128::new(deposit_amount),
                    }],
                )
                .unwrap();

            let _request_withdraw_response = app
                .execute_contract(
                    user.clone(),
                    app_addr.clone(),
                    &ExecuteMsg::RequestWithdrawal {
                        perp_id: 0,
                        usdc_amount: withdraw_amount as u64,
                    },
                    &[],
                )
                .unwrap();
        }

        let q_resp: WithdrawalsResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Withdrawals { perp_id: 0 })
            .unwrap();

        let withdrawal_queue = q_resp.withdrawal_queue;
        assert!(withdrawal_queue.len() == 3);
        assert!(withdrawal_queue[0].recipient_addr == user2);
        assert!(withdrawal_queue[0].lp_tokens == Uint128::new(withdraw_amount));
        assert!(withdrawal_queue[1].recipient_addr == user3);
        assert!(withdrawal_queue[1].lp_tokens == Uint128::new(withdraw_amount));
        assert!(withdrawal_queue[2].recipient_addr == user4);
        assert!(withdrawal_queue[2].lp_tokens == Uint128::new(withdraw_amount));

        let _cancel_withdraw_response = app
            .execute_contract(
                user3.clone(),
                app_addr.clone(),
                &ExecuteMsg::CancelWithdrawalRequests { perp_id: 0 },
                &[],
            )
            .unwrap();

        let q_resp: WithdrawalsResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Withdrawals { perp_id: 0 })
            .unwrap();

        let user3_lp: LpTokenBalanceResponse = app
            .wrap()
            .query_wasm_smart(
                app_addr.clone(),
                &QueryMsg::UserLpTokens {
                    perp_id: 0,
                    user: user3.to_string(),
                },
            )
            .unwrap();

        let withdrawal_queue = q_resp.withdrawal_queue;
        assert!(withdrawal_queue.len() == 2);
        assert!(withdrawal_queue[0].recipient_addr == user2);
        assert!(withdrawal_queue[1].recipient_addr == user4);
        assert!(user3_lp.balance == Uint128::new(deposit_amount));

        let cancel_withdraw_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::CancelWithdrawalRequests { perp_id: 0 },
                &[],
            )
            .unwrap();

        let q_resp: WithdrawalsResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Withdrawals { perp_id: 0 })
            .unwrap();

        let withdrawal_queue = q_resp.withdrawal_queue;
        assert!(withdrawal_queue.len() == 1);
        assert!(withdrawal_queue[0].recipient_addr == user4);

        let cancel_withdraw_events = fetch_response_events(
            &cancel_withdraw_response,
            "cancelled_withdrawal_requests".to_string(),
        );
        assert!(cancel_withdraw_events.len() == 1);
        assert!(cancel_withdraw_events[0].ty == "wasm-cancelled_withdrawal_requests");
        assert!(cancel_withdraw_events[0].attributes.len() == 4);
        assert!(cancel_withdraw_events[0].attributes[1].key == "withdrawer");
        assert!(
            cancel_withdraw_events[0].attributes[1].value
                == "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap"
        );
        assert!(cancel_withdraw_events[0].attributes[2].key == "perp_id");
        assert!(cancel_withdraw_events[0].attributes[2].value == "0");
        assert!(cancel_withdraw_events[0].attributes[3].key == "restored_lp_tokens");
        assert!(cancel_withdraw_events[0].attributes[3].value == "1000");
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap does not have permission to process withdrawals"
    )]
    fn only_trader_can_process_withdrawals() {
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

        mint_native(
            &mut app,
            user2.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
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
                &ExecuteMsg::RequestWithdrawal {
                    perp_id: 0,
                    usdc_amount: 0u64,
                },
                &[],
            )
            .unwrap();

        let _process_withdrawal_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::ProcessWithdrawals {
                    perp_id: 0,
                    max_num_withdrawals: 1,
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn trader_can_process_multiple_withdrawal_requests_at_once() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let user3 = users[3].clone();
        let user4 = users[4].clone();
        let deposit_amount = 1_000_000;
        let withdraw_amount = 1_000u128;

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        // mint, deposit and request withdraw for all users
        for user in vec![user2.clone(), user3.clone(), user4.clone()] {
            mint_native(
                &mut app,
                user.to_string(),
                USDC_COIN_TYPE.to_string(),
                deposit_amount,
            );

            let _deposit_response = app
                .execute_contract(
                    user.clone(),
                    app_addr.clone(),
                    &ExecuteMsg::DepositIntoVault { perp_id: 0 },
                    &[Coin {
                        denom: USDC_COIN_TYPE.to_string(),
                        amount: Uint128::new(deposit_amount),
                    }],
                )
                .unwrap();

            let _request_withdraw_response = app
                .execute_contract(
                    user.clone(),
                    app_addr.clone(),
                    &ExecuteMsg::RequestWithdrawal {
                        perp_id: 0,
                        usdc_amount: withdraw_amount as u64,
                    },
                    &[],
                )
                .unwrap();
        }

        let q_resp: WithdrawalsResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Withdrawals { perp_id: 0 })
            .unwrap();

        let withdrawal_queue = q_resp.withdrawal_queue;
        assert!(withdrawal_queue.len() == 3);

        let process_withdrawal_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::ProcessWithdrawals {
                    perp_id: 0,
                    max_num_withdrawals: 8,
                },
                &[],
            )
            .unwrap();

        let q_resp: WithdrawalsResponse = app
            .wrap()
            .query_wasm_smart(app_addr.clone(), &QueryMsg::Withdrawals { perp_id: 0 })
            .unwrap();

        let withdrawal_queue = q_resp.withdrawal_queue;
        assert!(withdrawal_queue.len() == 0);

        for user in vec![user2.clone(), user3.clone(), user4.clone()] {
            let lp: LpTokenBalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    app_addr.clone(),
                    &QueryMsg::UserLpTokens {
                        perp_id: 0,
                        user: user.to_string(),
                    },
                )
                .unwrap();
            if user == user4 {
                assert!(lp.balance == Uint128::new(deposit_amount - withdraw_amount - 1));
            // round up # of LP tokens to burn
            } else {
                assert!(lp.balance == Uint128::new(deposit_amount - withdraw_amount));
            }
        }

        let processed_withdraw_events = fetch_response_events(
            &process_withdrawal_response,
            "processed_withdrawal".to_string(),
        );
        assert!(processed_withdraw_events.len() == 3);
        assert!(processed_withdraw_events[0].ty == "wasm-processed_withdrawal");
        assert!(processed_withdraw_events[0].attributes.len() == 5);
        assert!(processed_withdraw_events[0].attributes[1].key == "recipient");
        assert!(
            processed_withdraw_events[0].attributes[1].value
                == "cosmwasm1vqjarrly327529599rcc4qhzvhwe34pp5uyy4gylvxe5zupeqx3sg08lap"
        );
        assert!(processed_withdraw_events[0].attributes[2].key == "perp_id");
        assert!(processed_withdraw_events[0].attributes[2].value == "0");
        assert!(processed_withdraw_events[0].attributes[3].key == "withdrawn_usdc");
        assert!(processed_withdraw_events[0].attributes[3].value == "1000");
        assert!(processed_withdraw_events[0].attributes[4].key == "burnt_lp_tokens");
        assert!(processed_withdraw_events[0].attributes[4].value == "1000");

        assert!(processed_withdraw_events[1].attributes.len() == 5);
        assert!(processed_withdraw_events[1].attributes[1].key == "recipient");
        assert!(
            processed_withdraw_events[1].attributes[1].value
                == "cosmwasm1tps04uptd0rzy2a94jjjx4s0pcmyenvtv7lwfph730muq82f9n9s2w0guk"
        );
        assert!(processed_withdraw_events[1].attributes[2].key == "perp_id");
        assert!(processed_withdraw_events[1].attributes[2].value == "0");
        assert!(processed_withdraw_events[1].attributes[3].key == "withdrawn_usdc");
        assert!(processed_withdraw_events[1].attributes[3].value == "999");
        assert!(processed_withdraw_events[1].attributes[4].key == "burnt_lp_tokens");
        assert!(processed_withdraw_events[1].attributes[4].value == "1000");

        assert!(processed_withdraw_events[2].attributes.len() == 5);
        assert!(processed_withdraw_events[2].attributes[1].key == "recipient");
        assert!(
            processed_withdraw_events[2].attributes[1].value
                == "cosmwasm12f57lxqdu3upnw3azs6q73n9yckyr6fnmjfvrgna6hgpkpr6eq8qf0fk2p"
        );
        assert!(processed_withdraw_events[2].attributes[2].key == "perp_id");
        assert!(processed_withdraw_events[2].attributes[2].value == "0");
        assert!(processed_withdraw_events[2].attributes[3].key == "withdrawn_usdc");
        assert!(processed_withdraw_events[2].attributes[3].value == "999");
        assert!(processed_withdraw_events[2].attributes[4].key == "burnt_lp_tokens");
        assert!(processed_withdraw_events[2].attributes[4].value == "1000");
    }

    #[test]
    fn withdrawing_with_amount_as_0_withdraws_everything() {
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

        let _withdraw_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::RequestWithdrawal {
                    perp_id: 0,
                    usdc_amount: 0u64,
                },
                &[],
            )
            .unwrap();

        let _process_withdrawal_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::ProcessWithdrawals {
                    perp_id: 0,
                    max_num_withdrawals: 1,
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
        assert!(vault_resp.asset_usdc_value == Decimal::zero());
        assert!(vault_resp.perp_usdc_value == Decimal::zero());
        assert!(vault_resp.depositor_lp_tokens == Uint128::zero());
        assert!(vault_resp.outstanding_lp_tokens == Uint128::zero());
    }

    #[test]
    #[should_panic(
        expected = "Could not find LP tokens with perp_id: 0 for cosmwasm1tps04uptd0rzy2a94jjjx4s0pcmyenvtv7lwfph730muq82f9n9s2w0guk"
    )]
    fn only_users_with_deposits_can_request_withdrawal_from_vault() {
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
                &ExecuteMsg::RequestWithdrawal {
                    perp_id: 0,
                    usdc_amount: withdraw_amount as u64,
                },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn process_withdrawals_will_fail_if_it_increases_leverage_over_1x() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();
        let deposit_amount = 500_001;
        let perp_quantums = 82_802; // we want the value to be ~ $0.50, so 60384.18054 * x = 0.5

        let app_addr = instantiate_contract_with_trader_and_vault(
            &mut app,
            code_id,
            owner.clone(),
            user1.clone(),
        );

        mint_native(
            &mut app,
            user2.to_string(),
            USDC_COIN_TYPE.to_string(),
            deposit_amount,
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

        app.router().custom.sudo_add_perp_position(
            0,
            PerpetualPosition {
                perpetual_id: 0,
                quantums: SerializableInt::new(perp_quantums.into()),
                funding_index: SerializableInt::new(BigInt::ZERO),
            },
        );

        // this withdrawal works
        let _withdraw_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::RequestWithdrawal {
                    perp_id: 0,
                    usdc_amount: 1u64,
                },
                &[],
            )
            .unwrap();

        let _process_withdrawal_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::ProcessWithdrawals {
                    perp_id: 0,
                    max_num_withdrawals: 1,
                },
                &[],
            )
            .unwrap();

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
        assert!(vault_resp.asset_usdc_value == Decimal::from_atomics(5u128, 1).unwrap());
        assert!(
            vault_resp.perp_usdc_value == Decimal::from_atomics(499993091707308u128, 15).unwrap()
        );

        // now it will fail
        let _withdraw_response = app
            .execute_contract(
                user2.clone(),
                app_addr.clone(),
                &ExecuteMsg::RequestWithdrawal {
                    perp_id: 0,
                    usdc_amount: 6u64,
                },
                &[],
            )
            .unwrap();

        let process_withdrawal_response = app.execute_contract(
            user1.clone(),
            app_addr.clone(),
            &ExecuteMsg::ProcessWithdrawals {
                perp_id: 0,
                max_num_withdrawals: 1,
            },
            &[],
        );

        assert!(process_withdrawal_response.is_err());
        if let Some(error) = process_withdrawal_response
            .unwrap_err()
            .downcast_ref::<ContractError>()
        {
            assert_eq!(
                error,
                &ContractError::WithdrawalWouldIncreaseLeverageTooMuch { perp_id: 0 }
            );
        } else {
            panic!("Expected ContractError::WithdrawalWouldIncreaseLeverageTooMuch");
        }
    }
}
