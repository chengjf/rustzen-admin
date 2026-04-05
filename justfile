# justfile - Project unified command entry

# check
check:
    cargo check &
    cd web && pnpm lint

# Development mode: start backend + web together
dev:
    just dev-web &
    just dev-backend

# Start Rust backend (with hot reload)
dev-backend:
    cargo watch -x run -w src

# Start web (Vite dev mode)
dev-web:
    cd web && pnpm dev

# Build all (production)
build:
    just build-web
    just build-backend

# Build Rust backend release
build-backend:
    cargo build --release

# Build web production bundle
build-web:
    cd web && pnpm build

# Clean build outputs
clean:
    rm -rf /target web/dist

# reset and init database
init-db:
    sqlx database drop -y
    sqlx database create
    sqlx migrate run

# Export TypeScript types from Rust
export-types:
    cargo test export_bindings
    @echo "✅ TypeScript types exported to web/src/api/types/"

# Run tests with coverage report (terminal summary)
coverage:
    cargo llvm-cov --summary-only

# Run tests with coverage and open HTML report
coverage-html:
    cargo llvm-cov --html
    open target/llvm-cov/html/index.html