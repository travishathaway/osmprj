#! /bin/bash

# Build Rust binary
cargo auditable install --locked --no-track --bins --root "$PREFIX" --path .
cargo-bundle-licenses --format yaml --output ./THIRDPARTY.yml

# Move default themes to themes directory
mkdir -p "$PREFIX/share/osmprj/"
cp -r themes "$PREFIX/share/osmprj/"

# Setup environment variables
mkdir -p $PREFIX/etc/conda/env_vars.d
cat > $PREFIX/etc/conda/env_vars.d/osmprj.json << EOF
{
  "OSMPRJ_THEME_PATH": "$PREFIX/share/osmprj/themes/"
}
EOF
