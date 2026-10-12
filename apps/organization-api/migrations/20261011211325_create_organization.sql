-- Migration: create_organization
-- Criação da tabela de organizações baseada na entidade Organization

CREATE TABLE IF NOT EXISTS organizations (
    id BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    document_value VARCHAR(50) NOT NULL,
    document_type VARCHAR(30) NOT NULL,
    document_country VARCHAR(10),
    deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100) NOT NULL,
    updated_by VARCHAR(100) NOT NULL,
    CONSTRAINT uq_organizations_document UNIQUE (document_type, document_value)
);

CREATE INDEX IF NOT EXISTS idx_organizations_name ON organizations (name);
CREATE INDEX IF NOT EXISTS idx_organizations_created_at ON organizations (created_at);

COMMENT ON TABLE organizations IS 'Tabela de cadastro e gestão de organizações';
COMMENT ON COLUMN organizations.id IS 'Identificador único TSID (64-bit integer)';
COMMENT ON COLUMN organizations.name IS 'Nome ou razão social da organização';
COMMENT ON COLUMN organizations.document_value IS 'Valor bruto/formatado do documento de identificação';
COMMENT ON COLUMN organizations.document_type IS 'Tipo do documento (CNPJ, CPF, CUIT, CUIL, DNI, PASSPORT, etc.)';
COMMENT ON COLUMN organizations.document_country IS 'País emissor do documento (BR, AR, etc.)';
COMMENT ON COLUMN organizations.deleted IS 'Indica se a organização foi logicamente removida/desativada';
COMMENT ON COLUMN organizations.created_at IS 'Data e hora de criação do registro em UTC';
COMMENT ON COLUMN organizations.updated_at IS 'Data e hora da última atualização em UTC';
COMMENT ON COLUMN organizations.created_by IS 'Identificador/login do criador do registro';
COMMENT ON COLUMN organizations.updated_by IS 'Identificador/login do último atualizador do registro';
