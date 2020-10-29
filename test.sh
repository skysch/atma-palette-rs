#!/bin/sh
# Use like this to silence output:
# ./test.sh 2>/dev/null

# Exit on error.
set -e

# Setup new empty palette.
cargo run -- new palette test.atma-palette --set-active --overwrite;



cargo run -- insert '#000';
cargo run -- insert 'ramp(14, blend(:0.0.0, :0.0.15))';
cargo run -- insert '#FFF' --at ':0.0.15';



# Dump palette to screen.
cargo run -- list;
