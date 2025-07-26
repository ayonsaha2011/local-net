# Local Network Chat

A modern lightweight local network IP chat and file sharing cross-platform app using Rust Dioxus.

## Features

✅ **Implemented:**
- Glass morphism UI with 20% transparency and blur effects
- Modern peer list with search functionality and online indicators
- Chat interface with file attachment support
- Settings page with comprehensive options (username, theme, notifications, etc.)
- Profile page with user information
- Custom titlebar with window controls
- Cross-platform compilation (desktop + web)
- SQLite database with complete schema for desktop
- Mock data system for web platform
- End-to-end encryption dependencies ready

🚧 **In Progress:**
- End-to-end encryption for messages and files
- Real-time socket updates and networking
- File transfer with progress tracking
- Typing indicators functionality
- User avatars and profile pictures
- Message replies and threading
- File preview functionality

## Running the App

### Web Version (Recommended for development)

```bash
# Run the web version (recommended for snap environments)
./run-web.sh

# Or manually:
dx serve --platform web --port 8080
```

### Desktop Version

```bash
# Simple run (auto-builds if needed)
./run-desktop.sh

# Or advanced run with system library troubleshooting
./run-desktop-system.sh

# Manual build and run
dx build --platform desktop --release
./target/dx/local-net-chat/release/linux/app/local-net-chat
```

**✅ Snap Environment Support:**
- **Fixed!** Snap environment glibc conflicts have been resolved
- Uses `LD_PRELOAD` to force system libraries over snap libraries
- Both desktop and web versions work perfectly in snap environments
- All run scripts automatically detect and apply the fix

**🔧 Manual Fix (if needed):**
```bash
LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:/lib/x86_64-linux-gnu \
LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libpthread.so.0 \
./target/dx/local-net-chat/release/linux/app/local-net-chat
```

## Development

### Prerequisites

- Rust (latest stable)
- Dioxus CLI: `cargo install dioxus-cli`

### Project Structure

```
src/
├── main.rs              # Main app entry point with glass morphism styling
├── database.rs          # SQLite database with complete schema
├── network.rs           # Network layer for peer discovery and communication
└── components/
    ├── peer_list.rs     # Modern peer list with search
    ├── simple_chat.rs   # Chat interface with file support
    ├── chat_view.rs     # Main chat view component
    ├── settings.rs      # Comprehensive settings page
    └── profile.rs       # User profile page
```

### Key Technologies

- **Frontend:** Dioxus (Rust-based React-like framework)
- **Database:** SQLite with Rusqlite
- **Styling:** CSS-in-Rust with glass morphism effects
- **Encryption:** AES-GCM (dependencies ready)
- **Networking:** Tokio + WebSockets (ready for implementation)

## Architecture

- **Cross-platform:** Desktop (GTK/WebView) and Web (WASM)
- **Database:** SQLite for desktop, mock data for web
- **UI:** Modern glass morphism design with backdrop filters
- **State Management:** Dioxus signals and reactive state

## Quick Start

```bash
# Clone and navigate to the project
cd local-net-chat

# Run web version (recommended)
./run-web.sh

# Or build desktop version
dx build --platform desktop --release
```

## Next Steps

1. Implement real-time networking with peer discovery
2. Add end-to-end encryption for messages
3. Complete file transfer functionality
4. Add typing indicators and real-time updates
5. Implement user avatars and profile pictures

