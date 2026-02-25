#![cfg(test)]

use crate::{ManualToken, ManualTokenClient};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

fn setup(env: &Env) -> (Address, ManualTokenClient, Address, Address, Address) {
    let contract_id = env.register(ManualToken, ());
    let client = ManualTokenClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let alice = Address::generate(env);
    let bob = Address::generate(env);

    env.mock_all_auths();
    client.initialize(&admin);
    client.mint(&alice, &1000);

    (contract_id, client, admin, alice, bob)
}

// ---------------------------------------------------------------------------
// 1. Storage — same tests, same assertions
// ---------------------------------------------------------------------------

#[test]
fn test_mint_and_balance() {
    let env = Env::default();
    let (_, client, _, alice, _) = setup(&env);

    assert_eq!(client.balance(&alice), 1000);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
fn test_transfer() {
    let env = Env::default();
    let (_, client, _, alice, bob) = setup(&env);

    client.transfer(&alice, &bob, &300);

    assert_eq!(client.balance(&alice), 700);
    assert_eq!(client.balance(&bob), 300);
    assert_eq!(client.total_supply(), 1000);
}

#[test]
fn test_approve_and_transfer_from() {
    let env = Env::default();
    let (_, client, _, alice, bob) = setup(&env);
    let spender = Address::generate(&env);

    client.approve(&alice, &spender, &500);
    assert_eq!(client.allowance(&alice, &spender), 500);

    client.transfer_from(&spender, &alice, &bob, &200);

    assert_eq!(client.balance(&alice), 800);
    assert_eq!(client.balance(&bob), 200);
    assert_eq!(client.allowance(&alice, &spender), 300);
}

// ---------------------------------------------------------------------------
// 2. Errors — same tests
// ---------------------------------------------------------------------------

#[test]
fn test_insufficient_balance_error() {
    let env = Env::default();
    let (_, client, _, alice, bob) = setup(&env);

    let result = client.try_transfer(&alice, &bob, &5000);
    assert!(result.is_err());
}

#[test]
fn test_already_initialized_error() {
    let env = Env::default();
    let (_, client, admin, _, _) = setup(&env);

    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 3. Auth — manual MockAuth setup (compare to AuthClient version)
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_with_manual_mock_auth() {
    let env = Env::default();
    let contract_id = env.register(ManualToken, ());
    let client = ManualTokenClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(&admin);
    client.mint(&alice, &1000);

    // Manual MockAuth: you build the struct yourself, specifying
    // the contract address, function name, and args
    env.mock_auths(&[MockAuth {
        address: &alice,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "transfer",
            args: (alice.clone(), bob.clone(), 300i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.transfer(&alice, &bob, &300);

    assert_eq!(client.balance(&alice), 700);
    assert_eq!(client.balance(&bob), 300);
}

// ---------------------------------------------------------------------------
// 4. Auth — no real-signature equivalent without soroban-sdk-tools
//
// With raw SDK you'd need to:
//   - Write a custom account contract by hand
//   - Register it on the test ledger
//   - Build SorobanAuthorizationEntry structs manually
//   - Hash the preimage, sign it, pack it into ScVal
//   - Call env.set_auths()
//
// That's ~80 lines of XDR plumbing per test. The soroban-sdk-tools
// version is 3 lines: Keypair::random, .sign(&kp), .invoke().
// ---------------------------------------------------------------------------
