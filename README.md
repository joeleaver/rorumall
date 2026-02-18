# Rorumall

A native desktop chat client for the [OFSCP](https://forumall.org) (Open Federated Social Chat Protocol), built with Rust and the [Rinch](https://github.com/joeleaver/rinch) UI framework.

Rorumall connects to any OFSCP-compatible server and provides real-time messaging, group management, file sharing, and federated identity across multiple server instances.

## Features

### Messaging
- **Real-time chat** over WebSocket with automatic reconnection
- **Three message types** — standard messages, memos, and long-form articles with Markdown
- **File attachments** — upload files via file picker or paste images directly from clipboard (Ctrl+V)
- **Message threading** — reply to specific messages
- **Inline image previews** — instant thumbnails for pasted and uploaded images

### Groups & Channels
- Create and manage groups with custom privacy settings
- Multiple channels per group (text and voice types)
- Role-based access control with custom roles and colors
- Channel-level permissions for viewing and sending messages
- Group discoverability settings (public, unlisted, private)

### Identity & Security
- **OFSCP Ed25519 signing** — every API request is cryptographically signed
- Device keypair generation and persistent storage
- Federated identity (`@handle@domain`) across server instances
- Session management with secure credential storage

### Presence & Privacy
- Real-time presence updates (Online, Away, Do Not Disturb, Offline)
- Custom status messages
- Granular privacy controls — configure visibility for profile, presence, and group membership
- Per-setting policies: Public, Authenticated, Shared Groups, Contacts Only, Nobody

### Federation
- Connect to multiple OFSCP servers simultaneously
- Per-host WebSocket connections with independent state
- Cross-server group participation
- OFSCP discovery document support

## Architecture

```
crates/
  shared/     Protocol types, models, and OFSCP signing
  client/     Desktop UI application
    src/
      views/        7 full-screen views (login, home, channel, profile, etc.)
      components/   Reusable UI (messages, modals, profile cards, etc.)
      stores/       6 signal-based reactive stores
      ws/           WebSocket connection manager with reconnection
```

**Key patterns:**
- **Signal-based reactivity** — thread-local singleton stores with Rinch signals
- **Async bridge** — Tokio runtime for network I/O with main-thread callbacks for UI updates
- **Lazy connections** — WebSocket connections created on-demand per host
- **Cryptographic signing** — Ed25519 signatures on all HTTP and WebSocket requests

## Building

Requires Rust nightly and the Rinch framework as a sibling directory:

```
dev/
  rinch/       # https://github.com/joeleaver/rinch
  rorumall/    # this repo
```

```sh
# Build
cargo build

# Run
cargo run -p rorumall

# Size-optimized release
cargo build --profile small
```

### Dependencies

Rorumall uses Rinch for rendering (Vello/wgpu) and depends on:

| Category | Crates |
|----------|--------|
| UI | rinch, rinch-core, rinch-tabler-icons |
| Networking | reqwest (TLS), tokio-tungstenite |
| Crypto | ed25519-dalek, sha2, base64 |
| Serialization | serde, serde_json |
| Content | pulldown-cmark (Markdown), ammonia (sanitization) |
| Media | image (PNG encoding/decoding) |

## Usage

1. Launch the app — you'll see the login screen
2. Enter your OFSCP server URL (e.g. `forumall.lostconnection.dev`)
3. Register or log in with your handle and password
4. Browse groups in the sidebar, click channels to chat
5. Use the message input to send messages, attach files, or paste images

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Ctrl+V | Paste image from clipboard as attachment |
| Enter | Send message |

## Protocol

Rorumall implements the OFSCP client protocol:

- **Authentication** — handle + password with Ed25519 keypair registration
- **Request signing** — `METHOD\nPATH\nTIMESTAMP\nBODY_HASH` signed with Ed25519
- **WebSocket** — bidirectional messaging with `ClientCommand` / `ServerEvent` envelopes
- **Federation** — actor URIs (`@handle@domain`) for cross-server identity

## License

MIT
