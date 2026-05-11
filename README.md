# Cirrus

A clean, lightweight Minecraft launcher built with Tauri v2 and SvelteKit.

## Features

- **Microsoft authentication** via device code flow — sign in with any Microsoft account that owns Minecraft
- **Isolated instances** — every instance has its own mods, worlds, resource packs, screenshots, and config. Nothing bleeds between instances
- **Mod browser** — search and install mods from Modrinth with one click, SHA512-verified before install
- **Asset integrity** — every file downloaded from Mojang is SHA1-verified before use
- **Optimized JVM args** — Aikar's flags applied automatically, RAM allocated based on your system
- **Options sync** — share a single `options.txt` across all instances, or keep them separate per-instance
- No ads. No upsells. No telemetry.

## Stack

| | |
|---|---|
| Framework | Tauri v2 |
| Frontend | SvelteKit + Tailwind CSS |
| Backend | Rust 2021 |
| HTTP | reqwest (rustls, TLS enforced) |
| Storage | tauri-plugin-store (OS-encrypted) |

## Setup

### Prerequisites

- [Rust](https://rustup.rs)
- [Node.js](https://nodejs.org) (v18+)
- A Microsoft Azure app registration (public client, device code flow)

### Configuration

Create a `.env` file in the project root:

```
AZURE_CLIENT_ID=your-azure-client-id
```

Your Azure app registration must have:
- Platform: **Mobile and desktop applications**
- Redirect URI: `https://login.microsoftonline.com/common/oauth2/nativeclient`
- Allow public client flows: **Yes**
- Supported account types: **Personal Microsoft accounts**

### Running

```sh
npm install
npm run tauri dev
```

### Building

```sh
npm run tauri build
```

## Security

Tokens are never logged, stored in plaintext, or sent to the frontend. The MSA refresh token is the only credential persisted to disk, stored via OS-level encryption (DPAPI on Windows, Keychain on macOS, libsecret on Linux). All other tokens are held in memory only and zeroized after use.
