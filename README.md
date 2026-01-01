# League of Legends Game Pack

Game pack for League of Legends and TFT integration with Clip Companion.

## Features

- **Live Events**: Real-time game events (kills, dragons, barons, TFT placements, etc.)
- **Match History**: Automatic match tracking and statistics
- **Clip Triggers**: Configurable triggers for automatic clip recording
- **TFT Support**: Full Teamfight Tactics integration (same client, different UI)
- **LCU Events**: Client connection, queue, champion select, and phase change events

## Structure

```
pack-league/
├── config.json        # Pack configuration (version, stage config, triggers)
├── daemon/            # Rust backend (runs as subprocess)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs    # IPC entry point
│       ├── integration.rs  # LCU + Live Client API integration
│       ├── lcu.rs     # League Client API
│       └── ...
├── frontend/          # React components
│   ├── package.json
│   ├── index.ts       # Pack entry point
│   ├── MatchCard.tsx  # League match card
│   ├── TFTMatchCard.tsx   # TFT match card
│   └── ...
└── events-seed.json   # Event definitions
```

## Building

### Daemon

```bash
cd daemon
cargo build --release
```

### Frontend

```bash
cd frontend
pnpm install
pnpm build
```

## Development

This pack communicates with the main Clip Companion daemon via NDJSON over stdin/stdout using the `companion-pack-protocol` crate.

```rust
use companion_pack_protocol::{run_gamepack, GamepackHandler};

struct LeagueHandler { /* ... */ }

impl GamepackHandler for LeagueHandler {
    fn detect_running(&self) -> bool { /* LCU lockfile detection */ }
    fn poll_events(&mut self) -> Vec<GameEvent> { /* Live Client API */ }
    // ...
}

fn main() {
    run_gamepack(LeagueHandler::new());
}
```

## Releasing Updates

This pack uses GitHub Actions to automatically build releases when you push a version tag.

### Release Workflow

```bash
# 1. Make your changes and commit
git add -A && git commit -m "feat: your changes"

# 2. Update version in config.json
# Edit config.json: "version": "0.2.0"
git add config.json && git commit -m "chore: bump version to 0.2.0"

# 3. Create and push tag (triggers GitHub Actions)
git tag v0.2.0
git push && git push --tags

# 4. GitHub Actions automatically:
#    - Builds daemon for macOS (arm64, x64), Linux (x64), Windows (x64)
#    - Builds frontend.js bundle
#    - Creates checksums.txt
#    - Publishes GitHub release with all artifacts

# 5. Update packs-index with new version
cd ~/Projects/packs-index
# Edit index.json: update "version": "0.2.0" for the league pack
git add -A && git commit -m "chore: update league pack to v0.2.0"
git push
```

### How Updates Reach Users

1. Main app fetches `packs-index` repository to get available packs
2. Compares installed version vs index version
3. Shows "Update" button in the Packs UI when versions differ
4. Update downloads new artifacts from GitHub release

### Release Artifacts

Each release includes:
- `daemon-darwin-arm64` - macOS Apple Silicon
- `daemon-darwin-x64` - macOS Intel
- `daemon-linux-x64` - Linux x64
- `daemon-win32-x64.exe` - Windows x64
- `frontend.js` - React component bundle
- `checksums.txt` - SHA256 checksums

## License

MIT
