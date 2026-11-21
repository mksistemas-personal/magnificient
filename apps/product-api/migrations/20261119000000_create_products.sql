CREATE TABLE IF NOT EXISTS products (
    id BIGINT PRIMARY KEY,
    organization_id BIGINT NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sku VARCHAR(120),
    price NUMERIC(15,2),
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL,
    updated_by VARCHAR(100) NOT NULL,
    CONSTRAINT fk_products_organization
        FOREIGN KEY (organization_id) REFERENCES organizations (id)
);

CREATE INDEX IF NOT EXISTS idx_products_organization_id ON products (organization_id);
CREATE INDEX IF NOT EXISTS idx_products_name ON products (name);
CREATE INDEX IF NOT EXISTS idx_products_sku ON products (sku);
