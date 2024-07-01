deploy elixir integration contract:
    ./build/dydxprotocold tx wasm store /home/d/lou-dev/elixir-dydx-integration/target/wasm32-unknown-unknown/release/elixir_dydx_integration.wasm --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

instantiate the contract: 
    ./build/dydxprotocold tx wasm instantiate 1 '{"owner":"dydx199tqg4wdlnu4qjlxchpd7seg454937hjrknju4"}' --from alice --label test --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol --no-admin

create vault: 
    ./build/dydxprotocold tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"create_vault": {"perp_id": 0}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

deposit:
    ./build/dydxprotocold tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"deposit_into_vault": {"perp_id": 0}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol --amount 100000000ibc/8E27BA2D5493AF5636760E354E46004562C46AB7EC0CC4C1CA14E9E20E2545B5

    ./build/dydxprotocold tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"deposit_into_vault": {"perp_id": 0}}' --from bob --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol --amount 100000000ibc/8E27BA2D5493AF5636760E354E46004562C46AB7EC0CC4C1CA14E9E20E2545B5

query dydx account value: 
    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"vault_ownership": {"perp_id": 0, "depositor": "dydx199tqg4wdlnu4qjlxchpd7seg454937hjrknju4"}}'
    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"vault_ownership": {"perp_id": 0, "depositor": "dydx10fx7sy6ywd5senxae9dwytf8jxek3t2gcen2vs"}}'

place order: 
 ./build/dydxprotocold tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"place_order":{"subaccount_number":0,"client_id":101,"order_flags":64,"clob_pair_id":0,"side":1,"quantums":1000000,"subticks":100000,"good_til_block_time":1720791702,"time_in_force":0,"reduce_only":false,"client_metadata":0,"condition_type":0,"conditional_order_trigger_subticks":0}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

query subaccount: 
    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"other": {"perp_id": 0}}'

cancel order: 
 ./build/dydxprotocold tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"cancel_order":{"subaccount_number":0,"client_id":101,"order_flags":64,"clob_pair_id":0,"good_til_block_time":1720791702}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol


 withdraw:
    

















query dydx deposit: 
    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"dydx_subaccount": {"owner": "dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j", "number": 0}}'

    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"dydx_subaccount": {"owner": "dydx199tqg4wdlnu4qjlxchpd7seg454937hjrknju4", "number": 0}}'
    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"dydx_subaccount": {"owner": "dydx10fx7sy6ywd5senxae9dwytf8jxek3t2gcen2vs", "number": 0}}'
    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"dydx_subaccount": {"owner": "dydx1fjg6zp6vv8t9wvy4lps03r5l4g7tkjw9wvmh70", "number": 0}}'
    ./build/dydxprotocold query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"dydx_subaccount": {"owner": "dydx1wau5mja7j7zdavtfq9lu7ejef05hm6ffenlcsn", "number": 0}}'
