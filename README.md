# Meu Sistema

Estrutura de microsserviços e bibliotecas compartilhadas em Rust.

## Estrutura do Projeto

- `apps/`: Aplicações e APIs executáveis
  - `organization-api`: API de Organização
  - `client-api`: API de Cliente
  - `producer-api`: API de Produtor
- `crates/`: Bibliotecas compartilhadas
  - `shared`: Entidades, tipos e erros compartilhados
  - `database`: Módulo de acesso ao banco de dados
  - `messaging`: Módulo de mensageria
- `deploy/`: Arquivos de deploy
  - `docker/`: Configurações e Dockerfiles
  - `kubernetes/`: Manifestos Kubernetes
- `docs/`: Documentação
  - `architecture/`: Documentação de arquitetura
  - `api/`: Documentação e especificações de APIs
