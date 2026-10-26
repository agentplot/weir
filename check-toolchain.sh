#!/bin/sh
# Every Rust tool the gate runs is the build .prototools pins, on every
# host (core.40). The pin is read from .prototools, the one file that names
# Rust's version, and rustc, cargo, clippy-driver and rustfmt on PATH must
# each be that build. A tool missing, or found in another toolchain, a
# runner's rustup default say, fails here, named with what was found,
# rather than holding the tree to another version's rules.
set -eu

fail() {
  echo "check-toolchain: $*" >&2
  exit 1
}

pin=$(sed -n 's/^rust *= *"\(.*\)"$/\1/p' .prototools)
[ -n "$pin" ] || fail ".prototools pins no rust"

# A tool the pinned toolchain lacks, with the component that brings it.
missing() {
  fail "$1 is missing: add it to the pinned toolchain, rustup component add --toolchain $pin $2"
}

# Where PATH finds a tool, or missing.
on_path() {
  command -v "$1" 2>/dev/null || missing "$1" "$2"
}

rustc_path=$(on_path rustc rustc)
rustc=$(rustc -vV 2>/dev/null) || missing rustc rustc
release=$(printf '%s\n' "$rustc" | sed -n 's/^release: //p')
[ "$release" = "$pin" ] || fail "rustc at $rustc_path is ${release:-no release}, not $pin"

# The pinned toolchain's own tools lie beside its rustc: one it lacks is
# missing, whatever a later PATH entry offers in its place.
bin="$(rustc --print sysroot)/bin"
for tool in cargo:cargo clippy-driver:clippy rustfmt:rustfmt; do
  [ -x "$bin/${tool%%:*}" ] || missing "${tool%%:*}" "${tool#*:}"
done

cargo_path=$(on_path cargo cargo)
release=$(cargo -vV 2>/dev/null | sed -n 's/^release: //p')
[ "$release" = "$pin" ] || fail "cargo at $cargo_path is ${release:-no release}, not $pin"

# clippy-driver and rustfmt carry the commit of the rustc they ship with.
commit=$(printf '%s\n' "$rustc" | sed -n 's/^commit-hash: \(.\{9\}\).*/\1/p')
same() {
  path=$(on_path "$1" "$2")
  version=$("$1" --version 2>/dev/null) || missing "$1" "$2"
  case $version in
  *"($commit"*) ;;
  *) fail "$1 at $path is $version, not $pin's" ;;
  esac
}
same clippy-driver clippy
same rustfmt rustfmt
echo "rustc, cargo, clippy-driver and rustfmt are $pin"
