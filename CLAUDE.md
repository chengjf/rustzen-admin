# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rustzen-admin** is a full-stack admin system built with:
- **Backend**: Rust + Axum + SQLx + PostgreSQL
- **Frontend**: React + TypeScript + Vite + Ant Design + TanStack Router
- **Architecture**: Domain-driven layered architecture with RBAC permission system

The project uses a modular feature-based organization with clear separation between `core/`, `common/`, `middleware/`, and `features/`.

---

## Quick Commands

### Development

```bash
# Start both backend and frontend in development mode
just dev

# Start backend only (with hot reload via cargo-watch)
just dev-backend

# Start frontend only
just dev-web

# Run backend check (build verification without producing binary)
cargo check

# Run frontend linting
cd web && pnpm lint
```

### Building

```bash
# Build everything (frontend + backend release)
just build

# Build backend release binary only
just build-backend

# Build frontend production bundle only
just build-web
```

### Database

```bash
# Initialize database (drop, create, run migrations)
just init-db

# Check migration status
sqlx migrate info

# Run migrations manually
sqlx migrate run

# Generate TypeScript types from Rust (exports to web/src/api/types/)
just export-types
```

### Testing

```bash
# Run all Rust tests (unit + sqlx integration)
cargo test

# Run specific test by name
cargo test test_name

# Run tests with stdout output
cargo test -- --nocapture

# Run only the RBAC end-to-end integration tests
cargo test --test integration_rbac_flow

# Run frontend type checking
cd web && pnpm build:prod

# Run frontend lint
cd web && pnpm lint
```

### Coverage

Requires `cargo-llvm-cov` (one-time install):

```bash
cargo install cargo-llvm-cov
```

```bash
# Terminal summary (per-file line/function/region %)
just coverage

# Generate HTML report and open in browser
just coverage-html
```

### Cleanup

```bash
# Clean build artifacts
just clean
# Removes: target/ (Rust) and web/dist/ (frontend)
```

---

## Architecture

### Backend Structure (Rust/Axum)

```
src/
├── main.rs              # Entry point: server startup, route assembly
├── core/                # Infrastructure & core capabilities
│   ├── app.rs           # Server creation, middleware stack, route configuration
│   ├── config.rs        # Configuration loading (env + defaults via Figment)
│   ├── db.rs            # Database connection pool initialization
│   ├── extractor.rs     # Axum extractors (e.g., CurrentUser)
│   ├── jwt.rs           # JWT generation & validation
│   ├── password.rs      # Password hashing (argon2)
│   ├── permission.rs    # Permission checking & caching (RBAC)
│   ├── system_info.rs   # System metrics (CPU, memory)
│   └── web_embed.rs     # Static file serving for frontend
├── middleware/          # Axum middleware
│   ├── auth.rs          # Authentication middleware
│   └── log.rs           # Request logging middleware
├── common/              # Shared types & utilities across features
│   ├── api.rs           # Unified response (ApiResponse, AppResult, OptionItem)
│   ├── error.rs         # Error types (ServiceError, AppError)
│   ├── pagination.rs    # Pagination helpers
│   ├── router_ext.rs    # Router extensions (route_with_permission)
│   └── files.rs         # File upload handling
└── features/            # Business features (domain modules)
    ├── auth/            # Login, token refresh, session management
    ├── dashboard/       # Dashboard data & statistics
    └── system/          # System management
        ├── user/        # User CRUD, user-role associations
        ├── role/        # Role management, permission assignments
        ├── menu/        # Menu tree, permission-based filtering
        └── log/         # System audit logs, partitioning (by month)
```

### Frontend Structure (React/TypeScript)

```
web/
├── src/
│   ├── main.tsx                     # App bootstrap, QueryClient setup
│   ├── api/                         # API clients & TypeScript types
│   │   ├── index.ts                # Main API functions (wrappers)
│   │   ├── types/                  # Auto-generated Rust types
│   │   ├── auth.ts                 # Auth-specific API calls
│   │   └── system/                 # System module API calls
│   ├── routes/                      # TanStack Router pages
│   │   ├── __root.tsx              # Root layout, auth provider
│   │   ├── index.tsx               # Home/dashboard
│   │   ├── login.tsx               # Login page
│   │   ├── 403.tsx, 404.tsx        # Error pages
│   │   └── system/                 # System management pages
│   ├── components/                  # Reusable UI components
│   │   ├── auth/                   # Login form, auth guards
│   │   ├── button/                 # Button variants
│   │   ├── user/                   # User-related components
│   │   ├── permission-group-select/# Permission assignment UI
│   │   ├── TabBar.tsx              # Multi-tab navigation bar
│   │   └── error-boundary.tsx      # React error boundary
│   ├── layouts/                    # Layout components
│   ├── stores/                     # Zustand state stores
│   ├── constant/                   # Constants (e.g., permission enums)
│   ├── util/                       # Utility functions
│   └── assets/                     # Static assets
├── vite.config.ts                   # Vite + TanStack Router config
├── tailwind.config.js               # TailwindCSS (v4)
└── package.json                     # Dependencies (pnpm)
```

---

## Key Architectural Patterns

