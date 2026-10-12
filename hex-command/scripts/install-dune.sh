#!/usr/bin/env bash
# Build the pinned Dune from its checksummed upstream release.
set -euo pipefail
project=$(cd "$(dirname "$0")/.." && pwd)
version=$(cat "$project/.dune-version")
install_root=${1:-"$HOME/.local"}
if [ -x "$install_root/bin/dune" ] && [ "$("$install_root/bin/dune" --version)" = "$version" ]; then
  exit 0
fi
command -v ocaml >/dev/null || { echo 'Dune bootstrap requires OCaml 4.14+ on PATH (or install Dune with opam).' >&2; exit 1; }
python3 - "$(ocamlc -version)" <<'PY'
import sys
if tuple(map(int, sys.argv[1].split('.')[:2])) < (4, 14):
    raise SystemExit('Dune bootstrap requires OCaml 4.14+')
PY
cache_root=${XDG_CACHE_HOME:-"$HOME/.cache"}/dune-bootstrap
mkdir -p "$cache_root"
archive="$cache_root/dune-$version.tbz"
if [ ! -f "$archive" ]; then
  curl --proto '=https' --tlsv1.2 -fsSL "https://github.com/ocaml/dune/releases/download/$version/dune-$version.tbz" -o "$archive"
fi
python3 - "$archive" <<'PY'
import hashlib, pathlib, sys
expected = '472798691b0216daf538709f0f4703b3617ef24ad0866c9096068baaba4d762a'
if hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest() != expected:
    raise SystemExit('Dune upstream archive checksum mismatch')
PY
tar -xjf "$archive" -C "$cache_root"
cd "$cache_root/dune-$version"
ocaml boot/bootstrap.ml -j "${DUNE_BUILD_JOBS:-4}"
mkdir -p "$install_root/bin"
cp _boot/dune.exe "$install_root/bin/dune"
chmod 755 "$install_root/bin/dune"
"$install_root/bin/dune" --version
