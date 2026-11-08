CREATE TABLE IF NOT EXISTS addresses (
    id BIGINT PRIMARY KEY,
    street VARCHAR(255) NOT NULL,
    number VARCHAR(50) NOT NULL,
    complement VARCHAR(255),
    neighborhood VARCHAR(150),
    city VARCHAR(150) NOT NULL,
    state VARCHAR(100),
    state_code VARCHAR(10),
    postal_code VARCHAR(30) NOT NULL,
    country VARCHAR(100),
    country_code VARCHAR(10),
    reference VARCHAR(255),
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL DEFAULT 'system',
    updated_by VARCHAR(100) NOT NULL DEFAULT 'system'
);

CREATE INDEX IF NOT EXISTS idx_addresses_postal_code ON addresses (postal_code);
CREATE INDEX IF NOT EXISTS idx_addresses_city ON addresses (city);
CREATE INDEX IF NOT EXISTS idx_addresses_country_code ON addresses (country_code);

COMMENT ON TABLE addresses IS 'Address records shared by application entities';
COMMENT ON COLUMN addresses.id IS 'Unique TSID (64-bit integer)';
COMMENT ON COLUMN addresses.street IS 'Street name';
COMMENT ON COLUMN addresses.number IS 'Street number or S/N';
COMMENT ON COLUMN addresses.complement IS 'Address complement';
COMMENT ON COLUMN addresses.neighborhood IS 'Neighborhood or district';
COMMENT ON COLUMN addresses.city IS 'City or municipality';
COMMENT ON COLUMN addresses.state IS 'State or province name';
COMMENT ON COLUMN addresses.state_code IS 'State or province code';
COMMENT ON COLUMN addresses.postal_code IS 'Formatted postal code';
COMMENT ON COLUMN addresses.country IS 'Country name';
COMMENT ON COLUMN addresses.country_code IS 'ISO country code';
COMMENT ON COLUMN addresses.reference IS 'Address reference point';
COMMENT ON COLUMN addresses.deleted IS 'Indicates whether the address was logically deleted';
COMMENT ON COLUMN addresses.created_at IS 'Creation timestamp in UTC';
COMMENT ON COLUMN addresses.updated_at IS 'Last update timestamp in UTC';
COMMENT ON COLUMN addresses.created_by IS 'Creator identifier';
COMMENT ON COLUMN addresses.updated_by IS 'Last updater identifier';
