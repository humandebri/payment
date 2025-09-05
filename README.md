ICP Payments (Non-Custodial, ICRC-2, MVP)

Rust-based payments canister skeleton implementing PaymentIntent flow for ICRC-2 assets (ckUSDC/ckBTC planned). MVP supports single full capture only.

Features (MVP)
- PaymentIntent: create → capture (single, full) → release/refund (stubs to be added)
- Ledger registry for multi-asset setup
- Deterministic escrow subaccount derivation
- Candid interface for SDK integration
- Basic event log and lazy expiry; merchant-only auth on capture/release/refund

Decisions
- Language: Rust
- Capture: single full capture only (no partial/multiple in MVP)
- Fees: Platform fee 0; network fees borne by user assets. Capture fee from payer account; release fee from escrow (reduces merchant net).
- Webhook/Registry: out-of-scope for MVP; focus on events + SDK verify stub later
- Local testing: PocketIC with ICRC-2 mock ledgers (ckUSDC/ckBTC analogues)

Layout
- `canisters/payments`: Rust canister crate
- `dfx.json`: workspace config
- `apps/dashboard`: Next.js + Tailwind + minimal shadcn-style UI

Build/Deploy (local)
- Requires: `rustup`, `wasm32-unknown-unknown` target, `dfx`
- Build: `cargo build --target wasm32-unknown-unknown --package payments --release`
- Deploy: `dfx start --clean` then `dfx deploy payments`

Dashboard (local)
- Install deps: `cd apps/dashboard && npm install`
- Set env: create `apps/dashboard/.env.local` with
  - `NEXT_PUBLIC_DFX_NETWORK=local`
  - `PAYMENTS_CANISTER_ID=$(dfx canister id payments)`
- Run: `npm run dev` (http://localhost:3000)

Next
- Add certified event log root + pagination
- TypeScript SDK stubs for `createPaymentIntent`/`capture`/`release`/`refund`
- PocketIC E2E tests with a mock ICRC-2 ledger
