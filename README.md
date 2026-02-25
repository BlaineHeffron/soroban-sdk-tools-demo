# soroban-sdk-tools demo

Companion repo for the video walkthrough of [soroban-sdk-tools](https://crates.io/crates/soroban-sdk-tools). Three contracts: one written with raw SDK boilerplate, the same thing rewritten with the macros, and a vault that shows cross-contract error composition.

## Setup

```sh
cargo add soroban-sdk-tools
cargo add soroban-sdk-tools --dev --features testutils
```

Build the token WASM first (the vault imports it):

```sh
cargo build --release --target wasm32v1-none -p demo-token
cargo build --release --target wasm32v1-none -p demo-vault
```

Run everything:

```sh
cargo test
```

## Video walkthrough

### Before/after: manual SDK vs soroban-sdk-tools

Open `contracts/manual-token/src/lib.rs` and `contracts/demo-token/src/lib.rs` side by side. They do the same thing.

#### Storage

Manual (22 lines):

```rust
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Balance(Address),
    Allowance(Address, Address),
    TotalSupply,
}

// then in every method:
let key = DataKey::Balance(to.clone());
let balance: i128 = env.storage().persistent().get(&key).unwrap_or(0);
env.storage().persistent().set(&key, &(balance + amount));
env.storage().persistent().extend_ttl(&key, 50, 100);
```

With soroban-sdk-tools (6 lines for the definition, 1 line per operation):

```rust
#[contractstorage(auto_shorten = true)]
pub struct TokenStorage {
    admin: InstanceItem<Address>,
    balances: PersistentMap<Address, i128>,
    allowances: PersistentMap<(Address, Address), i128>,
    total_supply: InstanceItem<i128>,
}

// then:
TokenStorage::update_balances(env, to, |bal| bal.unwrap_or(0) + amount);
TokenStorage::extend_balances_ttl(env, to, 50, 100);
```

The struct declaration is the entire storage schema. The macro generates `get_*`, `set_*`, `update_*`, `remove_*`, `has_*`, and `extend_*_ttl` methods for each field. `auto_shorten` hashes the keys for 30-40% storage fee savings.

#### Errors

Manual:

```rust
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InsufficientBalance = 3,
    InsufficientAllowance = 4,
    InvalidAmount = 5,
}
```

With soroban-sdk-tools:

```rust
#[scerr]
pub enum TokenError {
    /// already initialized
    AlreadyInitialized,
    /// not initialized
    NotInitialized,
    /// insufficient balance
    InsufficientBalance,
    /// insufficient allowance
    InsufficientAllowance,
    /// amount must be positive
    InvalidAmount,
}
```

`#[scerr]` assigns the codes sequentially and generates all the derive/repr boilerplate. The doc comments become error descriptions in the WASM spec. More importantly, `#[scerr]` enables composition with `#[from_contract_client]` (covered in part 3).

#### Auth testing

Manual mock auth (test.rs):

```rust
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
```

With AuthClient:

```rust
let auth = token::AuthClient::new(&env, &contract_id);
auth.transfer(&alice, &bob, &300).authorize(&alice).invoke();
```

And for real crypto signatures (ed25519, secp256r1), the manual approach needs ~80 lines of XDR plumbing per test. With soroban-sdk-tools:

```rust
let kp = Keypair::random(&env);
auth.transfer(kp.address(), &bob, &300).sign(&kp).invoke();
```

Run both contracts to verify they behave identically:

```sh
cargo test -p manual-token
cargo test -p demo-token
```

---

### Part 1: storage (`contracts/demo-token/src/lib.rs`)

Open `lib.rs` and look at the storage definition on line 17:

```rust
#[contractstorage(auto_shorten = true)]
pub struct TokenStorage {
    admin: InstanceItem<Address>,
    balances: PersistentMap<Address, i128>,
    allowances: PersistentMap<(Address, Address), i128>,
    total_supply: InstanceItem<i128>,
}
```

This struct replaces the `DataKey` enum and all the `env.storage().persistent().get()`/`set()` calls you'd normally write by hand.

The macro generates:
- `TokenStorage::get_balances(env, &addr)` / `set_balances(env, &addr, &val)` -- static one-liners
- `TokenStorage::update_balances(env, &addr, |bal| ...)` -- atomic read-modify-write
- `TokenStorage::extend_balances_ttl(env, &addr, min, max)` -- TTL management
- `TokenStorage::new(env)` -- struct instance for when you need multiple fields at once

`auto_shorten = true` hashes the storage keys down to short prefixes. In practice that saves 30-40% on storage fees.

`mint` on line 62 is a good one to show on camera:

```rust
TokenStorage::update_balances(env, to, |bal| bal.unwrap_or(0) + amount);
TokenStorage::update_total_supply(env, |s| s.unwrap_or(0) + amount);
TokenStorage::extend_balances_ttl(env, to, 50, 100);
```

```sh
cargo test -p demo-token test_mint_and_balance
cargo test -p demo-token test_transfer
cargo test -p demo-token test_approve_and_transfer_from
```

### Part 2: error handling (`contracts/demo-token/src/lib.rs` line 29)

```rust
#[scerr]
pub enum TokenError {
    /// already initialized
    AlreadyInitialized,
    /// not initialized
    NotInitialized,
    /// insufficient balance
    InsufficientBalance,
    /// insufficient allowance
    InsufficientAllowance,
    /// amount must be positive
    InvalidAmount,
}
```

`#[scerr]` generates a `#[contracterror]` enum with sequential error codes (1, 2, 3...) and wires up the doc comments as descriptions. You don't assign the numbers yourself.

The rest of the contract is just normal Rust -- `?` operator, `Result<(), TokenError>` returns.

```sh
cargo test -p demo-token test_insufficient_balance_error
cargo test -p demo-token test_already_initialized_error
```

### Part 3: cross-contract error composition (`contracts/demo-vault/src/lib.rs`)

The vault imports the token WASM:

```rust
mod token {
    soroban_sdk_tools::contractimport!(
        file = "../../target/wasm32v1-none/release/demo_token.wasm"
    );
}
```

`contractimport!` is like the SDK's version but also generates the `ContractError` trait impls that `#[scerr]` needs for composition.

The vault's error enum pulls in token errors with `#[from_contract_client]`:

```rust
#[scerr]
pub enum VaultError {
    AlreadyInitialized,
    NotInitialized,
    InvalidAmount,
    InsufficientDeposit,

    #[from_contract_client]
    Token(token::TokenError),
}
```

In the deposit method (line 89), the `??` operator converts token errors into vault errors:

```rust
token::Client::new(env, &token_id)
    .try_transfer_from(&vault_addr, user, &vault_addr, &amount)??;
```

First `?` unwraps the SDK's `Result`, second `?` converts `TokenError` into `VaultError::Token(...)` via the generated `From` impl. The WASM spec comes out fully flattened, so TypeScript bindings see every error variant without gaps.

```sh
cargo test -p demo-vault test_deposit_and_withdraw
cargo test -p demo-vault test_deposit_exceeds_allowance
```

`test_deposit_exceeds_allowance` is worth calling out -- alice tries to deposit more than her allowance. The token contract returns `InsufficientAllowance`, and it propagates through the vault as `VaultError::Token(TokenError::InsufficientAllowance)`.

### Part 4: auth testing (`contracts/demo-token/src/test.rs`)

The test file imports the token WASM through `contractimport!`, which generates an `AuthClient` alongside the normal `Client`:

```rust
mod token {
    soroban_sdk_tools::contractimport!(
        file = "../../target/wasm32v1-none/release/demo_token.wasm"
    );
}
```

#### Mock auth with AuthClient (line 98)

```rust
let auth = token::AuthClient::new(&env, &contract_id);
auth.transfer(&alice, &bob, &300).authorize(&alice).invoke();
```

Builder pattern. Call the method, chain `.authorize()` with the address that should sign, call `.invoke()`. Replaces the manual `MockAuth` / `MockAuthInvoke` struct setup.

```sh
cargo test -p demo-token test_transfer_with_auth_client
```

#### Real ed25519 signatures (line 124)

```rust
let kp = Keypair::random(&env);
let alice = kp.address().clone();

// ...mint tokens to alice...

let auth = token::AuthClient::new(&env, &contract_id);
auth.transfer(&alice, &bob, &300).sign(&kp).invoke();
```

`Keypair::random` generates a real ed25519 keypair and registers a Stellar account on the test ledger. `.sign(&kp)` produces an actual cryptographic signature over the auth payload -- nothing is mocked here.

```sh
cargo test -p demo-token test_transfer_with_ed25519_signature
```

#### Real passkey signatures (line 148)

```rust
let kp = Secp256r1Keypair::random(&env);
let signer_addr = kp.address().clone();

// ...mint tokens to signer_addr...

let auth = token::AuthClient::new(&env, &contract_id);
auth.transfer(&signer_addr, &bob, &500).sign(&kp).invoke();
```

`Secp256r1Keypair::random` registers a custom account contract that verifies P-256 signatures (same curve as WebAuthn/passkeys). The test signs the payload for real and the custom account contract verifies it on-chain.

```sh
cargo test -p demo-token test_transfer_with_passkey_signature
```

### Run all tests

```sh
cargo test
```

You should see 17 tests pass:

```
running 6 tests                    (manual-token)
test test::test_already_initialized_error ... ok
test test::test_approve_and_transfer_from ... ok
test test::test_insufficient_balance_error ... ok
test test::test_mint_and_balance ... ok
test test::test_transfer ... ok
test test::test_transfer_with_manual_mock_auth ... ok

running 8 tests                    (demo-token)
test test::test_already_initialized_error ... ok
test test::test_approve_and_transfer_from ... ok
test test::test_insufficient_balance_error ... ok
test test::test_mint_and_balance ... ok
test test::test_transfer ... ok
test test::test_transfer_with_auth_client ... ok
test test::test_transfer_with_ed25519_signature ... ok
test test::test_transfer_with_passkey_signature ... ok

running 3 tests                    (demo-vault)
test test::test_deposit_and_withdraw ... ok
test test::test_deposit_exceeds_allowance ... ok
test test::test_withdraw_exceeds_deposit ... ok
```

## Project structure

```
contracts/
  manual-token/src/
    lib.rs     raw SDK version (DataKey enum, manual get/set, #[contracterror])
    test.rs    manual MockAuth struct setup
  demo-token/src/
    lib.rs     same contract with soroban-sdk-tools (#[contractstorage], #[scerr])
    test.rs    AuthClient with .authorize(), .sign() (ed25519, secp256r1)
  demo-vault/src/
    lib.rs     cross-contract error composition (#[from_contract_client], ??)
    test.rs    integration tests (deposit/withdraw, error propagation)
```

## Links

- Crate: https://crates.io/crates/soroban-sdk-tools
- Docs: https://docs.rs/soroban-sdk-tools
- Source: https://github.com/BlaineHeffron/soroban-sdk-tools
