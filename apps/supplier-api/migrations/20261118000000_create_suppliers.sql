CREATE TABLE IF NOT EXISTS suppliers (
    id BIGINT PRIMARY KEY,
    organization_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    nickname VARCHAR(120),
    second_address_id BIGINT,
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL,
    updated_by VARCHAR(100) NOT NULL,
    CONSTRAINT fk_suppliers_organization
        FOREIGN KEY (organization_id) REFERENCES organizations (id),
    CONSTRAINT fk_suppliers_second_address
        FOREIGN KEY (second_address_id) REFERENCES addresses (id)
);

CREATE INDEX IF NOT EXISTS idx_suppliers_organization_id ON suppliers (organization_id);
CREATE INDEX IF NOT EXISTS idx_suppliers_name ON suppliers (name);
