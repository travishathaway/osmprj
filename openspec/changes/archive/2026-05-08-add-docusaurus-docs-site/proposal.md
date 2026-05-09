## Why

`osmprj` has no public documentation site. The README covers the tool well but is not discoverable, not navigable, and not suited for onboarding new users. A hosted docs site lowers the barrier to adoption, gives the project a professional presence, and provides a natural home for richer guides as the tool matures.

## What Changes

- Add a `docs/` directory at the repository root containing a Docusaurus v3 site.
- The site has a landing page (marketing-style) with a hero banner, feature highlights, and an embedded asciinema terminal demo.
- The docs section is structured with three top-level sections: **Introduction**, **Getting Started**, and **Guides**.
- The site is deployed automatically via a GitHub Actions workflow to GitHub Pages at `travishathaway.github.io/osmprj`.
- No changes are made to the Rust source code or the existing build system.

## Capabilities

### New Capabilities

- `docs-site`: A Docusaurus v3 static site in `docs/` that builds and serves the osmprj documentation.
- `landing-page`: A custom React landing page (`docs/src/pages/index.tsx`) with hero banner (tagline: "A friendly, modern tool for managing OSM data with PostgreSQL"), feature highlights, and an asciinema player component wired to a placeholder `.cast` file.
- `docs-content`: Markdown content for Introduction, Getting Started, and Guides sections — light key-workflow coverage, structured for easy manual expansion.
- `docs-dev-container`: A `docs/docker-compose.yaml` that runs the Docusaurus dev server in a Node container with the `docs/` directory mounted, so local development does not require Node installed on the host.
- `gh-pages-deploy`: GitHub Actions workflow (`.github/workflows/deploy-docs.yml`) that builds the Docusaurus site on push to `main` and deploys to the `gh-pages` branch.

## Testing

- `npm run build` inside `docs/` must succeed without errors.
- The deployed site must be reachable at `travishathaway.github.io/osmprj` after the GitHub Actions workflow runs.
- The asciinema player component must render without errors (pointing at the placeholder cast file path).
- `docker compose up` inside `docs/` must start the dev server and serve the site on a local port.

## Impact

- `docs/`: New Docusaurus v3 project (Node.js, not affecting the Rust build).
- `docs/docker-compose.yaml`: Docker Compose file for local development without a host Node install.
- `.github/workflows/deploy-docs.yml`: New GitHub Actions workflow.
- No changes to `src/`, `Cargo.toml`, `pixi.toml`, or any existing files.
