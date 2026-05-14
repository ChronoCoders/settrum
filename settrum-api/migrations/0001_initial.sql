-- Settrum initial schema

CREATE TABLE operators (
    id              INTEGER PRIMARY KEY,
    account         TEXT        NOT NULL UNIQUE,
    name            TEXT        NOT NULL,
    collateral      TEXT        NOT NULL,
    status          TEXT        NOT NULL DEFAULT 'Active'
                    CHECK (status IN ('Active', 'Suspended', 'Terminated')),
    settlement_count BIGINT     NOT NULL DEFAULT 0,
    registered_at   INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE assets (
    id               INTEGER PRIMARY KEY,
    issuer           TEXT        NOT NULL,
    name             TEXT        NOT NULL,
    symbol           TEXT        NOT NULL,
    asset_type       TEXT        NOT NULL
                     CHECK (asset_type IN ('Fiat', 'Commodity', 'Security', 'InternalLedger')),
    decimals         SMALLINT    NOT NULL CHECK (decimals BETWEEN 0 AND 18),
    total_supply     TEXT        NOT NULL DEFAULT '0',
    settlement_rules TEXT        NOT NULL DEFAULT '',
    registered_at    INTEGER     NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE settlements (
    id            INTEGER PRIMARY KEY,
    operator_id   INTEGER     NOT NULL REFERENCES operators (id),
    asset_id      INTEGER     NOT NULL REFERENCES assets (id),
    operation     TEXT        NOT NULL
                  CHECK (operation IN ('Issue', 'Redeem', 'Transfer', 'Lock', 'Unlock')),
    amount        TEXT        NOT NULL,
    from_account  TEXT        NOT NULL,
    to_account    TEXT        NOT NULL,
    reference     TEXT        NOT NULL DEFAULT '',
    status        TEXT        NOT NULL DEFAULT 'Pending'
                  CHECK (status IN ('Pending', 'Finalized', 'Disputed')),
    submitted_at  INTEGER     NOT NULL DEFAULT 0,
    finalized_at  INTEGER,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX settlements_operator_idx ON settlements (operator_id);
CREATE INDEX settlements_asset_idx    ON settlements (asset_id);
CREATE INDEX settlements_status_idx   ON settlements (status);

CREATE TABLE account_balances (
    account    TEXT        NOT NULL,
    asset_id   INTEGER     NOT NULL REFERENCES assets (id),
    balance    TEXT        NOT NULL DEFAULT '0',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (account, asset_id)
);

CREATE TABLE locked_balances (
    account    TEXT        NOT NULL,
    asset_id   INTEGER     NOT NULL REFERENCES assets (id),
    balance    TEXT        NOT NULL DEFAULT '0',
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (account, asset_id)
);

CREATE TABLE proofs (
    id            INTEGER PRIMARY KEY,
    settlement_id INTEGER     NOT NULL REFERENCES settlements (id),
    proof_type    TEXT        NOT NULL
                  CHECK (proof_type IN ('Signature', 'Oracle', 'Multisig', 'ZeroKnowledge', 'Documentary')),
    hash          TEXT        NOT NULL UNIQUE,
    submitter     TEXT        NOT NULL,
    data          TEXT        NOT NULL DEFAULT '',
    status        TEXT        NOT NULL DEFAULT 'Pending'
                  CHECK (status IN ('Pending', 'Verified', 'Revoked')),
    submitted_at  INTEGER     NOT NULL DEFAULT 0,
    verified_at   INTEGER,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX proofs_settlement_idx ON proofs (settlement_id);

CREATE TABLE cross_settlements (
    id               INTEGER PRIMARY KEY,
    initiator_id     INTEGER     NOT NULL REFERENCES operators (id),
    participants     INTEGER[]   NOT NULL DEFAULT '{}',
    legs             JSONB       NOT NULL DEFAULT '[]',
    approvals        INTEGER[]   NOT NULL DEFAULT '{}',
    reference        TEXT        NOT NULL DEFAULT '',
    status           TEXT        NOT NULL DEFAULT 'Pending'
                     CHECK (status IN ('Pending', 'Approved', 'Executed', 'Cancelled')),
    created_at_block INTEGER     NOT NULL DEFAULT 0,
    expires_at_block INTEGER     NOT NULL DEFAULT 0,
    executed_at_block INTEGER,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX cross_settlements_initiator_idx ON cross_settlements (initiator_id);
CREATE INDEX cross_settlements_status_idx    ON cross_settlements (status);
