ICP Payments (Non-Custodial, ICRC-2, MVP)

Rust-based payments canister skeleton implementing PaymentIntent flow for ICRC-2 assets (ckUSDC/ckBTC planned). MVP supports single full capture only.

Features (MVP)
- PaymentIntent: create → capture (single, full) → release/refund (stubs to be added)
- Ledger registry for multi-asset setup
- Deterministic escrow subaccount derivation
- Candid interface for SDK integration

Decisions
- Language: Rust
- Capture: single full capture only (no partial/multiple in MVP)
- Fees: Platform fee 0; network fees borne by user assets. Capture fee from payer account; release fee from escrow (reduces merchant net).
- Webhook/Registry: out-of-scope for MVP; focus on events + SDK verify stub later
- Local testing: PocketIC with ICRC-2 mock ledgers (ckUSDC/ckBTC analogues)

Layout
- `canisters/payments`: Rust canister crate
- `dfx.json`: workspace config

Build/Deploy (local)
- Requires: `rustup`, `wasm32-unknown-unknown` target, `dfx`
- Build: `cargo build --target wasm32-unknown-unknown --package payments --release`
- Deploy: `dfx start --clean` then `dfx deploy payments`

Next
- Implement `release` and `refund` with balance checks
- Event log with certified data and pagination
- TypeScript SDK stubs for `createPaymentIntent` and `capture`
- PocketIC tests exercising approve/transfer_from with mock ledger

