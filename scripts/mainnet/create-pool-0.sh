export NEAR_ENV=mainnet
export INTENTS_CONTRACT=intents.near
export SOLVER_REGISTRY_CONTRACT=solver-registry-dev.near
export SOLVER_GOV_ACCOUNT=solver-gov.near
export SOLVER_TESTER_ACCOUNT=solver-alpha.near
export WNEAR_TOKEN=wrap.near
export USDC_TOKEN=17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1
export POOL_ID=0
export POOL_CONTRACT='pool-'$POOL_ID'.'$SOLVER_REGISTRY_CONTRACT

# --- prepare funds ---

# deposit NEAR for wNEAR
near call $WNEAR_TOKEN near_deposit '{}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 1
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'

# get some USDC from Rhea
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'


# --- create pool ---

# create liquidity pool
near call $SOLVER_REGISTRY_CONTRACT create_liquidity_pool '{"token_ids":["'$WNEAR_TOKEN'","'$USDC_TOKEN'"],"fee":100}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 1.5 --gas 300000000000000

# register accounts
near call $WNEAR_TOKEN storage_deposit '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'","registration_only":true}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 0.00125
near call $USDC_TOKEN storage_deposit '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'","registration_only":true}' --accountId $SOLVER_TESTER_ACCOUNT --deposit 0.00125

# deposit into pool
near call $WNEAR_TOKEN ft_transfer_call '{"receiver_id":"'$SOLVER_REGISTRY_CONTRACT'","amount":"1000000000000000000000000","msg":"{\"DepositIntoPool\": {\"pool_id\": '$POOL_ID'}}"}' --accountId $SOLVER_TESTER_ACCOUNT --gas 300000000000000 --depositYocto 1
near call $USDC_TOKEN ft_transfer_call '{"receiver_id":"'$SOLVER_REGISTRY_CONTRACT'","amount":"2000000","msg":"{\"DepositIntoPool\": {\"pool_id\": '$POOL_ID'}}"}' --accountId $SOLVER_TESTER_ACCOUNT --gas 300000000000000 --depositYocto 1


# --- check balances ---

# check balance (USDC)
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$INTENTS_CONTRACT'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'"}'
near view $USDC_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'

# check balance (wNEAR)
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$INTENTS_CONTRACT'"}'
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$SOLVER_REGISTRY_CONTRACT'"}'
near view $WNEAR_TOKEN ft_balance_of '{"account_id":"'$SOLVER_TESTER_ACCOUNT'"}'

# check mt balance
near view $INTENTS_CONTRACT mt_balance_of '{"account_id":"'$POOL_CONTRACT'","token_id":"nep141:'$WNEAR_TOKEN'"}'
near view $INTENTS_CONTRACT mt_balance_of '{"account_id":"'$POOL_CONTRACT'","token_id":"nep141:'$USDC_TOKEN'"}'
