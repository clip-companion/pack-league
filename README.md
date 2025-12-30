# League of Legends Game Pack

Game pack for League of Legends integration. Provides:

- **Live Events**: Real-time game events (kills, dragons, barons, etc.)
- **Match History**: Automatic match tracking and statistics
- **Clip Triggers**: Configurable triggers for automatic clip recording

## Structure

```
pack-league/
├── manifest.json       # Pack metadata
├── daemon/            # Rust backend (runs as subprocess)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs    # IPC entry point
│       └── ...        # Integration logic
├── frontend/          # React components
│   ├── package.json
│   └── src/
│       ├── index.ts   # Pack entry point
│       └── ...        # UI components
└── migrations/        # SQLite migrations for pack-specific data
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

This pack communicates with the main companion app via NDJSON over stdin/stdout. See the protocol documentation in the main app repository.

## Status

**Work in Progress**: This pack is being migrated from the main companion app. The following needs to be completed:

- [ ] Daemon: Wire up IPC to actual League integration logic
- [ ] Frontend: Resolve imports from main app (cn, formatters, animations, etc.)
- [ ] Frontend: Add local copies of shared utilities
- [ ] Build: Test cross-platform daemon builds
- [ ] Build: Test frontend bundle generation

## License

MIT
