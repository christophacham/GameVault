---
sidebar_position: 1
---

# Architecture Overview

GameVault is a portable game library manager built with a Rust backend and React frontend, designed to run as a single executable on Windows.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GameVault.exe                                │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Rust Backend (Axum)                          ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        ││
│  │  │  Router  │  │ Handlers │  │    DB    │  │  Steam   │        ││
│  │  │          │──│          │──│ (SQLite) │  │   API    │        ││
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘        ││
│  │       │              │             │             │              ││
│  │       ▼              ▼             ▼             ▼              ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐        ││
│  │  │ Embedded │  │ Scanner  │  │  Local   │  │  Config  │        ││
│  │  │  Static  │  │          │  │ Storage  │  │  (TOML)  │        ││
│  │  │  Assets  │  └──────────┘  └──────────┘  └──────────┘        ││
│  │  └──────────┘                                                   ││
│  └─────────────────────────────────────────────────────────────────┘│
│                              │                                       │
│                              ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                   Frontend (Embedded)                           ││
│  │  ┌─────────────────────────────────────────────────────────────┐││
│  │  │  Next.js 15 Static Export + React 19 + TailwindCSS          │││
│  │  └─────────────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
          │                    │                      │
          ▼                    ▼                      ▼
    ┌──────────┐        ┌──────────┐          ┌──────────┐
    │  System  │        │   Web    │          │   Game   │
    │   Tray   │        │ Browser  │          │ Folders  │
    └──────────┘        └──────────┘          └──────────┘
```

## Core Design Principles

### 1. Portable Executable
GameVault compiles to a single `.exe` file with the frontend embedded using `rust-embed`. No installation, no dependencies, no registry entries.

### 2. Dual-Write Persistence
All metadata changes are written to both:
- **SQLite database**: Primary storage for fast queries
- **`.gamevault/metadata.json`**: Per-game folder backup for portability

### 3. Localhost by Default
Security-first approach:
- Binds to `127.0.0.1` by default
- CORS restricted to localhost origins
- Optional API key authentication

### 4. Graceful Degradation
- Works offline (browsing cached data)
- Steam API failures don't break the app
- Missing metadata is handled gracefully

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Runtime** | Rust + Tokio | Async runtime, memory safety |
| **Web Framework** | Axum 0.7 | HTTP server, routing, middleware |
| **Database** | SQLite + SQLx | Embedded database with async queries |
| **Frontend** | Next.js 15 / React 19 | Static export for embedding |
| **Styling** | TailwindCSS | Utility-first CSS |
| **Asset Embedding** | rust-embed | Compile frontend into binary |
| **System Tray** | tray-icon | Windows notification area |

## Module Dependencies

```
main.rs
  ├── config.rs      (configuration loading)
  ├── db.rs          (database operations)
  ├── handlers.rs    (API endpoints)
  │     ├── scanner.rs    (folder scanning)
  │     ├── steam.rs      (Steam API client)
  │     └── local_storage.rs (file I/O)
  ├── embedded.rs    (static file serving)
  ├── tray.rs        (system tray - Windows)
  └── models.rs      (data structures)
```

## Data Flow

### Scan → Enrich → Browse

```
1. SCAN
   Game Folders ──▶ scanner.rs ──▶ db.rs ──▶ SQLite

2. ENRICH
   SQLite ──▶ steam.rs ──▶ Steam API ──▶ db.rs ──▶ SQLite
                                      └──▶ local_storage.rs ──▶ .gamevault/

3. BROWSE
   Browser ──▶ API ──▶ handlers.rs ──▶ db.rs ──▶ SQLite ──▶ JSON Response
```

### Edit Flow (Dual-Write)

```
User Edit ──▶ PUT /api/games/:id
                    │
                    ├──▶ db.rs (UPDATE + SELECT in transaction)
                    │         └──▶ SQLite
                    │
                    └──▶ local_storage.rs
                              └──▶ .gamevault/metadata.json
```

## Deployment Options

| Option | Best For | Configuration |
|--------|----------|---------------|
| **Windows Portable** | Personal use | `GameVault.exe` + `config.toml` |
| **Docker/Podman** | Server deployment | `docker-compose.yml` |
| **Development** | Contributing | Separate frontend/backend |

## Next Steps

- [Backend Architecture](./backend) - Deep dive into Rust modules
- [Frontend Architecture](./frontend) - React components and state
- [Database Schema](./database) - SQLite table structure
- [Asset Embedding](./embedding) - How rust-embed works
