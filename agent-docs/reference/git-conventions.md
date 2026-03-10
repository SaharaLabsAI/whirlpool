# Git Conventions

## Commit Message Format

Conventional commits style:

```
<type>(<scope>): <short description>

<optional body explaining why>

Co-authored-by: Sisyphus <clio-agent@sisyphuslabs.ai>
```

### Types
- `feat` — New feature or functionality
- `fix` — Bug fix
- `refactor` — Code restructuring without behavior change
- `docs` — Documentation only changes
- `test` — Adding or updating tests
- `chore` — Maintenance tasks (deps, CI, tooling)

### Scopes
- `workspace` — Changes affecting multiple crates or workspace config
- `consensus` — Changes to the consensus trait crate
- `consensus-simplex` — Changes to the simplex adapter crate
- `whirlpool-node` — Changes to the binary crate
- `chain-binary` — (legacy scope, now `whirlpool-node`)

## Branch Strategy

- Feature branches for non-trivial work
- Atomic commits per completed unit of work
- Keep commits small and focused

## Pre-Push Checklist

```bash
nix develop --command cargo fmt --workspace
nix develop --command cargo clippy --workspace -- -D warnings
nix develop --command cargo nextest run --workspace
```

## Rules

- **Never commit secrets** (.env, credentials, API keys)
- **Never force-push to main** without explicit approval
- **Keep changes small** — one logical change per commit
- **Match existing style** — follow the patterns already in the codebase

## AI Agent Commits

When AI agents create commits, include the co-author trailer:
```
Co-authored-by: Sisyphus <clio-agent@sisyphuslabs.ai>
```
