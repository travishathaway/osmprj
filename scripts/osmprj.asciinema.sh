#!/usr/bin/env bash
set -e

run() {
  printf '$ %s\n' "$1"
  sleep "${2:-0.5}"
  eval "$1"
  printf '\n'
  sleep "${3:-1}"
}

say() {
  printf '$ %s\n\n' "$1"
  sleep "${2:-0.5}"
}

say "# Welcome to the osmprj demo!" 2
say "# All projects start by creating an osmprj.toml file" 2

run "osmprj init --db postgresql://postgres@localhost:65432/postgres" 0.5 1

say "# Data sources are added with osmprj add <source>" 2
run "osmprj add monaco" 0.5 0.5

say "# The --theme option lets you choose various schema layouts" 2
run "osmprj add bremen --theme pgosm" 0.5 1

say "# This all gets saved to your osmprj.toml" 2
run "bat -p osmprj.toml" 0.5 2

say "# To download and import data, run osmprj sync" 2
run "osmprj sync" 0.5 1

say "# To update your database, run osmprj sync again" 2
run "osmprj sync" 0.5 1

run "cowsay 'Now go have fun with your data!'" 0.5 2
