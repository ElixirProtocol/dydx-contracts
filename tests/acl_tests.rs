#[cfg(test)]
mod tests {

    use cosmwasm_std::{testing::MockApi, Addr, Attribute, Event};
    use cw_multi_test::{App, AppResponse, ContractWrapper, Executor};
    use elixir_dydx_integration::msg::{AdminsResp, ExecuteMsg, InstantiateMsg, QueryMsg};

    fn test_setup() -> (App, u64, Vec<Addr>) {
        let mut app = App::default();

        let contract_wrapper = ContractWrapper::new(
            elixir_dydx_integration::contract::execute,
            elixir_dydx_integration::contract::instantiate,
            elixir_dydx_integration::contract::query,
        );
        let code_id = app.store_code(Box::new(contract_wrapper));

        let mock_api = MockApi::default();
        let owner = mock_api.addr_make("owner");
        let user1 = mock_api.addr_make("user1");
        let user2 = mock_api.addr_make("user2");

        (app, code_id, vec![owner, user1, user2])
    }

    fn instantiate_contract(app: &mut App, code_id: u64, owner: Addr) -> Addr {
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

    fn fetch_attributes(resp: &AppResponse, key: String) -> Vec<Attribute> {
        let wasm = resp.events.iter().find(|ev| ev.ty == "wasm").unwrap();
        wasm.attributes
        .iter()
        .cloned()
        .filter(|attr| attr.key == key)
        .collect()
    }

    fn fetch_response_events(resp: &AppResponse, event_name: String) -> Vec<Event> {
        resp.events
            .iter()
            .cloned()
            .filter(|ev| ev.ty == format!("wasm-{event_name}"))
            .collect()
    }

    #[test]
    fn can_instantiate_contract() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let resp: AdminsResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Admins {})
            .unwrap();
        assert_eq!(
            resp,
            AdminsResp {
                admin_addrs: vec![owner]
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
    fn owner_can_add_admins() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_admins = vec![user1.to_string()];
        let add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddAdmins { new_admins },
                &[],
            )
            .unwrap();

        let admins_resp: AdminsResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Admins {})
            .unwrap();

        assert!(admins_resp.admin_addrs.len() == 2);
        assert!(admins_resp.admin_addrs.contains(&owner));
        assert!(admins_resp.admin_addrs.contains(&user1));

        let admin_added_events = fetch_response_events(&add_response, "admin_added".to_string());
        assert!(admin_added_events.len() == 1);
        assert!(admin_added_events[0].ty == "wasm-admin_added");

        let action_attributes = fetch_attributes(&add_response, "action".to_string());
        assert!(action_attributes.len() == 1);
        assert!(action_attributes[0].key == "action");
        assert!(action_attributes[0].value == "add_admins");

        let count_attributes = fetch_attributes(&add_response, "added_count".to_string());
        assert!(count_attributes.len() == 1);
        assert!(count_attributes[0].key == "added_count");
        assert!(count_attributes[0].value == "1");
    }

    #[test]
    #[should_panic(
        expected = "cosmwasm1pgzph9rze2j2xxavx4n7pdhxlkgsq7rak245x0vk7mgh3j4le6gqmlwcfu does not have permission to modify admins"
    )]
    fn only_admins_can_add_admins() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_admins = vec![user1.to_string()];
        let add_response = app
            .execute_contract(
                user1.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddAdmins { new_admins },
                &[],
            )
            .unwrap();
    }

    #[test]
    fn owner_can_add_multiple_admins() {
        let (mut app, code_id, users) = test_setup();
        let owner = users[0].clone();
        let user1 = users[1].clone();
        let user2 = users[2].clone();

        let app_addr = instantiate_contract(&mut app, code_id, owner.clone());

        let new_admins = vec![user1.to_string(), user2.to_string()];
        let add_response = app
            .execute_contract(
                owner.clone(),
                app_addr.clone(),
                &ExecuteMsg::AddAdmins { new_admins },
                &[],
            )
            .unwrap();

        let admins_resp: AdminsResp = app
            .wrap()
            .query_wasm_smart(app_addr, &QueryMsg::Admins {})
            .unwrap();

        assert!(admins_resp.admin_addrs.len() == 3);
        assert!(admins_resp.admin_addrs.contains(&owner));
        assert!(admins_resp.admin_addrs.contains(&user1));
        assert!(admins_resp.admin_addrs.contains(&user2));

        let admin_added_events = fetch_response_events(&add_response, "admin_added".to_string());
        assert!(admin_added_events.len() == 2);

        let action_attributes = fetch_attributes(&add_response, "action".to_string());
        assert!(action_attributes.len() == 1);
        assert!(action_attributes[0].key == "action");
        assert!(action_attributes[0].value == "add_admins");

        let count_attributes = fetch_attributes(&add_response, "added_count".to_string());
        assert!(count_attributes.len() == 1);
        assert!(count_attributes[0].key == "added_count");
        assert!(count_attributes[0].value == "2");
    }
}
