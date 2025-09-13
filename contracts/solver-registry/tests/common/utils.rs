use std::collections::HashSet;
use std::str::FromStr;

use near_contract_standards::fungible_token::{metadata::FungibleTokenMetadata, Balance};
use near_gas::NearGas;
use near_sdk::{AccountId, NearToken, PublicKey};
use near_workspaces::{
    network::Sandbox, result::ExecutionFinalResult, types::SecretKey, Account, Contract, Worker,
};
use serde_json::json;
use solver_registry::{pool::PoolInfo, types::TimestampMs};

use super::constants::*;

type WorkerInfo = solver_registry::Worker;

pub const SOLVER_REGISTRY_CONTRACT_WASM: &str =
    "../../target/near/solver_registry/solver_registry.wasm";
pub const MOCK_INTENTS_CONTRACT_WASM: &str = "../../target/near/mock_intents/mock_intents.wasm";
pub const MOCK_FT_CONTRACT_WASM: &str = "../../target/near/mock_ft/mock_ft.wasm";

pub async fn create_account(
    sandbox: &Worker<Sandbox>,
    prefix: &str,
    balance: Balance,
) -> Result<Account, Box<dyn std::error::Error>> {
    let root = sandbox.root_account().unwrap();
    Ok(root
        .create_subaccount(prefix)
        .initial_balance(NearToken::from_near(balance))
        .transact()
        .await?
        .result)
}

pub async fn create_account_with_secret_key(
    sandbox: &Worker<Sandbox>,
    prefix: &str,
    balance: Balance,
    secret_key: SecretKey,
) -> Result<Account, Box<dyn std::error::Error>> {
    let root = sandbox.root_account().unwrap();
    Ok(root
        .create_subaccount(prefix)
        .initial_balance(NearToken::from_near(balance))
        .keys(secret_key)
        .transact()
        .await?
        .result)
}

pub async fn create_ft(
    sandbox: &Worker<Sandbox>,
    name: &str,
    symbol: &str,
    decimals: u32,
    total_supply: Balance,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let mock_ft_contract_wasm =
        std::fs::read(MOCK_FT_CONTRACT_WASM).expect("Contract wasm not found");

    let ft_account = create_account(sandbox, symbol.to_lowercase().as_str(), 100).await?;
    let ft_contract = ft_account.deploy(&mock_ft_contract_wasm).await?.result;
    let result = ft_contract
        .call("new")
        .args_json(json!({
            "owner_id": ft_contract.id(),
            "total_supply": total_supply.to_string(),
            "metadata": {
                "spec": "ft-1.0.0".to_string(),
                "name": name.to_string(),
                "symbol": symbol.to_string(),
                "icon": None::<String>,
                "reference": None::<String>,
                "reference_hash": None::<String>,
                "decimals": decimals,
            }
        }))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{:#?}",
        result.into_result().unwrap_err()
    );

    let result = ft_contract.view("ft_metadata").await?;
    let metadata: FungibleTokenMetadata = serde_json::from_slice(&result.result).unwrap();
    println!(
        "\n [LOG] FT metadata: {{ name: {}, symbol: {}, decimals: {} }}",
        metadata.name, metadata.symbol, metadata.decimals
    );

    Ok(ft_contract)
}

pub async fn storage_deposit(
    ft: &Contract,
    account: &Account,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = ft
        .call("storage_deposit")
        .args_json(json!({
            "account_id": account.id(),
            "registration_only": true
        }))
        .deposit(NearToken::from_millinear(1250))
        .transact()
        .await?;

    Ok(result)
}

