# Contributing to OpenNotebook

## Getting Started

1. Read `AGENT.md` for the full project overview and architecture
2. Read `docs/PRD.md` for product requirements and roadmap
3. Read the relevant ADRs in `docs/adr/` for design decisions
4. Check `Capability.toml` for current feature status

## Development Setup

See README.md for build instructions. All three components (kernel, server, frontend) must be running for full integration.

## Code Style

- **Rust**: `cargo fmt` + `cargo clippy --all-features -- -D warnings`
- **Go**: `go fmt` + `go vet ./...`
- **TypeScript**: `npx tsc --noEmit` (type-check only; ESLint WIP)

## Pull Requests

1. Ensure all CI checks pass (kernel tests, go vet, frontend build, integration tests)
2. Include tests for new functionality
3. Update `Capability.toml` when adding/changing features
4. Update ADRs for significant architecture changes

## Testing

```bash
cd kernel && cargo test --all-features
cd server && go test ./...
cd frontend && npm run build
```

## Security

Report vulnerabilities privately via GitHub Issues with the `security` label.
