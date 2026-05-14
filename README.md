# Settrum

Private settlement infrastructure for regulated financial institutions. Built on a permissioned Substrate blockchain with a REST API layer for institutional integrations.

## Overview

Settrum provides a complete settlement layer for institutions that require a private, auditable, and deterministic record of asset movements. Operators register on-chain with collateral, define assets, and submit settlements that are finalized and proven through a structured lifecycle. Cross-operator atomic settlements are supported natively.

All settlement logic executes in a Byzantine fault-tolerant blockchain with 6-second finality. The REST API provides a familiar HTTP interface for integrating with existing institutional systems.

## Architecture

```
┌─────────────────────────────────────────────┐
│               settrum-api                   │
│         Actix Web · JWT · PostgreSQL        │
└────────────────────┬────────────────────────┘
                     │ WebSocket RPC
┌────────────────────▼────────────────────────┐
│              settrum-node                   │
│  ┌──────────────────────────────────────┐   │
│  │             Runtime                  │   │
│  │  operators · asset-registry          │   │
│  │  settlement-engine · proofs          │   │
│  │  cross-settlement                    │   │
│  └──────────────────────────────────────┘   │
│  Aura block authoring · GRANDPA finality    │
└─────────────────────────────────────────────┘
```

## Pallets

| Pallet | Responsibility |
|---|---|
| `pallet-operators` | Operator registration, collateral management, status lifecycle (Active / Suspended / Terminated) |
| `pallet-asset-registry` | Multi-type asset registry (Fiat, Commodity, Security, InternalLedger) with supply tracking per issuer |
| `pallet-settlement-engine` | Settlement execution — Issue, Redeem, Transfer, Lock, Unlock — with balance and locked-balance accounting |
| `pallet-settlement-proofs` | Proof submission and verification across five proof types: Signature, Oracle, Multisig, ZeroKnowledge, Documentary |
| `pallet-cross-settlement` | Multi-leg atomic cross-operator settlements with participant approval workflow and expiry |

## API

The REST API runs on `/api/v1`. All write endpoints require a JWT obtained via `/auth/login`.

| Resource | Endpoints |
|---|---|
| Auth | `POST /auth/login` |
| Operators | `POST /operators` · `GET /operators` · `GET /operators/{id}` · `GET /operators/me` · `PUT /operators/{id}/status` |
| Assets | `POST /assets` · `GET /assets` · `GET /assets/{id}` · `PUT /assets/{id}/supply` |
| Settlements | `POST /settlements` · `GET /settlements` · `GET /settlements/{id}` · `POST /settlements/{id}/finalize` |
| Balances | `GET /balances/{asset_id}/{account_id}` · `GET /balances/locked/{asset_id}/{account_id}` |
| Proofs | `POST /proofs` · `GET /proofs/{id}` · `PUT /proofs/{id}/verify` |
| Cross-Settlements | `POST /cross-settlements` · `GET /cross-settlements/{id}` · `POST /cross-settlements/{id}/approve` · `POST /cross-settlements/{id}/execute` |
| Health | `GET /health` · `GET /status` |

## Stack

| Layer | Technology |
|---|---|
| Blockchain | Polkadot SDK `polkadot-stable2512-3` |
| Consensus | Aura (block authoring) + GRANDPA (finality) · 6-second blocks |
| API server | Rust · Actix Web 4 |
| Database | PostgreSQL 15 |
| Auth | JWT HS256 per operator |
| Build | Rust 1.75+ stable · single binary per component |

## Requirements

- Rust 1.75+ (`rustup target add wasm32-unknown-unknown`)
- PostgreSQL 15+
- Docker and Docker Compose (for local deployment)

## Running Locally

```bash
cp .env.example .env
# Fill in DATABASE_URL, JWT_SECRET, ADMIN_API_KEY

docker compose up
```

| Service | Address |
|---|---|
| Node RPC | `ws://localhost:9944` |
| API | `http://localhost:8080` |
| PostgreSQL | `localhost:5432` |

## Configuration

All API configuration is via environment variables. See `.env.example` for the full list. Required variables:

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Signing secret for operator JWTs (min 64 chars) |
| `ADMIN_API_KEY` | Admin API key for privileged operations (min 64 chars) |

## Building from Source

```bash
# Build all components
cargo build --release --workspace

# Run pallet tests
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Node binary
./target/release/settrum-node --dev --tmp

# API (requires running node and PostgreSQL)
./target/release/settrum-api
```

## Production Deployment

See `docker-compose.prod.yml` for a reference production stack including validator nodes, the API server, Nginx TLS termination, Prometheus, and Grafana.

Validator keys must be generated and injected before starting production nodes:

```bash
./settrum-node key generate --scheme sr25519
./settrum-node key insert --key-type aura --scheme sr25519 ...
./settrum-node key insert --key-type gran --scheme ed25519 ...
```

## Security

- All settlement state is on-chain and cryptographically finalized before the API reflects it
- Operator authentication via JWT; administrative operations require a separate API key
- Rate limiting enforced per operator
- Database queries use parameterized statements throughout
- No unsafe Rust (`#![forbid(unsafe_code)]` in every crate)

