---
sidebar_position: 1
slug: /
---

# Introduction

**GameVault** is a portable Windows game library manager that helps you organize, browse, and enrich your local game collection with metadata from Steam.

## What is GameVault?

GameVault is a lightweight, self-contained application that:

- **Scans** your game folders and automatically detects installed games
- **Enriches** game entries with metadata from Steam (covers, descriptions, reviews, genres)
- **Organizes** your library with search, filtering, and detailed game views
- **Runs anywhere** - fully portable with no installation required

## Key Features

| Feature | Description |
|---------|-------------|
| **Portable** | Single ~20 MB executable - runs from USB, no installation |
| **Steam Integration** | Automatic metadata enrichment from Steam store |
| **Edit Metadata** | Manually edit game titles, descriptions, and details |
| **Adjust Matches** | Fix incorrect Steam matches with correct game ID |
| **Keyboard Navigation** | Full accessibility support with keyboard controls |
| **Local Caching** | Cover images and metadata cached locally |
| **Web Interface** | Beautiful, responsive UI accessible via browser |
| **System Tray** | Runs in background with tray icon for quick access |
| **Configurable** | Simple TOML configuration file |
| **Docker Ready** | Multi-stage Dockerfile for containerized deployment |

## How It Works

```
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│  Game Folders   │────▶│   Scanner    │────▶│    Database     │
│  (D:\Games)     │     │              │     │   (SQLite)      │
└─────────────────┘     └──────────────┘     └─────────────────┘
                                                      │
                                                      ▼
┌─────────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Web Browser   │◀────│  Web Server  │◀────│  Steam API      │
│  (localhost)    │     │   (Axum)     │     │  (Enrichment)   │
└─────────────────┘     └──────────────┘     └─────────────────┘
```

1. **Scan**: GameVault scans your configured game library folder
2. **Match**: Games are matched to Steam entries using fuzzy title matching
3. **Enrich**: Metadata (covers, descriptions, reviews) is fetched from Steam
4. **Browse**: Access your library through a beautiful web interface

## Architecture

GameVault is built with modern technologies:

- **Backend**: Rust with Axum web framework and SQLite database
- **Frontend**: Next.js 15 with React 19, statically exported and embedded
- **Packaging**: Single portable Windows executable via rust-embed

## Next Steps

Ready to get started? Head to [Getting Started](./getting-started) to download and run GameVault.