pub async fn ft_transfer(
    ft: &Contract,
    sender: &Account,
    receiver: &Account,
    amount: Balance,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = sender
        .call(ft.id(), "ft_transfer")
        .args_json(json!({
            "receiver_id": receiver.id(),
            "amount": amount.to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    println!("\nResult: transfer {} {} FT {:?}", amount, ft.id(), result);

    Ok(result)
}

pub async fn deploy_mock_intents(
    sandbox: &Worker<Sandbox>,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let mock_intents_contract_wasm =
        std::fs::read(MOCK_INTENTS_CONTRACT_WASM).expect("Contract wasm not found");
    let mock_intents_account = create_account(sandbox, "intents", 100).await?;
    let mock_intents_contract = mock_intents_account
        .deploy(&mock_intents_contract_wasm)
        .await?
        .result;

    println!("Initializing solver mock intents contract...");
    let result = mock_intents_contract.call("new").transact().await?;
    println!("\nResult init: {:?}", result);

    Ok(mock_intents_contract)
}

pub async fn deploy_solver_registry(
    sandbox: &Worker<Sandbox>,
    intents_contract: &Contract,
    owner: &Account,
    worker_ping_timeout_ms: TimestampMs,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let solver_registry_contract_wasm =
        std::fs::read(SOLVER_REGISTRY_CONTRACT_WASM).expect("Contract wasm not found");
    let solver_registry_account = create_account(sandbox, "solver-registry", 100).await?;
    let solver_registry_contract = solver_registry_account
        .deploy(&solver_registry_contract_wasm)
        .await?
        .result;

    println!("Initializing solver registry contract...");
    let result = solver_registry_contract
        .call("new")
        .args_json(json!({
            "owner_id": owner.id(),
            "intents_contract_id": intents_contract.id(),
            "worker_ping_timeout_ms": worker_ping_timeout_ms
        }))
        .transact()
        .await?;
    println!("\nResult init: {:?}", result);

    Ok(solver_registry_contract)
}

pub async fn deposit_into_pool(
    solver_registry: &Contract,
    user: &Account,
    pool_id: u32,
    ft: &Contract,
    amount: Balance,
) -> Result<ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = user
        .call(ft.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": solver_registry.id(),
            "amount": amount.to_string(),
            "msg": json!({
                "DepositIntoPool": {
                    "pool_id": pool_id
                }
            }).to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(200))
        .transact()
        .await?;
    println!("\nResult: deposit token {:?}", result);

    Ok(result)
}

// Helper function to print execution logs
pub fn print_logs(result: &near_workspaces::result::ExecutionFinalResult) {
    for (i, log) in result.logs().iter().enumerate() {
        println!("  [{}] {}", i + 1, log);
    }
}

// Helper function to create test tokens (wNEAR and USDC)
pub async fn create_test_tokens(
    sandbox: &Worker<Sandbox>,
) -> Result<(Contract, Contract), Box<dyn std::error::Error>> {
    println!("Deploying wNEAR contract...");
    let wnear = create_ft(
        sandbox,
        "Wrapped NEAR",
        "wNEAR",
        24,
        NearToken::from_near(1_000_000_000).as_yoctonear(), // 1B
    )
    .await?;

    println!("Deploying USDC contract...");
    let usdc = create_ft(
        sandbox,
        "USD Coin",
        "USDC",
        6,
        10_000_000_000_000_000, // 10B
    )
    .await?;

    Ok((wnear, usdc))
}

// Helper function to create test accounts (owner, alice, bob)
pub async fn create_test_accounts(
    sandbox: &Worker<Sandbox>,
) -> Result<(Account, Account, Account), Box<dyn std::error::Error>> {
    let owner = create_account(sandbox, "owner", 10).await?;
    let alice = create_account_with_secret_key(
        sandbox,
        "alice",
        10,
        SecretKey::from_str(SECRET_KEY_ALICE).unwrap(),
    )
    .await?;
    let bob = create_account_with_secret_key(
        sandbox,
        "bob",
        10,
        SecretKey::from_str(SECRET_KEY_BOB).unwrap(),
    )
    .await?;

    Ok((owner, alice, bob))
}

// Helper function to register accounts for NEP-141 tokens
pub async fn register_accounts_for_tokens(
    wnear: &Contract,
    usdc: &Contract,
    accounts: &[&Account],
) -> Result<(), Box<dyn std::error::Error>> {
    for account in accounts {
        let _ = storage_deposit(wnear, account).await?;
        let _ = storage_deposit(usdc, account).await?;
    }
    Ok(())
}

// Helper function to setup the complete test environment
pub async fn setup_test_environment(
    sandbox: &Worker<Sandbox>,
    worker_ping_timeout_ms: TimestampMs,
) -> Result<
    (
        Contract,
        Contract,
        Account,
        Account,
        Account,
        Contract,
        Contract,
    ),
    Box<dyn std::error::Error>,
> {
    // Create test tokens
    let (wnear, usdc) = create_test_tokens(sandbox).await?;

    // Create test accounts
    let (owner, alice, bob) = create_test_accounts(sandbox).await?;

    // Register accounts for NEP-141 tokens
    register_accounts_for_tokens(&wnear, &usdc, &[&alice, &bob]).await?;

    // Deploy contracts
    println!("Deploying mock intents contract...");
    let mock_intents = deploy_mock_intents(sandbox).await?;

    println!("Deploying Solver Registry contract...");
    let solver_registry =
        deploy_solver_registry(sandbox, &mock_intents, &owner, worker_ping_timeout_ms).await?;

    // Register contracts for NEP-141 tokens
    register_accounts_for_tokens(
        &wnear,
        &usdc,
        &[mock_intents.as_account(), solver_registry.as_account()],
    )
    .await?;

    Ok((
        wnear,
        usdc,
        owner,
        alice,
        bob,
        mock_intents,
        solver_registry,
    ))
}

// Helper function to create a liquidity pool
pub async fn create_liquidity_pool(
    solver_registry: &Contract,
    wnear: &Contract,
    usdc: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating liquidity pool...");
    let result = solver_registry
        .call("create_liquidity_pool")
        .args_json(json!({
            "token_ids": [wnear.id(), usdc.id()],
            "fee": 300
        }))
        .deposit(NearToken::from_yoctonear(1_500_000_000_000_000_000_000_000)) // 1.5 NEAR
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{:#?}",
        result.into_result().unwrap_err()
    );
    Ok(())
}

// Helper function to approve compose hash
pub async fn approve_compose_hash(
    owner: &Account,
    solver_registry: &Contract,
) -> Result<(), Box<dyn std::error::Error>> {
    let result = owner
        .call(solver_registry.id(), "approve_compose_hash")
        .deposit(NearToken::from_yoctonear(1))
        .args_json(json!({
            "compose_hash": COMPOSE_HASH
        }))
        .transact()
        .await?;
    assert!(
        result.is_success(),
        "{:#?}",
        result.into_result().unwrap_err()
    );
    Ok(())
}

// Helper function to register a worker
pub async fn register_worker(
    worker: &Account,
    solver_registry: &Contract,
    pool_id: u32,
    quote_hex: &str,
    collateral: &str,
    checksum: &str,
    tcb_info: &str,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = worker
        .call(solver_registry.id(), "register_worker")
        .args_json(json!({
            "pool_id": pool_id,
            "quote_hex": quote_hex.to_string(),
            "collateral": collateral.to_string(),
            "checksum": checksum.to_string(),
            "tcb_info": tcb_info.to_string()
        }))
        .deposit(NearToken::from_yoctonear(1))
        .gas(NearGas::from_tgas(300))
        .transact()
        .await?;

    print_logs(&result);
    Ok(result)
}

// Helper function to register Alice as a worker
pub async fn register_worker_alice(
    alice: &Account,
    solver_registry: &Contract,
    pool_id: u32,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    register_worker(
        alice,
        solver_registry,
        pool_id,
        QUOTE_HEX_ALICE,
        QUOTE_COLLATERAL_ALICE,
        CHECKSUM_ALICE,
        TCB_INFO_ALICE,
    )
    .await
}

// Helper function to register Bob as a worker
pub async fn register_worker_bob(
    bob: &Account,
    solver_registry: &Contract,
    pool_id: u32,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    register_worker(
        bob,
        solver_registry,
        pool_id,
        QUOTE_HEX_BOB,
        QUOTE_COLLATERAL_BOB,
        CHECKSUM_BOB,
        TCB_INFO_BOB,
    )
    .await
}

// Helper function to wait for worker timeout
pub async fn wait_for_worker_timeout(timeout_seconds: u64) {
    println!(
        "Waiting for worker timeout ({} seconds)...",
        timeout_seconds
    );
    tokio::time::sleep(tokio::time::Duration::from_secs(timeout_seconds + 1)).await;
    // Add 1 second buffer
}

// Helper function to get worker info
pub async fn get_worker_info(
    solver_registry: &Contract,
    account_id: &Account,
) -> Result<Option<WorkerInfo>, Box<dyn std::error::Error>> {
    let result = solver_registry
        .view("get_worker")
        .args_json(json!({"account_id" : account_id.id()}))
        .await?;
    let worker_info: Option<WorkerInfo> = serde_json::from_slice(&result.result).unwrap();
    Ok(worker_info)
}

// Helper function to get pool info
pub async fn get_pool_info(
    solver_registry: &Contract,
    pool_id: u32,
) -> Result<PoolInfo, Box<dyn std::error::Error>> {
    let result = solver_registry
        .view("get_pool")
        .args_json(json!({"pool_id" : pool_id}))
        .await?;
    let pool_info: PoolInfo = serde_json::from_slice(&result.result).unwrap();
    Ok(pool_info)
}

// Helper function to ping as a worker
pub async fn ping_worker(
    worker: &Account,
    solver_registry: &Contract,
) -> Result<near_workspaces::result::ExecutionFinalResult, Box<dyn std::error::Error>> {
    let result = worker.call(solver_registry.id(), "ping").transact().await?;
    Ok(result)
}

// Helper function to demonstrate active worker pinging
pub async fn demonstrate_active_worker_pinging(
    worker: &Account,
    solver_registry: &Contract,
    num_pings: u32,
    delay_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Demonstrating active worker pinging with {} pings and {}ms delay...",
        num_pings, delay_ms
    );

    for i in 1..=num_pings {
        println!("Ping {} of {}...", i, num_pings);
        let result = ping_worker(worker, solver_registry).await?;
        assert!(
            result.is_success(),
            "Worker ping {} should succeed: {:#?}",
            i,
            result.into_result().unwrap_err()
        );

        // Configurable delay between pings to ensure timestamp differences
        if i < num_pings {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }
    }

    println!("Active worker pinging demonstration completed");
    Ok(())
}

// Helper function to get public keys from mock-intents contract for a pool
pub async fn get_pool_public_keys(
    mock_intents: &Contract,
    pool_account_id: &AccountId,
) -> Result<HashSet<PublicKey>, Box<dyn std::error::Error>> {
    let result = mock_intents
        .view("public_keys_of")
        .args_json(json!({"account_id": pool_account_id}))
        .await?;
    let public_keys: HashSet<PublicKey> = serde_json::from_slice(&result.result).unwrap();
    Ok(public_keys)
}

// Helper function to get pool account ID from pool ID
pub fn get_pool_account_id(solver_registry: &Contract, pool_id: u32) -> AccountId {
    format!("pool-{}.{}", pool_id, solver_registry.id())
        .parse()
        .unwrap()
}
