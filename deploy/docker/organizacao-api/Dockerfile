FROM rust:1.94 AS builder

WORKDIR /workspace

# Arquivos do workspace
COPY Cargo.toml Cargo.lock ./

# Projetos e crates do workspace
COPY apps ./apps
COPY crates ./crates

# Compila somente a organization-api
RUN cargo build --release -p organizacao-api


FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /workspace/target/release/organizacao-api .

EXPOSE 8080

CMD ["./organizacao-api"]