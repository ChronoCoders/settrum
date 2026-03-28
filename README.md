# Railnet

Settlement infrastructure for regulated financial institutions. Private Substrate blockchain with a REST API layer.

## Architecture

```
railnet-node/
  pallets/          # 5 custom Substrate pallets
  runtime/          # Runtime integrating all pallets
  node/             # Node binary (Aura + GRANDPA)
railnet-api/        # Actix Web REST API (PostgreSQL + Subxt)
railnet-web/        # Frontend (Astro)
```

## Pallets

| Pallet | Description |
|--------|-------------|
| `pallet-operators` | Operator registration, collateral tracking, status lifecycle (Active / Suspended / Terminated) |
| `pallet-asset-registry` | Multi-type asset registry (Fiat, Commodity, Security, InternalLedger) with supply tracking |
| `pallet-settlement-engine` | Settlement execution — Issue, Redeem, Transfer, Lock, Unlock |
| `pallet-settlement-proofs` | Proof submission and verification (Signature, Oracle, Multisig, ZeroKnowledge, Documentary) |
| `pallet-cross-settlement` | Multi-leg atomic cross-operator settlements with participant approval workflow |

## Stack

- Rust 1.75+ (stable)
- Polkadot SDK `polkadot-stable2512-3`
- Aura block authoring + GRANDPA finality (6-second blocks)
- Actix Web + PostgreSQL 15 + Subxt
- JWT (HS256) operator authentication

## Development

```bash
# Build pallets
cargo build --workspace

# Test (all 103 tests)
cargo test --workspace

# Lint (zero warnings enforced)
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Running Locally

```bash
cp .env.example .env
# Edit .env with your secrets

docker compose up
```

Services:
- Node RPC: `ws://localhost:9944`
- API: `http://localhost:8080`
- PostgreSQL: `localhost:5432`

## Status

**Phase 1 complete** — all 5 pallets implemented, 103 tests passing, zero warnings.

**Phase 2 in progress** — runtime integration, node binary, API implementation, database migrations.

---

Distributed Systems Labs
