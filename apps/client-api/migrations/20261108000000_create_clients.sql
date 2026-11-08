CREATE TABLE IF NOT EXISTS clients (
    id BIGINT PRIMARY KEY,
    organization_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    nickname VARCHAR(120),
    -- Managed by purchase-related use cases; the CRUD leaves these columns untouched.
    purchase_rate DOUBLE PRECISION NULL,
    last_purchase_at TIMESTAMPTZ NULL,
    second_address_id BIGINT,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL,
    updated_by VARCHAR(100) NOT NULL,
    CONSTRAINT fk_clients_organization
        FOREIGN KEY (organization_id) REFERENCES organizations (id),
    CONSTRAINT fk_clients_second_address
        FOREIGN KEY (second_address_id) REFERENCES addresses (id)
);

CREATE INDEX IF NOT EXISTS idx_clients_organization_id ON clients (organization_id);
CREATE INDEX IF NOT EXISTS idx_clients_name ON clients (name);
CREATE INDEX IF NOT EXISTS idx_clients_last_purchase_at ON clients (last_purchase_at);
