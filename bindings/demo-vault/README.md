# demo-vault bindings

Generated from `target/wasm32v1-none/release/demo_vault.wasm` with:

```sh
stellar contract bindings typescript \
  --wasm target/wasm32v1-none/release/demo_vault.wasm \
  --output-dir bindings/demo-vault
```

The generated surface you probably want to show on screen is [src/index.ts](src/index.ts).
