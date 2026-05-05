## 1. Bootstrap Docusaurus Project

- [x] 1.1 Create `docs/` directory at the repo root
- [x] 1.2 Create `docs/package.json` with `@docusaurus/core`, `@docusaurus/preset-classic`, `asciinema-player`, and TypeScript dev dependencies; set `name` to `osmprj-docs` and include `start`, `build`, `serve`, and `deploy` scripts
- [x] 1.3 Create `docs/tsconfig.json` extending `@docusaurus/tsconfig`
- [x] 1.4 Create `docs/babel.config.js` with the standard Docusaurus Babel preset
- [x] 1.5 Create `docs/docusaurus.config.ts` with `url: "https://travishathaway.github.io"`, `baseUrl: "/osmprj/"`, `organizationName: "travishathaway"`, `projectName: "osmprj"`, navbar (logo + Docs link + GitHub link), footer (GitHub link + "Built with Docusaurus"), and `@docusaurus/preset-classic` with docs and theme config
- [x] 1.6 Run `npm install` inside `docs/` to generate `package-lock.json`

## 2. Custom CSS

- [x] 2.1 Create `docs/src/css/custom.css` with CSS variables for Docusaurus `--ifm-color-primary` (use a green/teal palette fitting an OSM/geo tool) and hero section styles (`.hero`, `.hero__title`, `.hero__subtitle`, `.hero__buttons`, `.features`, `.featureCard`)

## 3. Placeholder Cast File

- [x] 3.1 Create `docs/static/demo.cast` as a minimal valid asciinema v2 cast file (header JSON line + a few event lines) so the build succeeds and the player renders without a 404; add a comment at the top noting it is a placeholder to be replaced

## 4. Landing Page

- [x] 4.1 Create `docs/src/pages/index.tsx` with:
  - Hero section: title "osmprj", tagline "A friendly, modern tool for managing OSM data with PostgreSQL", two buttons ("Get Started" → `/docs/intro`, "GitHub" → repo URL)
  - Feature cards section: three cards — "Simple Workflow", "Auto-Tuned Imports", "Incremental Updates" — each with a short one-sentence description
  - Demo section: heading "See it in action", asciinema player wrapped in `BrowserOnly` from `@docusaurus/core` pointing at `/osmprj/demo.cast`
  - Import `asciinema-player/dist/bundle/player.css` inside `BrowserOnly` callback using a dynamic `require` so it is client-side only
  - Comment noting that `/osmprj/demo.cast` must match `baseUrl + "demo.cast"` if baseUrl ever changes

## 5. Docs Content

- [x] 5.1 Create `docs/docs/intro.md` with front matter `id: intro`, `title: Introduction`, `sidebar_position: 1`; content: what osmprj is, what it wraps (osm2pgsql + Geofabrik + osm2pgsql-replication), experimental warning callout, link to Getting Started
- [x] 5.2 Create `docs/docs/getting-started.md` with front matter `id: getting-started`, `title: Getting Started`, `sidebar_position: 2`; content: installation (pixi global and conda create commands), the 4-step quick start from the README (`init` → `add` → `status` → `sync`), brief explanation of what happens on first vs. subsequent syncs
- [x] 5.3 Create `docs/docs/guides/_category_.json` setting `label: "Guides"` and `position: 3`
- [x] 5.4 Create `docs/docs/guides/managing-sources.md` — stub with front matter and a short intro sentence about `osmprj add` and `osmprj remove`; leave body as `<!-- TODO: expand -->`
- [x] 5.5 Create `docs/docs/guides/syncing-data.md` — stub covering `osmprj sync` and the first-run vs. update distinction; leave body as `<!-- TODO: expand -->`
- [x] 5.6 Create `docs/docs/guides/themes.md` — stub listing available themes and mentioning `OSMPRJ_THEME_PATH`; leave body as `<!-- TODO: expand -->`
- [x] 5.7 Create `docs/docs/guides/configuration.md` — stub with the `osmprj.toml` configuration table from the README; leave detailed field descriptions as `<!-- TODO: expand -->`

## 6. Docker Compose Dev Environment

- [x] 6.1 Create `docs/docker-compose.yaml` with a single `docs` service using the official `node:20-alpine` image, mounting `./` to `/app` inside the container, setting the working directory to `/app`, running `npm install && npm start -- --host 0.0.0.0` as the command, and publishing container port 3000 to host port 3000; add a `stdin_open: true` / `tty: true` so Ctrl-C works cleanly

## 7. GitHub Actions Deploy Workflow

- [x] 7.1 Create `.github/workflows/deploy-docs.yml` that triggers on push to `main`; checks out the repo (with `fetch-depth: 0`); sets up Node 20; runs `npm ci` then `npm run build` in `docs/`; deploys `docs/build/` to the `gh-pages` branch using `peaceiris/actions-gh-pages@v4` with `github_token: ${{ secrets.GITHUB_TOKEN }}` and `publish_dir: ./docs/build`

## 8. Repository Housekeeping

- [x] 8.1 Add `docs/node_modules/` and `docs/build/` to `.gitignore` (create `.gitignore` at repo root if it does not exist, otherwise append)

## 9. Verification

- [x] 9.1 Run `docker compose up` inside `docs/` and confirm the dev server starts and the site is reachable at `http://localhost:3000/osmprj/`
- [x] 9.2 Run `npm run build` inside `docs/` (or via `docker compose run docs npm run build`) and confirm it exits 0 with no errors
- [ ] 9.3 Manually verify the landing page, asciinema player placeholder, and all three docs sections render correctly in a browser
- [ ] 9.4 Confirm the GitHub Actions workflow file is valid YAML (use `actionlint` or push to a branch and check the Actions tab)
