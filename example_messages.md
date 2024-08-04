deploy elixir integration contract:
    wasmd tx wasm store /path/to/elixir-dydx-integration/target/wasm32-unknown-unknown/release/elixir_dydx_integration.wasm --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

instantiate the contract: 
    wasmd tx wasm instantiate X '{"owner":"dydx199tqg4wdlnu4qjlxchpd7seg454937hjrknju4"}' --from alice --label test --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol --admin alice

figure out contract address:
    wasmd query wasm list-contract-by-code "X"

create vault: 
    wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"create_vault": {"perp_id": 0}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

deposit:
    wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"deposit_into_vault": {"perp_id": 0}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol --amount 100000000ibc/8E27BA2D5493AF5636760E354E46004562C46AB7EC0CC4C1CA14E9E20E2545B5

    wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"deposit_into_vault": {"perp_id": 0}}' --from bob --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol --amount 100000000ibc/8E27BA2D5493AF5636760E354E46004562C46AB7EC0CC4C1CA14E9E20E2545B5

 request withdrawal:
    wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"request_withdrawal":{"perp_id":0,"usdc_amount":100}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

 process withdrawal:
    wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"process_withdrawals":{"perp_id":0,"max_num_withdrawals":1}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

cancel withdrawals: 
    wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"cancel_withdrawal_requests":{"perp_id":0}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

place order: 
 wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"market_make":{"subaccount_number":0,"clob_pair_id":0,"new_orders":[{"client_id":101,"side":1,"quantums":1000000,"subticks":100000,"good_til_block_time":1720791702,"time_in_force":0,"reduce_only":false,"client_metadata":0,"conditional_order_trigger_subticks":0}],"cancel_client_ids":[],"cancel_good_til_block_time":0}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

cancel order: 
 wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"market_make":{"subaccount_number":0,"clob_pair_id":0,"new_orders":[],"cancel_client_ids":[101],"cancel_good_til_block_time":1721231980}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol

batch cancel: 
  wasmd tx wasm execute dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"batch_cancel":{"subaccount_number":0,"order_batches":[{"clob_pair_id":0,"client_ids":[101,102]}],"good_til_block":123}}' --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol


query withdrawal queue:
     wasmd query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"withdrawals": {"perp_id": 0}}'

query subaccount: 
    wasmd query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"dydx_subaccount": {"owner": "dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j", "number": 0}}'

query trader:
    wasmd query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '"trader"' 

query dydx account value: 
    wasmd query wasm contract-state smart dydx1fyr2mptjswz4w6xmgnpgm93x0q4s4wdl6srv3rtz3utc4f6fmxeqr4pkyd '{"vault_ownership": {"perp_id": 0, "depositor": "dydx199tqg4wdlnu4qjlxchpd7seg454937hjrknju4"}}'
    wasmd query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"vault_ownership": {"perp_id": 0, "depositor": "dydx10fx7sy6ywd5senxae9dwytf8jxek3t2gcen2vs"}}'

query dydx vault state: 
    wasmd query wasm contract-state smart dydx14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s2de90j '{"vault_state": {"perp_id": 0}}'








migration: 
    wasmd tx wasm migrate <old-contract-address> "<new-code-id>" "{}" --from alice --gas-prices 25000000000adv4tnt --gas auto --gas-adjustment 1.5 --chain-id localdydxprotocol