### Backend Layering Rules

**Dependency direction (outer → inner):**
```
main.rs
├── core/      ← innermost, no feature dependencies
├── common/    ← cross-feature, depends on nothing in features/
├── middleware/ ← depends on common/error
└── features/
    ├── api.rs     → extracts DTOs, calls service, returns ApiResponse
    ├── service.rs → business logic, validations, coordinates repos
    ├── repo.rs    → pure SQL, returns models only
    ├── model.rs   → DB entity (no HTTP types, snake_case fields)
    └── dto.rs     → request/query/response structs (can From<Model])
```

**Forbidden dependencies:**
- ✗ `model.rs` may not depend on DTOs
- ✗ `repo.rs` may not call other feature repos directly
- ✗ `service.rs` may not directly access another feature's repo (use their service instead)
- ✗ `core/` may not depend on `features/`

### Permission System

The RBAC system uses a flexible `PermissionsCheck` enum with three modes:

```rust
use crate::features::auth::permission::PermissionsCheck;

// Single permission
PermissionsCheck::Single("system:user:list")

// Any of multiple permissions (OR logic)
PermissionsCheck::Any(vec!["system:user:create", "admin:full"])

// All required permissions (AND logic)
PermissionsCheck::All(vec!["system:user:delete", "admin:confirm"])
```

Routes register via the `route_with_permission` extension method (in `common/router_ext.rs`), which automatically applies the auth middleware and permission checks.

**Permission naming convention:** `domain:resource:action` (e.g., `system:user:list`, `system:user:create`)

### API Response Pattern

All API handlers return `AppResult<T>`, which is `Result<Json<ApiResponse<T>>, AppError>`.

Success response:
```rust
Ok(ApiResponse::success(data))
// => { code: 0, message: "操作成功", data: ... }

Ok(ApiResponse::page(items, total))
// => { code: 0, message: "操作成功", data: [...], total: 123 }
```

Errors return `AppError`, which implements `IntoResponse` with appropriate HTTP status and JSON body.

### Database Conventions

- **SQLx**: `sqlx::query_as` with compile-time checking; run `cargo check` after query changes to verify
- **Pool**: `PgPool` passed through function parameters (not stored in repositories)
- **Migrations**: Located in `migrations/` and grouped by category (`0101_table.sql` through `0105_seed.sql`)
- **Operation logs**: `operation_logs` is a plain table defined in `0101_table.sql`
- **Login lockout**: Defined in `0101_table.sql` and `0104_func.sql`; accounts are locked for 30 minutes after 5 consecutive failures

### Frontend State & API

- **State management**: Zustand stores in `web/src/stores/`
- **Data fetching**: TanStack Query (`@tanstack/react-query`) for server state
- **Routing**: TanStack Router (`@tanstack/react-router`) with file-based routes
- **HTTP client**: Custom wrapper in `web/src/api/index.ts` using native fetch with interceptor for auth headers and error handling
- **UI**: Ant Design v6 + TailwindCSS v4

---

## Configuration

### Environment Variables (Backend)

Prefix: `RUSTZEN_`

| Variable | Default | Description |
|----------|---------|-------------|
| `RUSTZEN_DB_URL` | `sqlite://rustzen.db` | PostgreSQL connection string |
| `RUSTZEN_APP_HOST` | `0.0.0.0` | Bind address |
| `RUSTZEN_APP_PORT` | `8007` | Port |
| `RUSTZEN_JWT_SECRET` | `rustzen-admin-secret-key` | JWT signing secret (CHANGE IN PROD) |
| `RUSTZEN_JWT_EXPIRATION` | `3600` | Token TTL in seconds |
| `RUSTZEN_DB_MAX_CONN` | `10` | Max DB connections |
| `RUSTZEN_DB_MIN_CONN` | `1` | Min DB connections |
| `RUSTZEN_RUST_LOG` | - | Rust log level filter |

**Note:** `sqlx-cli` expects `DATABASE_URL` instead of `RUSTZEN_DB_URL`.

### Frontend Environment

The frontend proxies API requests to the backend in development via Vite config (check `web/vite.config.ts`). Production builds expect the backend at the same origin or configure API_BASE_URL.

---

## Code Style & Conventions

### Rust

- **Edition**: 2024
- **Formatting**: `cargo fmt` (rustfmt.toml present)
- **Linting**: Consider adding `cargo clippy` (not configured yet)
- **Error handling**: Use `thiserror` for custom errors; propagate via `?` operator
- **Async**: All I/O should be async using `tokio`; prefer `.await` on async calls

### TypeScript/React

- **Build tool**: Vite 8 + TanStack Router plugin
- **Linting**: Oxlint (`.oxlintrc.json` present)
- **Formatting**: `pnpm fmt` (via `vite-plus`)
- **Components**: Functional components with hooks; prefer named exports

### Naming Conventions (Rust)

See `docs/architecture.md` (in Chinese) for detailed conventions. Key points:

