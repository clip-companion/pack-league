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

Releases are **fully automated**. Just push to main:

```bash
git add -A && git commit -m "feat: your changes"
git push
```

GitHub Actions automatically:
1. Bumps the patch version in `config.json` (0.1.1 → 0.1.2)
2. Commits with `[skip ci]` to avoid infinite loops
3. Creates a git tag (v0.1.2)
4. Builds daemon for all platforms (macOS arm64/x64, Linux x64, Windows x64)
5. Builds frontend.js bundle
6. Creates checksums.txt
7. Publishes GitHub release with all artifacts
8. Sends webhook to `packs-index` to update the version

### Manual Version Bumps

For minor/major version bumps, manually edit `config.json`:

```bash
# Edit config.json to set desired version
git add config.json && git commit -m "chore: bump version to 0.2.0"
git push
```

### Required GitHub Secrets

The release workflow requires one secret:

| Secret | Purpose |
|--------|---------|
| `PACKS_INDEX_TOKEN` | Personal Access Token to trigger packs-index update |

**Setting up PACKS_INDEX_TOKEN:**

1. Go to GitHub → Settings → Developer settings → Fine-grained tokens
2. Create a new token with:
   - Repository: `clip-companion/packs-index`
   - Permissions: Contents (read/write)
3. Add the token as a secret in this repository:
   ```bash
   gh secret set PACKS_INDEX_TOKEN
   # Paste the token when prompted
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

### Running CI Tests Locally

To verify the release configuration is correct before pushing:

```bash
cd frontend
pnpm test:ci
```

This checks that:
- Cargo.toml uses git URL (not local path) for dependencies
- package.json uses git URL (not file: path) for dependencies
- Workflow files are configured correctly

## License

MIT

