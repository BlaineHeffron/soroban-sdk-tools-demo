# soroban-sdk-tools demo video script

~4.5 minutes at normal speaking pace. Screen shows `walkthrough.html` in a browser and scrolls straight down section by section.

---

[SCREEN: walkthrough.html top of page]

Hey, I'm Blaine. I made soroban-sdk-tools. It's a set of proc macros for Soroban contracts. Instead of just describing it, I wrote the same token contract two ways: one with the raw SDK, one with the macros.

[SCREEN: section 1, Storage recap]

Quick recap from tranche 1. With the raw SDK, you define a DataKey enum, one variant per storage field, and every method ends up building keys and doing get and set calls by hand.

[SCREEN: scroll to section 2, Storage example]

One example and then I'm moving on, because storage isn't the new part here. In mint, you read a balance, update it, write it back, then do the same thing again for total supply. It works. It's just repetitive.

[SCREEN: scroll to section 3, focus left and middle columns]

Errors are where it starts getting annoying. You write the contracterror attribute, the derive line, repr(u32), and then number every variant yourself. Add one later and you're either renumbering things or living with gaps.

[SCREEN: scroll to section 4, focus left column]

Then in tests, if you want targeted auth for one address instead of mock_all_auths, you build a MockAuth struct by hand. Contract address, function name, args converted to Val, sub_invokes. That's a lot of setup just to say "alice authorized this transfer."

And if you want real signature verification in tests, ed25519 or secp256r1, there's nothing built in. You end up writing custom account plumbing, building SorobanAuthorizationEntry structs, hashing the preimage, signing it, packing it into ScVal. Roughly 80 lines of XDR work per test.

[SCREEN: scroll slightly up if needed, keep sections 1 through 4 in frame as you compare left vs right]

OK. Same contract, with soroban-sdk-tools.

[SCREEN: section 1, focus right column]

With soroban-sdk-tools, the storage side is six lines. That's the whole schema.

auto_shorten hashes the storage keys into short prefixes. Saves about 30 to 40 percent on storage fees.

[SCREEN: section 2, focus right column]

Mint is the same logic, just less boilerplate. update_balances does read-modify-write in one call. update_total_supply does the same thing. That's the storage recap.

[SCREEN: section 3, focus middle column]

Now the part I actually want to focus on: errors. #[scerr] takes care of contracterror, derives, repr, and numbering. Codes are assigned sequentially. Doc comments become descriptions in the WASM spec. That matters once you start composing contracts.

[SCREEN: section 3, focus generated TypeScript column]

And because that lands in the spec, the generated TypeScript already has the full error map. You don't have to maintain some separate client-side list of codes.

[SCREEN: section 4, focus right column]

Auth testing also gets much cleaner. contractimport generates an AuthClient alongside the normal Client. You call the method, chain .authorize() with the signer, then .invoke(). One line instead of that whole MockAuth block.

[SCREEN: scroll to section 5, left column]

For real signatures, Keypair::random gives you an ed25519 keypair and registers a Stellar account on the test ledger. Then .sign(&kp) signs the auth payload for real. Nothing mocked.

[SCREEN: section 5, right column]

Passkeys work the same way. Secp256r1Keypair::random registers a custom account contract that does P-256 verification. A few lines instead of that 80-line XDR detour.

[SCREEN: scroll to section 6]

One more thing: cross-contract error composition. The vault imports the token WASM with contractimport, which generates the error types. Then the vault error enum uses #[from_contract_client] to pull in TokenError.

[SCREEN: section 6, focus left and middle columns]

In deposit, the vault calls token.try_transfer_from, and the double question mark does the conversion. The first ? unwraps the SDK Result. The second ? converts the token error into a vault error through the generated From impl. The spec comes out flattened, so TypeScript sees every variant cleanly.

[SCREEN: section 6, focus generated TypeScript column]

And you can see that directly in the generated TypeScript binding for the vault. Vault errors first, then imported token errors, plus the generic cross-contract abort cases.

[SCREEN: scroll to section 7]

Same deal for the token client. The methods are typed already, and the generated error map carries over the Rust docs.

[SCREEN: optionally cut to terminal with testnet links or `cargo test` output]

That’s it. 17 tests: 6 for the manual version, 8 for the tools version, 3 for the vault. All green.

[SCREEN: show the passing test output or testnet contract pages]

That's the library. cargo add soroban-sdk-tools, and for tests, --dev --features testutils. Links are below if you want to dig into the repo.