- `model.rs`: DB entities (PascalCase, snake_case fields)
- `dto.rs`: `CreateXxxDto`, `UpdateXxxDto`, `XxxQuery`, `XxxResp`, `XxxVo`
- `repo.rs`: Functions: `find_by_*`, `get_by_id` (returns error if not found), `insert`, `update_by_id`, `delete_by_id`
- `service.rs`: Functions: `create`, `get`, `list`, `update`, `delete`

---

## Important Implementation Notes

### Database Migrations

- New database changes must include a migration file in `migrations/`
- Use the existing category-aligned numbering pattern in `migrations/`
- Run `sqlx migrate add <description>` to scaffold a new migration
- After modifying queries, run `cargo check` to ensure SQLx compile-time checks pass

### Frontend API Client

The TypeScript API client (`web/src/api/`) is generated from Rust types using `ts-rs`. After adding/changing Rust structs marked with `#[derive(TS)]`, run:

```bash
just export-types
```

This runs a test that regenerates types into `web/src/api/types/`. Do this before frontend work that depends on type changes.

### Static File Serving

The backend serves the frontend build from the `web/dist` directory via fallback route (commented out in `core/app.rs`). The current default uses `web_embed::web_embed_file_handler` which embeds static files into the binary at compile time using `include_dir!`.

To switch to serving from filesystem (easier for dev), modify `core/app.rs`:
```rust
.fallback(ServeDir::new("web/dist").append_index_html_on_directories(true))
```

### Authentication Flow

1. **Login**: POST `/api/auth/login` returns `{ token }`
2. **Token usage**: Include `Authorization: Bearer <token>` header
3. **Middleware**: `auth_middleware` validates token, loads user, injects `CurrentUser` extractor
4. **Permission check**: Use `route_with_permission` macro to enforce RBAC
5. **Refresh**: Not yet implemented (if adding, extend `features/auth`)
6. **Login lockout**: After 5 consecutive failures, account is auto-locked for 30 minutes (`UserStatus::Locked`); lock clears automatically on next login after expiry. Token validation also checks lock status and invalidates tokens for locked users.

---

## Testing

### Backend test structure

Tests are organized in three layers, all using `#[sqlx::test]` which auto-creates/migrates/tears-down a real PostgreSQL database per test:

| Layer | Location | What's tested |
|-------|----------|--------------|
| **Unit** | `#[test]` inline in source | Pure logic: pagination, password hashing, permission checks, CSV escaping, menu type constraints |
| **Repo** | `#[sqlx::test]` at end of each `repo.rs` | SQL correctness: CRUD, uniqueness queries, soft delete, filters |
| **Service** | `#[sqlx::test]` at end of each `service.rs` | Business validation: uniqueness guards, role/menu path integrity, lockout flow, status transitions |
| **Integration** | `tests/integration_rbac_flow.rs` | End-to-end RBAC lifecycle: role creation → user creation → login lockout → admin unlock → permission verification |

Current line coverage: **~44%** (business logic layers ~60%, API handlers 0% — not yet covered by HTTP-level tests).

### Frontend

- **Unit/integration**: Vitest (`web/vitest.config.ts`) — stores and API auth headers are covered
- Run: `cd web && pnpm test`

---

## Troubleshooting

### Database Connection Issues

- Ensure PostgreSQL is running
- Verify `RUSTZEN_DB_URL` is correct and database exists
- Use `sqlx migrate info` to check migration status
- Check that `sqlx-cli` version matches `sqlx` crate version (run `sqlx --version`)

### "Type Mismatch" After Rust Type Changes

Regenerate TypeScript types:
```bash
just export-types
```

### Frontend Not Loading API Data

- Check browser dev tools Network tab for CORS errors (in dev, CORS is permissive)
- Verify backend is running on expected port (default 8007)
- Check that `Authorization` header is present if endpoint is protected

### Permission Denied

1. Ensure user has role assignments via `system/user` and `system/role` APIs
2. Verify the permission code string matches exactly the one in `route_with_permission`
3. Check backend logs for permission check details (debug level)

---

## Related Documentation

- [docs/architecture.md](./docs/architecture.md) - Detailed Rust architecture design (Chinese)
- [docs/permissions-guide.md](./docs/permissions-guide.md) - Permission system usage guide
- [docs/static-files.md](./docs/static-files.md) - Static file serving options
- [docs/git.md](./docs/git.md) - Git workflow guidelines
- [README.md](./README.md) - Project overview, quick start

---

## Notes for Claude Code

When working on this codebase:
1. **Respect layered architecture**: Keep dependencies flowing inward; never let inner layers depend on outer layers
2. **Database changes**: Always pair with migrations and update `ts-rs` derives if structs are exposed to frontend
3. **Permissions**: Use `route_with_permission` for all protected routes; choose the appropriate `PermissionsCheck` variant
4. **Error handling**: Return `ServiceError`/`AppError` types; avoid panics or unwrap in production code
5. **Minimalism**: Follow the principle of "sufficient but not excessive" design as stated in architecture.md

For frontend changes:
1. Keep API calls in `web/src/api/`, UI in `components/`, pages in `routes/`
2. Use TanStack Query for server state; Zustand for client UI state
3. Follow existing component patterns (Ant Design + TailwindCSS)

---

*Last analyzed: 2026-04-05*
*Claude Code: claude.ai/code*
