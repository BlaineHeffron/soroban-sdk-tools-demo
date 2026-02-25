#![cfg(test)]

use crate::{DemoVault, DemoVaultClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

mod token_wasm {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/demo_token.wasm"
    );
}

fn setup(env: &Env) -> (Address, DemoVaultClient, token_wasm::Client, Address, Address) {
    let admin = Address::generate(env);
    let alice = Address::generate(env);

    // Deploy token
    let token_id = env.register(token_wasm::WASM, ());
    let token = token_wasm::Client::new(env, &token_id);

    // Deploy vault
    let vault_id = env.register(DemoVault, ());
    let vault = DemoVaultClient::new(env, &vault_id);

    env.mock_all_auths();

    // Initialize both contracts
    token.initialize(&admin);
    vault.initialize(&admin, &token_id);

    // Mint tokens to alice and approve the vault as spender
    token.mint(&alice, &10_000);
    token.approve(&alice, &vault_id, &10_000);

    (admin, vault, token, alice, token_id)
}

// ---------------------------------------------------------------------------
// Cross-contract flow: deposit + withdraw
// ---------------------------------------------------------------------------

#[test]
fn test_deposit_and_withdraw() {
    let env = Env::default();
    let (_admin, vault, token, alice, _) = setup(&env);

    vault.deposit(&alice, &3000);
    assert_eq!(vault.get_deposit(&alice), 3000);
    assert_eq!(vault.total_deposited(), 3000);
    assert_eq!(token.balance(&alice), 7000);

    vault.withdraw(&alice, &1000);
    assert_eq!(vault.get_deposit(&alice), 2000);
    assert_eq!(token.balance(&alice), 8000);
}

// ---------------------------------------------------------------------------
// Error propagation: token error surfaces through vault
// ---------------------------------------------------------------------------

#[test]
fn test_deposit_exceeds_allowance() {
    let env = Env::default();
    let (_admin, vault, _token, alice, _) = setup(&env);

    // Alice only approved 10_000. Depositing more should fail
    // with a token error propagated through #[from_contract_client]
    let result = vault.try_deposit(&alice, &20_000);
    assert!(result.is_err());
}

#[test]
fn test_withdraw_exceeds_deposit() {
    let env = Env::default();
    let (_admin, vault, _token, alice, _) = setup(&env);

    vault.deposit(&alice, &1000);

    let result = vault.try_withdraw(&alice, &5000);
    assert!(result.is_err());
}
