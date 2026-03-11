# soroban-sdk-tools demo video script

~4.5 minutes at normal speaking pace. Screen shows code in an editor with a terminal panel.

---

[SCREEN: editor with manual-token/src/lib.rs open]

Hey, I'm Blaine. I made soroban-sdk-tools, which is a set of proc macros for Soroban contracts. Instead of just telling you what it does, I wrote the same token contract two ways: one with the raw SDK, one with the macros. Let me show you both.

[SCREEN: highlight DataKey enum, lines 14-21]

So here's the standard approach. You define a DataKey enum, one variant per storage field. Admin, Balance, Allowance, TotalSupply. And then every method that touches storage, you're constructing the key, calling env.storage().persistent().get(), unwrapping it, doing your thing, calling set() again.

[SCREEN: scroll to mint(), lines 57-78, highlight the key construction and get/set calls]

Look at mint. Four lines to read a balance, update it, write it back. Three more for total supply. You're cloning the address into the DataKey each time. It's fine, it works, you just end up writing it a lot.

[SCREEN: scroll to transfer(), lines 81-104]

Transfer. Same thing. Build the key, get, check, set. Build another key, get, set. You're copy-pasting the pattern and changing the field name.

[SCREEN: scroll up to the error enum, lines 27-36]

For errors you write the contracterror attribute, the derive line, repr(u32), and number each variant yourself. Insert one in the middle later and you're either re-numbering or leaving gaps.

[SCREEN: switch to manual-token/src/test.rs, scroll to test_transfer_with_manual_mock_auth, lines 90-119]

Tests. When you need targeted auth for a specific address instead of mock_all_auths, you build a MockAuth struct by hand. Contract address, function name as a string, args converted to Val, sub_invokes. That's a lot of setup just to say "alice authorized this transfer."

If you want actual signature verification in tests, ed25519 or secp256r1, there's nothing built in. You'd write a custom account contract, register it, build SorobanAuthorizationEntry structs, hash the preimage, sign it, pack it into ScVal. About 80 lines of XDR plumbing. Per test.

[SCREEN: split view, manual-token/src/lib.rs left, demo-token/src/lib.rs right]

OK. Same contract, with soroban-sdk-tools.

[SCREEN: highlight TokenStorage struct on the right, lines 17-23]

Six lines. That's the whole storage schema. admin is an InstanceItem, balances is a PersistentMap, allowances is a PersistentMap with a tuple key, total_supply is an InstanceItem. The macro generates get, set, update, remove, has, and extend_ttl for every field. You can use static one-liners like TokenStorage::get_balances, or grab the struct with TokenStorage::new when you need a few fields together.

auto_shorten hashes the storage keys into short prefixes. Saves about 30 to 40 percent on storage fees.

[SCREEN: highlight mint() on the right, lines 62-74]

Mint. update_balances does read-modify-write in one call. update_total_supply, same. extend_balances_ttl for the TTL. Compare it to the left side, same logic, no key construction.

[SCREEN: highlight the #[scerr] enum on the right, lines 29-41]

Errors: #[scerr] handles contracterror, the derives, repr, the numbering. Codes are assigned sequentially. Doc comments become descriptions in the WASM spec. I'll show you why that matters in a sec.

[SCREEN: switch to demo-token/src/test.rs, show test_transfer_with_auth_client, lines 98-118]

Auth testing. contractimport generates an AuthClient alongside the normal Client. Builder pattern: call the method, chain .authorize() with the signing address, .invoke(). One line instead of that whole MockAuth block.

[SCREEN: scroll to test_transfer_with_ed25519_signature, lines 124-146]

Real signatures. Keypair::random generates an ed25519 keypair and registers a Stellar account on the test ledger. .sign(&kp) signs the auth payload for real. Nothing mocked.

[SCREEN: scroll to test_transfer_with_passkey_signature, lines 148-173]

Passkeys work the same way. Secp256r1Keypair::random registers a custom account contract that does P-256 verification. Three lines instead of that 80-line XDR thing I mentioned.

[SCREEN: switch to demo-vault/src/lib.rs]

One more. Cross-contract error composition. The vault imports the token WASM with contractimport, which generates the error types. The vault's error enum uses #[from_contract_client] to pull in TokenError.

[SCREEN: highlight the VaultError enum, lines 36-50]

In the deposit method, the vault calls the token's try_transfer_from, and the double question mark handles the conversion. First ? unwraps the SDK Result, second ? converts the token error into a vault error through the generated From impl. The WASM spec comes out flattened so your TypeScript bindings see every variant without gaps.

[SCREEN: switch to bindings/demo-vault/src/index.ts, highlight VaultError]

And here’s the generated TypeScript binding for the vault. You can see the flattened error surface directly: the vault errors first, then the imported token errors, plus the generic cross-contract abort cases.

[SCREEN: switch to bindings/demo-token/src/index.ts, highlight Client interface]

Same deal for the token client. Every contract method is already typed, and the generated error map carries over the doc comments from the Rust enum.

[SCREEN: terminal, run cargo test]

Let's run it. 17 tests: 6 for the manual version, 8 for the tools version, 3 for the vault. All green.

[SCREEN: show the passing test output]

That's the library. cargo add soroban-sdk-tools, and for tests, --dev --features testutils. Links below, the demo repo is there too if you want to poke around.
