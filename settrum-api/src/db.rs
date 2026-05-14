use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

// ─── Row types ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OperatorRow {
    pub id: i32,
    pub account: String,
    pub name: String,
    pub collateral: String,
    pub status: String,
    pub settlement_count: i64,
    pub registered_at: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AssetRow {
    pub id: i32,
    pub issuer: String,
    pub name: String,
    pub symbol: String,
    pub asset_type: String,
    pub decimals: i16,
    pub total_supply: String,
    pub settlement_rules: String,
    pub registered_at: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettlementRow {
    pub id: i32,
    pub operator_id: i32,
    pub asset_id: i32,
    pub operation: String,
    pub amount: String,
    pub from_account: String,
    pub to_account: String,
    pub reference: String,
    pub status: String,
    pub submitted_at: i32,
    pub finalized_at: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct BalanceRow {
    pub account: String,
    pub asset_id: i32,
    pub balance: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProofRow {
    pub id: i32,
    pub settlement_id: i32,
    pub proof_type: String,
    pub hash: String,
    pub submitter: String,
    pub data: String,
    pub status: String,
    pub submitted_at: i32,
    pub verified_at: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CrossSettlementRow {
    pub id: i32,
    pub initiator_id: i32,
    pub participants: Vec<i32>,
    pub legs: serde_json::Value,
    pub approvals: Vec<i32>,
    pub reference: String,
    pub status: String,
    pub created_at_block: i32,
    pub expires_at_block: i32,
    pub executed_at_block: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Query helpers ────────────────────────────────────────────────────────────

pub async fn get_operator_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<Option<OperatorRow>, sqlx::Error> {
    sqlx::query_as::<_, OperatorRow>("SELECT * FROM operators WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_operator_by_account(
    pool: &PgPool,
    account: &str,
) -> Result<Option<OperatorRow>, sqlx::Error> {
    sqlx::query_as::<_, OperatorRow>("SELECT * FROM operators WHERE account = $1")
        .bind(account)
        .fetch_optional(pool)
        .await
}

pub async fn list_operators(pool: &PgPool) -> Result<Vec<OperatorRow>, sqlx::Error> {
    sqlx::query_as::<_, OperatorRow>("SELECT * FROM operators ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn insert_operator(
    pool: &PgPool,
    id: i32,
    account: &str,
    name: &str,
    collateral: &str,
    registered_at: i32,
) -> Result<OperatorRow, sqlx::Error> {
    sqlx::query_as::<_, OperatorRow>(
        "INSERT INTO operators (id, account, name, collateral, status, settlement_count, registered_at)
         VALUES ($1, $2, $3, $4, 'Active', 0, $5)
         RETURNING *",
    )
    .bind(id)
    .bind(account)
    .bind(name)
    .bind(collateral)
    .bind(registered_at)
    .fetch_one(pool)
    .await
}

pub async fn update_operator_status(
    pool: &PgPool,
    id: i32,
    status: &str,
) -> Result<Option<OperatorRow>, sqlx::Error> {
    sqlx::query_as::<_, OperatorRow>(
        "UPDATE operators SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(status)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_asset_by_id(pool: &PgPool, id: i32) -> Result<Option<AssetRow>, sqlx::Error> {
    sqlx::query_as::<_, AssetRow>("SELECT * FROM assets WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_assets(pool: &PgPool) -> Result<Vec<AssetRow>, sqlx::Error> {
    sqlx::query_as::<_, AssetRow>("SELECT * FROM assets ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn insert_asset(
    pool: &PgPool,
    id: i32,
    issuer: &str,
    name: &str,
    symbol: &str,
    asset_type: &str,
    decimals: i16,
    total_supply: &str,
    settlement_rules: &str,
    registered_at: i32,
) -> Result<AssetRow, sqlx::Error> {
    sqlx::query_as::<_, AssetRow>(
        "INSERT INTO assets (id, issuer, name, symbol, asset_type, decimals, total_supply, settlement_rules, registered_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING *",
    )
    .bind(id)
    .bind(issuer)
    .bind(name)
    .bind(symbol)
    .bind(asset_type)
    .bind(decimals)
    .bind(total_supply)
    .bind(settlement_rules)
    .bind(registered_at)
    .fetch_one(pool)
    .await
}

pub async fn update_asset_supply(
    pool: &PgPool,
    id: i32,
    total_supply: &str,
) -> Result<Option<AssetRow>, sqlx::Error> {
    sqlx::query_as::<_, AssetRow>(
        "UPDATE assets SET total_supply = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(total_supply)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_settlement_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<Option<SettlementRow>, sqlx::Error> {
    sqlx::query_as::<_, SettlementRow>("SELECT * FROM settlements WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_settlements(pool: &PgPool) -> Result<Vec<SettlementRow>, sqlx::Error> {
    sqlx::query_as::<_, SettlementRow>("SELECT * FROM settlements ORDER BY id")
        .fetch_all(pool)
        .await
}

pub async fn insert_settlement(
    pool: &PgPool,
    id: i32,
    operator_id: i32,
    asset_id: i32,
    operation: &str,
    amount: &str,
    from_account: &str,
    to_account: &str,
    reference: &str,
    submitted_at: i32,
) -> Result<SettlementRow, sqlx::Error> {
    sqlx::query_as::<_, SettlementRow>(
        "INSERT INTO settlements
         (id, operator_id, asset_id, operation, amount, from_account, to_account, reference, status, submitted_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'Pending', $9)
         RETURNING *",
    )
    .bind(id)
    .bind(operator_id)
    .bind(asset_id)
    .bind(operation)
    .bind(amount)
    .bind(from_account)
    .bind(to_account)
    .bind(reference)
    .bind(submitted_at)
    .fetch_one(pool)
    .await
}

pub async fn finalize_settlement(
    pool: &PgPool,
    id: i32,
    finalized_at: i32,
) -> Result<Option<SettlementRow>, sqlx::Error> {
    sqlx::query_as::<_, SettlementRow>(
        "UPDATE settlements SET status = 'Finalized', finalized_at = $1, updated_at = NOW()
         WHERE id = $2 AND status = 'Pending'
         RETURNING *",
    )
    .bind(finalized_at)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_balance(
    pool: &PgPool,
    account: &str,
    asset_id: i32,
) -> Result<Option<BalanceRow>, sqlx::Error> {
    sqlx::query_as::<_, BalanceRow>(
        "SELECT * FROM account_balances WHERE account = $1 AND asset_id = $2",
    )
    .bind(account)
    .bind(asset_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_locked_balance(
    pool: &PgPool,
    account: &str,
    asset_id: i32,
) -> Result<Option<BalanceRow>, sqlx::Error> {
    sqlx::query_as::<_, BalanceRow>(
        "SELECT * FROM locked_balances WHERE account = $1 AND asset_id = $2",
    )
    .bind(account)
    .bind(asset_id)
    .fetch_optional(pool)
    .await
}

pub async fn get_proof_by_id(pool: &PgPool, id: i32) -> Result<Option<ProofRow>, sqlx::Error> {
    sqlx::query_as::<_, ProofRow>("SELECT * FROM proofs WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_proof(
    pool: &PgPool,
    id: i32,
    settlement_id: i32,
    proof_type: &str,
    hash: &str,
    submitter: &str,
    data: &str,
    submitted_at: i32,
) -> Result<ProofRow, sqlx::Error> {
    sqlx::query_as::<_, ProofRow>(
        "INSERT INTO proofs (id, settlement_id, proof_type, hash, submitter, data, status, submitted_at)
         VALUES ($1, $2, $3, $4, $5, $6, 'Pending', $7)
         RETURNING *",
    )
    .bind(id)
    .bind(settlement_id)
    .bind(proof_type)
    .bind(hash)
    .bind(submitter)
    .bind(data)
    .bind(submitted_at)
    .fetch_one(pool)
    .await
}

pub async fn verify_proof(
    pool: &PgPool,
    id: i32,
    verified_at: i32,
) -> Result<Option<ProofRow>, sqlx::Error> {
    sqlx::query_as::<_, ProofRow>(
        "UPDATE proofs SET status = 'Verified', verified_at = $1, updated_at = NOW()
         WHERE id = $2 AND status = 'Pending'
         RETURNING *",
    )
    .bind(verified_at)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_cross_settlement_by_id(
    pool: &PgPool,
    id: i32,
) -> Result<Option<CrossSettlementRow>, sqlx::Error> {
    sqlx::query_as::<_, CrossSettlementRow>("SELECT * FROM cross_settlements WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn insert_cross_settlement(
    pool: &PgPool,
    id: i32,
    initiator_id: i32,
    participants: &[i32],
    legs: &serde_json::Value,
    reference: &str,
    created_at_block: i32,
    expires_at_block: i32,
) -> Result<CrossSettlementRow, sqlx::Error> {
    sqlx::query_as::<_, CrossSettlementRow>(
        "INSERT INTO cross_settlements
         (id, initiator_id, participants, legs, approvals, reference, status, created_at_block, expires_at_block)
         VALUES ($1, $2, $3, $4, ARRAY[]::integer[], $5, 'Pending', $6, $7)
         RETURNING *",
    )
    .bind(id)
    .bind(initiator_id)
    .bind(participants)
    .bind(legs)
    .bind(reference)
    .bind(created_at_block)
    .bind(expires_at_block)
    .fetch_one(pool)
    .await
}

pub async fn approve_cross_settlement(
    pool: &PgPool,
    id: i32,
    operator_id: i32,
) -> Result<Option<CrossSettlementRow>, sqlx::Error> {
    sqlx::query_as::<_, CrossSettlementRow>(
        "UPDATE cross_settlements
         SET approvals = array_append(approvals, $1), updated_at = NOW()
         WHERE id = $2 AND status = 'Pending' AND NOT ($1 = ANY(approvals))
         RETURNING *",
    )
    .bind(operator_id)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn execute_cross_settlement(
    pool: &PgPool,
    id: i32,
    executed_at_block: i32,
) -> Result<Option<CrossSettlementRow>, sqlx::Error> {
    sqlx::query_as::<_, CrossSettlementRow>(
        "UPDATE cross_settlements
         SET status = 'Executed', executed_at_block = $1, updated_at = NOW()
         WHERE id = $2 AND status = 'Approved'
         RETURNING *",
    )
    .bind(executed_at_block)
    .bind(id)
    .fetch_optional(pool)
    .await
}
