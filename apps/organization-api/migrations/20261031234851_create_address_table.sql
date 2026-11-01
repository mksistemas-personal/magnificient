-- Migration: create_address_table
-- Criação da tabela de endereços e adição de chave estrangeira em organizations

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

-- Adição da coluna address_id e foreign key na tabela organizations
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS address_id BIGINT;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_organizations_address'
    ) THEN
        ALTER TABLE organizations
            ADD CONSTRAINT fk_organizations_address
            FOREIGN KEY (address_id) REFERENCES addresses (id)
            ON DELETE SET NULL;
    END IF;
END $$;

CREATE INDEX IF NOT EXISTS idx_organizations_address_id ON organizations (address_id);

COMMENT ON TABLE addresses IS 'Tabela de cadastro e gestão de endereços';
COMMENT ON COLUMN addresses.id IS 'Identificador único TSID (64-bit integer)';
COMMENT ON COLUMN addresses.street IS 'Logradouro (rua, avenida, etc.)';
COMMENT ON COLUMN addresses.number IS 'Número do endereço ou S/N';
COMMENT ON COLUMN addresses.complement IS 'Complemento do endereço (apartamento, sala, etc.)';
COMMENT ON COLUMN addresses.neighborhood IS 'Bairro ou distrito';
COMMENT ON COLUMN addresses.city IS 'Cidade ou município';
COMMENT ON COLUMN addresses.state IS 'Nome completo do estado ou província';
COMMENT ON COLUMN addresses.state_code IS 'Sigla ou código do estado/província (ex: SP, IL, C)';
COMMENT ON COLUMN addresses.postal_code IS 'Código postal ou CEP formatado/sanitizado';
COMMENT ON COLUMN addresses.country IS 'Nome do país';
COMMENT ON COLUMN addresses.country_code IS 'Código do país no formato ISO (ex: BR, AR, US)';
COMMENT ON COLUMN addresses.reference IS 'Ponto de referência do endereço';
COMMENT ON COLUMN addresses.deleted IS 'Indica se o endereço foi logicamente removido/desativado';
COMMENT ON COLUMN addresses.created_at IS 'Data e hora de criação do registro em UTC';
COMMENT ON COLUMN addresses.updated_at IS 'Data e hora da última atualização em UTC';
COMMENT ON COLUMN addresses.created_by IS 'Identificador/login do criador do registro';
COMMENT ON COLUMN addresses.updated_by IS 'Identificador/login do último atualizador do registro';
COMMENT ON COLUMN organizations.address_id IS 'Chave estrangeira referenciando o endereço principal da organização';
