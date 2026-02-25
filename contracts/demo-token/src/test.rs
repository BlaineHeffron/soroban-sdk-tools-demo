#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk_tools::{Keypair, Secp256r1Keypair, Signer};

// Import via contractimport so we get AuthClient
mod token {
    soroban_sdk_tools::contractimport!(
        file = "../../target/wasm32v1-none/release/demo_token.wasm"
    );
}

fn setup(env: &Env) -> (Address, Address, Address, Address) {
    let contract_id = env.register(token::WASM, ());
    let admin = Address::generate(env);
    let alice = Address::generate(env);
    let bob = Address::generate(env);

    env.mock_all_auths();
    token::Client::new(env, &contract_id).initialize(&admin);
    token::Client::new(env, &contract_id).mint(&alice, &1000);

    (contract_id, admin, alice, bob)
}

// ---------------------------------------------------------------------------
// 1. Storage — #[contractstorage] generates typed accessors
// ---------------------------------------------------------------------------

#[test]
fn test_mint_and_balance() {
    let env = Env::default();
    let (contract_id, _, alice, _) = setup(&env);
    let client = token::Client::new(&env, &contract_id);

    assert_eq!(client.balance(&alice), 1000);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    let (contract_id, _, alice, bob) = setup(&env);
    let client = token::Client::new(&env, &contract_id);

    client.transfer(&alice, &bob, &300);

    assert_eq!(client.balance(&alice), 700);
    assert_eq!(client.balance(&bob), 300);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    let (contract_id, _, alice, bob) = setup(&env);
    let client = token::Client::new(&env, &contract_id);
    let spender = Address::generate(&env);

    client.approve(&alice, &spender, &500);
    assert_eq!(client.allowance(&alice, &spender), 500);

    client.transfer_from(&spender, &alice, &bob, &200);

    assert_eq!(client.balance(&alice), 800);
    assert_eq!(client.balance(&bob), 200);
    assert_eq!(client.allowance(&alice, &spender), 300);
}

// ---------------------------------------------------------------------------
// 2. Errors — #[scerr] errors come back as typed variants
// ---------------------------------------------------------------------------

#[test]
fn test_insufficient_balance_error() {
    let env = Env::default();
    let (contract_id, _, alice, bob) = setup(&env);
    let client = token::Client::new(&env, &contract_id);

    let result = client.try_transfer(&alice, &bob, &5000);
    assert!(result.is_err());
}

#[test]
fn test_already_initialized_error() {
    let env = Env::default();
    let (contract_id, admin, _, _) = setup(&env);
    let client = token::Client::new(&env, &contract_id);

    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 3. Auth — AuthClient with .authorize() for mock auth
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_with_auth_client() {
    let env = Env::default();
    let contract_id = env.register(token::WASM, ());
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Use mock_all_auths for setup only
    env.mock_all_auths();
    let client = token::Client::new(&env, &contract_id);
    client.initialize(&admin);
    client.mint(&alice, &1000);

    // AuthClient: targeted mock auth via builder pattern
    let auth = token::AuthClient::new(&env, &contract_id);
    auth.transfer(&alice, &bob, &300).authorize(&alice).invoke();

    assert_eq!(client.balance(&alice), 700);
    assert_eq!(client.balance(&bob), 300);
}

// ---------------------------------------------------------------------------
// 4. Auth — AuthClient with .sign() for real crypto signatures
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_with_ed25519_signature() {
    let env = Env::default();
    let contract_id = env.register(token::WASM, ());
    let admin = Address::generate(&env);
    let bob = Address::generate(&env);

    // Generate a real ed25519 keypair (registers a Stellar account on the ledger)
    let kp = Keypair::random(&env);
    let alice = kp.address().clone();

    env.mock_all_auths();
    let client = token::Client::new(&env, &contract_id);
    client.initialize(&admin);
    client.mint(&alice, &1000);

    // .sign() uses real cryptographic signatures — no mocking
    let auth = token::AuthClient::new(&env, &contract_id);
    auth.transfer(&alice, &bob, &300).sign(&kp).invoke();

    assert_eq!(client.balance(&alice), 700);
    assert_eq!(client.balance(&bob), 300);
}

#[test]
fn test_transfer_with_passkey_signature() {
    let env = Env::default();
    let contract_id = env.register(token::WASM, ());
    let admin = Address::generate(&env);
    let bob = Address::generate(&env);

    // Generate a real secp256r1 (WebAuthn/passkey) keypair
    // This registers a custom account contract that verifies signatures
    let kp = Secp256r1Keypair::random(&env);
    let signer_addr = kp.address().clone();

    env.mock_all_auths();
    let client = token::Client::new(&env, &contract_id);
    client.initialize(&admin);
    client.mint(&signer_addr, &1000);

    // Real passkey signature — the custom account contract verifies it
    let auth = token::AuthClient::new(&env, &contract_id);
    auth.transfer(&signer_addr, &bob, &500)
        .sign(&kp)
        .invoke();

    assert_eq!(client.balance(&signer_addr), 500);
    assert_eq!(client.balance(&bob), 500);
}
