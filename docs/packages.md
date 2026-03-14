# Atlas Package System

## Overview

Atlas uses a **git-native package system** (D-059). There is no central registry server.
Git is the registry. Packages are distributed via git tags and fetched to a local cache.

```toml
[dependencies]
web = { git = "https://github.com/atl-pkg/web", tag = "v0.1.0" }
```

```atlas
import { new_router, htmx } from "web";
```

---

## How It Works

| Step | What happens |
|------|-------------|
| `atlas install` | Reads `atlas.toml`, fetches each git dep to `~/.atlas/cache/<name>/<tag>/`, writes `atlas.lock` |
| `atlas run` | Validates `atlas.lock` against cache before compiling |
| `import "pkg"` | Bare specifier resolves via `atlas.lock` → `~/.atlas/cache/<name>/<tag>/lib.atlas` |
| `atlas update` | Queries remote tags via `git ls-remote`, bumps `atlas.lock` to latest matching tag |
| `atlas publish` | Validates package, creates annotated git tag, prints push instruction |

**Entry point resolution order:** `lib.atlas` → `index.atlas` → `mod.atlas`

**Lockfile:** `atlas.lock` pins each dep to a specific git tag + commit SHA + SHA-256 checksum.
Reproducible builds guaranteed. Always commit `atlas.lock`.

---

## Official Packages — atl-pkg

Official Atlas packages live at **`github.com/atl-pkg`** — separate from the language org.

| Repo | What it is | Version |
|------|-----------|---------|
| [atl-pkg/web](https://github.com/atl-pkg/web) | Router, middleware, templates, HTMX | v0.1.0 |

**Local structure:**
```
~/dev/projects/
  atlas/        → github.com/atl-lang/atlas   (compiler — this repo)
  atl-pkg/
    web/        → github.com/atl-pkg/web
    crypto/     → github.com/atl-pkg/crypto   (future)
```

`~/dev/projects/atl-pkg/` is a local folder only — not a git repo. Each package inside is its own independent repo.

**Why two orgs?**
`atl-lang` stays minimal (compiler, LSP, toolchain). `atl-pkg` scales to any number of packages without polluting the language org. Clean separation, like Go's `golang` vs `golang/x` extended packages — but with better org hygiene.

**Naming:** short, no prefix. `web` not `atlas-web`. The org is the namespace.

---

## Community Packages

Community packages live in their authors' own repos anywhere on GitHub/GitLab/etc.
Discovery: `pkg.atl.dev` (future index — not yet live).

Used exactly the same way:
```toml
web-forms = { git = "https://github.com/someone/web-forms", tag = "v2.1.0" }
```

---

## Creating an Official Package

1. Create repo under `atl-pkg` org
2. Scaffold at `~/dev/projects/atl-pkg/<name>/`
3. `atlas.toml` must include `authors = ["Atlas Team"]`, `license = "MIT"`, `description`
4. Entry point: `lib.atlas` at repo root
5. Tag and push: `git tag v0.1.0 -m "atlas publish <name> v0.1.0" && git push origin v0.1.0`

---

## Known Language Gaps (as of v0.3)

_(None currently — all critical language gaps have been resolved.)_

---

## CLI Reference

```sh
atlas install               # Fetch all deps, write atlas.lock
atlas install --force       # Re-fetch even if lockfile is fresh
atlas install --dry-run     # Show plan without fetching
atlas update                # Bump all deps to latest matching tag
atlas update <pkg>          # Bump one package
atlas update --dry-run      # Show what would change
atlas publish               # Validate + create local git tag
atlas publish --dry-run     # Validate only, no tag
atlas publish --allow-dirty # Skip working tree clean check
```

---

## Design Decisions

- **D-059** — Package system: go-model, git is the registry
- **D-058** — Stdlib boundary: only primitives. Routers, middleware, ORMs → user-land packages
- **D-056** — Web architecture: http.serve() in stdlib, framework in atl-pkg/web
