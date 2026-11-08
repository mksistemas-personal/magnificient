-- Organization API owned schema.
CREATE TABLE IF NOT EXISTS organizations
(
    id               BIGINT PRIMARY KEY,
    name             VARCHAR(255) NOT NULL,
    document_value   VARCHAR(50)  NOT NULL,
    document_type    VARCHAR(30)  NOT NULL,
    document_country VARCHAR(10),
    deleted          BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by       VARCHAR(100) NOT NULL,
    updated_by       VARCHAR(100) NOT NULL,
    CONSTRAINT uq_organizations_document UNIQUE (document_type, document_value)
);

CREATE INDEX IF NOT EXISTS idx_organizations_name ON organizations (name);
CREATE INDEX IF NOT EXISTS idx_organizations_created_at ON organizations (created_at);

COMMENT ON TABLE organizations IS 'Organization registration and management';
COMMENT ON COLUMN organizations.id IS 'Unique TSID (64-bit integer)';
COMMENT ON COLUMN organizations.name IS 'Organization name or legal name';
COMMENT ON COLUMN organizations.document_value IS 'Raw or formatted identification document value';
COMMENT ON COLUMN organizations.document_type IS 'Identification document type';
COMMENT ON COLUMN organizations.document_country IS 'Country that issued the document';
COMMENT ON COLUMN organizations.deleted IS 'Indicates whether the organization was logically deleted';
COMMENT ON COLUMN organizations.created_at IS 'Creation timestamp in UTC';
COMMENT ON COLUMN organizations.updated_at IS 'Last update timestamp in UTC';
COMMENT ON COLUMN organizations.created_by IS 'Creator identifier';
COMMENT ON COLUMN organizations.updated_by IS 'Last updater identifier';

-- Link Organization to the Address entity created by the shared migration.
ALTER TABLE organizations
    ADD COLUMN IF NOT EXISTS address_id BIGINT;

DO
$$
    BEGIN
        IF NOT EXISTS (SELECT 1
                       FROM pg_constraint
                       WHERE conname = 'fk_organizations_address'
                         AND conrelid = 'organizations'::regclass) THEN
            ALTER TABLE organizations
                ADD CONSTRAINT fk_organizations_address
                    FOREIGN KEY (address_id) REFERENCES addresses (id)
                        ON DELETE SET NULL;
        END IF;
    END
$$;

CREATE INDEX IF NOT EXISTS idx_organizations_address_id ON organizations (address_id);
COMMENT ON COLUMN organizations.address_id IS 'Address linked to this organization';
