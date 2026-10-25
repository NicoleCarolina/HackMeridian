![Stellar](https://img.shields.io/badge/Stellar-black?style=for-the-badge)
![Network](https://img.shields.io/badge/testnet-LIVE-00e6aa?style=for-the-badge)
![Built with](https://img.shields.io/badge/built%20with-Soroban-000000?style=for-the-badge)
![Tests](https://img.shields.io/badge/tests-9%2F9%20passing-brightgreen?style=for-the-badge)
![Audit](https://img.shields.io/badge/audit-pending-ffb400?style=for-the-badge)

# DCA Platform · Stellar

> **Dollar-cost averaging, non-custodial, on Stellar Testnet.**
> Deposit once. Swap on a schedule. 0.5 % per execution to the treasury, hard-capped at 2 %. Cancel anytime.

---

## 🚨 Live on Testnet

| | |
|---|---|
| **Contract ID** | [`CCCT7ERFARCNHA2TSAL2LQLLYUTPICWHSQL7KQBC43AHUORAEHJACQHS`](https://stellar.expert/explorer/testnet/contract/CCCT7ERFARCNHA2TSAL2LQLLYUTPICWHSQL7KQBC43AHUORAEHJACQHS) |
| **Deploy tx** | [`58225b3f…f18b71`](https://stellar.expert/explorer/testnet/tx/58225b3f41dd07a013669b0f3274cb97a3b55b44be16257f1900373e50f18b71) |
| **First real swap** (1 XLM → 0.167 USDC) | [`c5fe1cab…63c233`](https://stellar.expert/explorer/testnet/tx/c5fe1cab011827bcd4bfe25f09e097282db52b44bb64aeda0e8fbeb32163c233) |
| **Soroswap pool (USDC/XLM)** | [`CDVAIOYH…FKDB`](https://stellar.expert/explorer/testnet/contract/CDVAIOYHCD4RUSLQNVFI7RIZBFT2JZMJWM4RTOLQZQXL4QAVXU5RFKDB) · reserves 36.9k USDC + 217k XLM |
| **Live product preview** | https://raw.githack.com/NicoleCarolina/HackMeridian/main/demo.html |

---

## What it does

```
create_plan  → user deposits amount × num_buys (escrowed in the contract)
execute      → anyone (bot / keeper) triggers it when due:
               fee → treasury, remainder → router → user receives the swap
cancel       → refunds the unexecuted balance (always allowed, even if paused)
admin        → set_fee_bps (≤ 200), set_treasury, set_router, set_paused
```

Contract: [`contracts/dca-platform/src/lib.rs`](contracts/dca-platform/src/lib.rs). Swaps go through [Soroswap](https://soroswap.finance)'s AMM router (interface declared in [`router.rs`](contracts/dca-platform/src/router.rs), verified against upstream source). Tests use a mocked router.

---

## 🧪 Try it yourself (testnet)

Need the Stellar CLI (`cargo install --locked stellar-cli` or `brew install stellar-cli`).

```bash
# 1. Fund a key
stellar keys generate me --network testnet --fund

# 2. Create a DCA plan — 1 XLM → USDC, every 60 s, 3 buys
stellar contract invoke \
  --id CCCT7ERFARCNHA2TSAL2LQLLYUTPICWHSQL7KQBC43AHUORAEHJACQHS \
  --network testnet --source-account me --send=yes \
  -- create_plan \
    --owner me \
    --token_in  CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
    --token_out CB3TLW74NBIOT3BUWOZ3TUM6RFDF6A4GVIRUQRQZABG5KPOUL4JJOV2F \
    --amount_per_buy 10000000 \
    --interval 60 \
    --num_buys 3 \
    --slippage_bps 100
# → returns your plan_id

# 3. Trigger a buy immediately (first execution is always due at creation time)
stellar contract invoke \
  --id CCCT7ERFARCNHA2TSAL2LQLLYUTPICWHSQL7KQBC43AHUORAEHJACQHS \
  --network testnet --source-account me --send=yes \
  -- execute --plan_id <YOUR_PLAN_ID>
# → settles the swap through Soroswap and returns the USDC amount received

# 4. (Optional) Refund the remainder
stellar contract invoke \
  --id CCCT7ERFARCNHA2TSAL2LQLLYUTPICWHSQL7KQBC43AHUORAEHJACQHS \
  --network testnet --source-account me --send=yes \
  -- cancel --plan_id <YOUR_PLAN_ID>
```

**Reversing the direction (USDC → XLM):** you'll need Soroswap's testnet USDC in your wallet. Grab some via Soroswap's [testnet faucet](https://faucet.soroswap.finance), then swap `token_in`/`token_out` above.

---

## 🏗️ How it's built

```
contract  →  Rust + Soroban SDK 26 (12.7 KB optimized WASM)
router    →  Soroswap AMM router (verified against soroswap/core upstream)
frontend  →  React + Vite + TypeScript (Scaffold Stellar client generation)
keeper    →  bash script, polls plans and calls `execute` when due
network   →  Stellar Testnet (room-ready for mainnet after audit)
```

### Key design decisions

- **Non-custodial.** User signs `create_plan` once; after that anyone can call `execute`. The contract itself never gains discretion over user funds — the only paths out are "to the pool via Soroswap" or "back to the owner via `cancel`".
- **Checks → effects → interactions.** Plan state is mutated before external calls, so a failed router call cannot replay the same slot.
- **Fee hard cap at 2 % in the constructor.** The admin cannot grief users by raising it; the enforcement lives in bytecode.
- **Soroswap pre-authorization.** Because the router pulls the input token from `to` (the DCA contract), we pre-authorize that nested `transfer` with `env.authorize_as_current_contract(...)`. Without this, `execute` fails with `Auth/InvalidAction`.
- **Slippage guard.** Caller sets `slippage_bps` (max 1 000 = 10 %). On execution the contract takes the router's quote and applies the guard — if the real settlement comes back worse, the whole tx reverts (`SlippageExceeded`).

---

## 📦 Repository layout

```
hack-meridian/
├── contracts/
│   └── dca-platform/           ← the DCA contract (Rust + Soroban)
├── app/                        ← React + Vite frontend
│   ├── .env.example            ← fill PUBLIC_USDC_CONTRACT and PUBLIC_XLM_CONTRACT
│   └── src/components/DcaDashboard.tsx
├── scripts/
│   └── keeper.sh               ← poor-man's keeper (polls + calls execute)
├── demo.html                   ← interactive product preview (works offline)
├── environments.toml           ← Scaffold Stellar environment config
└── e2e/tests/smoke.spec.ts     ← Playwright smoke test
```

---

## 🧰 Local dev

```bash
npm install
cp app/.env.example app/.env     # already pre-filled with Soroswap testnet addresses
cargo test -p dca-platform       # 9/9 passing
stellar contract build           # builds the WASM
npm start                        # frontend + Scaffold Stellar client gen
```

Keeper (runs every due plan):

```bash
DCA_ID=CCCT7ERFARCNHA2TSAL2LQLLYUTPICWHSQL7KQBC43AHUORAEHJACQHS ./scripts/keeper.sh
```

---

## 🧭 What's next

- **External audit** before mainnet. The contract is small (<13 KB) but it custodies funds.
- **Reflector oracle guard** so the slippage check doesn't rely on a quote taken in the same tx (sandwich-resistant).
- **Multi-asset routing** beyond USDC/XLM — any Soroswap pair.
- **Keeper-as-a-service** with on-chain incentives, so users don't need to run their own.

---

## 🤝 Credits

Built for **HackMeridian** on top of [Scaffold Stellar](https://github.com/stellar/stellar-scaffold), with [Soroswap](https://soroswap.finance) as the swap backend.

[`Nicole Fucili`](https://github.com/NicoleCarolina) · Alex Serra · 2026
