## Context

`osmprj` is a Rust CLI tool with no existing documentation site. The repo root contains `src/`, `Cargo.toml`, `pixi.toml`, and a detailed `README.md`. This change adds a completely separate Docusaurus v3 Node.js project under `docs/` — it has no coupling to the Rust build.

The site deploys to `travishathaway.github.io/osmprj`, which means Docusaurus `baseUrl` must be `/osmprj/`.

## Goals / Non-Goals

**Goals:**
- Bootstrap a Docusaurus v3 site in `docs/` with a custom landing page and three docs sections.
- Wire up the asciinema player to a placeholder `.cast` file so the user only needs to drop in the real recording.
- Automate deployment to GitHub Pages via GitHub Actions.
- Keep docs content light — correct structure and key workflows, stubs welcome.

**Non-Goals:**
- Writing comprehensive reference documentation (deferred to manual authoring).
- Custom CSS theming beyond Docusaurus defaults and basic hero styling.
- Versioned docs.
- i18n.

## Decisions

### 1. Docusaurus v3 with TypeScript

Use Docusaurus v3 (current stable) with TypeScript for the custom pages. The `docs/` directory is a standalone Node.js project with its own `package.json`. This keeps the Rust project untouched and lets the docs evolve independently.

### 2. Directory layout

```
docs/
├── docusaurus.config.ts        # site config (url, baseUrl, navbar, footer)
├── tsconfig.json
├── package.json
├── babel.config.js
├── static/
│   └── demo.cast               # placeholder asciinema cast file
├── src/
│   ├── pages/
│   │   └── index.tsx           # custom landing page
│   └── css/
│       └── custom.css          # hero + feature card overrides
└── docs/
    ├── intro.md
    ├── getting-started.md
    └── guides/
        ├── _category_.json
        ├── managing-sources.md
        ├── syncing-data.md
        ├── themes.md
        └── configuration.md
```

### 3. Asciinema embed via `asciinema-player` npm package

Use the [`asciinema-player`](https://github.com/asciinema/asciinema-player) npm package (not the hosted script). The player is wrapped in a React component that lazy-loads the player JS (client-side only, using Docusaurus `BrowserOnly`). The `.cast` file is stored at `docs/static/demo.cast` and referenced as `/osmprj/demo.cast` (baseUrl-prefixed). The user replaces this placeholder file with their real recording.

```
Landing page (index.tsx)
  └── <BrowserOnly>
        └── <AsciinemaPlayer src="/osmprj/demo.cast" ... />
```

`BrowserOnly` is required because `asciinema-player` accesses browser APIs and will break SSR/build if imported at module scope.

### 4. Landing page structure

```
┌─────────────────────────────────────────────────────┐
│  HERO                                               │
│  osmprj                                             │
│  "A friendly, modern tool for managing              │
│   OSM data with PostgreSQL"                         │
│  [Get Started]  [GitHub]                            │
├─────────────────────────────────────────────────────┤
│  FEATURES  (3 cards)                                │
│  ┌──────────────┐ ┌──────────────┐ ┌─────────────┐ │
│  │ One command  │ │ Auto-tuned   │ │ Incremental │ │
│  │ imports      │ │ for your HW  │ │ updates     │ │
│  └──────────────┘ └──────────────┘ └─────────────┘ │
├─────────────────────────────────────────────────────┤
│  DEMO                                               │
│  ┌─────────────────────────────────────────────┐   │
│  │  asciinema player                           │   │
│  └─────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

Feature cards:
1. **Simple workflow** — `init`, `add`, `sync`. That's it.
2. **Auto-tuned imports** — osmprj picks the right osm2pgsql flags for your hardware automatically.
3. **Incremental updates** — After the first import, subsequent syncs apply only the changes.

### 5. Docs section structure

| Section | File | Content |
|---|---|---|
| Introduction | `docs/intro.md` | What osmprj is, what it wraps, experimental warning |
| Getting Started | `docs/getting-started.md` | Installation (pixi/conda) + 4-step quick start |
| Guides | `docs/guides/` | Four stubs: managing sources, syncing, themes, configuration |

Guides use a `_category_.json` to set the sidebar label to "Guides" with position 3.

### 6. GitHub Actions deploy workflow

```yaml
# .github/workflows/deploy-docs.yml
# Trigger: push to main
# Steps:
#   1. Checkout
#   2. Setup Node 20
#   3. npm ci (in docs/)
#   4. npm run build (in docs/)
#   5. Deploy docs/build/ to gh-pages branch (peaceiris/actions-gh-pages)
```

The workflow uses `peaceiris/actions-gh-pages@v4` which is the standard choice for Docusaurus GitHub Pages deploys. It pushes the built output from `docs/build/` to the `gh-pages` branch.

GitHub Pages must be configured in the repo settings to serve from the `gh-pages` branch.

### 7. Navbar and footer

**Navbar:**
- Left: `osmprj` logo/text → `/`
- Right: `Docs` → `/docs/intro`, `GitHub` → repo URL (external)

**Footer:**
- Single column: GitHub link, "Built with Docusaurus"

## Risks / Trade-offs

- **`asciinema-player` SSR**: The player must only be rendered client-side. `BrowserOnly` from `@docusaurus/core` handles this. Forgetting it causes a cryptic build error — the design explicitly calls it out.
- **Placeholder cast file**: `docs/static/demo.cast` must exist at build time or the player will 404 at runtime (not a build error). A minimal valid cast file (a few lines of JSON) is created as placeholder so the build always succeeds.
- **baseUrl `/osmprj/`**: All internal asset paths must be baseUrl-aware. Docusaurus handles this for markdown links and `useBaseUrl()`, but the hardcoded `/osmprj/demo.cast` player `src` must match. Document this in a comment in `index.tsx`.
- **Node.js in a Rust repo**: Developers need Node 18+ to work on docs. This is not required for the Rust build. The `docs/` directory is self-contained and the GitHub Actions workflow installs its own Node — no pixi/conda changes needed.
