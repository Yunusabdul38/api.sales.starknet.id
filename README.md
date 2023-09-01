# api.sales.starknet.id

## Overview

This monorepo contains three programs to manage the sales data of the StarkNetID naming smart contract in a secury and privacy preserving way.

## Components

### 1. API Endpoint (`api_endpoint`)

- **Language**: Rust
- **Function**: API for frontend to register sales with metadata (user email, tax state, etc.)
- **Directory**: `./api_endpoint`

### 2. Indexer (`indexer`)

- **Language**: Deno
- **Function**: Fetches transactions and associated metadata hashes.
- **Directory**: `./indexer`

### 3. Sale Actions (`sale_actions`)

- **Language**: Rust
- **Function**: Automates actions like sending or scheduling emails upon a sale.
- **Directory**: `./sale_actions`

## Architecture

The monorepo uses a Rust workspace for the `api_endpoint` and `sale_actions` components. The `indexer` is separate, as it's Deno-based.

```
api.sales.starknet.id/
├── Cargo.toml # Workspace configuration
├── api_endpoint/ # Rust-based API endpoint
├── indexer/ # Deno-based indexer
├── sale_actions/ # Rust-based sales actions
└── README.md # This file
```