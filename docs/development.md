---
sidebar_position: 7
---

# Development

Guide for contributing to GameVault development.

## Prerequisites

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- [Podman](https://podman.io/) or Docker (for containerized builds)

## Project Structure

```
GameVault/
├── backend/           # Rust Axum server
│   ├── src/
│   │   ├── main.rs          # Server entry point
│   │   ├── handlers.rs      # API endpoint handlers
│   │   ├── db.rs            # SQLite operations
│   │   ├── scanner.rs       # Directory scanning
│   │   ├── steam.rs         # Steam API client
│   │   ├── models.rs        # Data structures
│   │   ├── embedded.rs      # Static asset embedding
│   │   └── local_storage.rs # Local metadata storage
│   └── Cargo.toml
├── frontend/          # Next.js 15 application
│   ├── src/
│   │   ├── app/             # App router pages
│   │   ├── components/      # React components
│   │   ├── lib/             # API client
│   │   └── test/            # Test setup
│   ├── vitest.config.ts
│   └── package.json
└── docs/              # Documentation
```

## Running Tests

### Frontend Tests (Vitest + Testing Library)

```bash
cd frontend

# Run tests in watch mode
npm test

# Run tests once (CI mode)
npm run test:run

# Run with coverage
npm run test:coverage
```

Test files are located next to the components they test:
- `EditModal.test.tsx` - Edit metadata modal tests
- `AdjustMatchModal.test.tsx` - Steam match adjustment tests
- `GameMenu.test.tsx` - Context menu and keyboard navigation tests

### Backend Tests (Cargo)

```bash
cd backend

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_clean_title
```

### Containerized Tests

Run all tests in isolated containers (useful for CI/CD):

```bash
# Build and run test container
podman build -f Dockerfile.test -t gamevault-test .

# This runs:
# - 28 frontend tests (Vitest)
# - 17 backend tests (Cargo)
```

## Development Workflow

### Backend Development

```bash
cd backend

# Run with hot reload
cargo watch -x run

# Build release
cargo build --release
```

### Frontend Development

```bash
cd frontend

# Development server with hot reload
npm run dev

# Build static export
npm run build

# Type checking
npm run typecheck
```

### Full Stack Development (Windows)

```powershell
# Start both frontend and backend in development mode
./dev.ps1
```

## Building

### Linux Container Build

```bash
# Build Docker/Podman image
podman build -t gamevault:latest .

# Run container
podman run -d -p 3000:3000 -v /games:/games:ro gamevault:latest
```

### Windows Portable Executable

```powershell
# Build portable .exe with embedded frontend
./build-portable.ps1

# Output: dist/GameVault.exe
```

The Windows build:
1. Builds the Next.js frontend as static export
2. Cross-compiles Rust backend for Windows (x86_64-pc-windows-gnu)
3. Embeds the frontend using rust-embed
4. Produces a single portable executable

## Architecture Notes

### Frontend Embedding

The frontend is embedded into the Rust binary using `rust-embed`:

```rust
#[derive(RustEmbed)]
#[folder = "../frontend/out/"]
pub struct StaticAssets;
```

This allows the entire application to be distributed as a single executable.

### Database Transactions

Database operations that require atomicity use SQLx transactions:

```rust
let mut tx = pool.begin().await?;
// ... operations ...
tx.commit().await?;
```

### Keyboard Accessibility

All interactive components support keyboard navigation:
- **Escape**: Close modals and menus
- **Arrow keys**: Navigate menu items
- **Enter/Space**: Activate menu items
- **Tab**: Focus navigation

## Code Style

### Rust
- Follow Rust standard formatting (`cargo fmt`)
- Use `clippy` for linting (`cargo clippy`)

### TypeScript
- ESLint with Next.js config
- Prettier for formatting
- Strict TypeScript mode enabled

## Submitting Changes

1. Create a feature branch
2. Write tests for new functionality
3. Ensure all tests pass (`Dockerfile.test`)
4. Submit a pull request
