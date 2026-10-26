#!/bin/sh
# The case's adapter's first run: prints five offers and a cursor, and
# exits reporting failure, with no completion.
set -eu
here=$(dirname "$0")
while [ $# -gt 0 ]; do
  case $1 in
  --raw-dir) shift 2 ;;
  --cursor) shift 2 ;;
  *)
    echo "first.sh: $1 is no flag of a run" >&2
    exit 2
    ;;
  esac
done
cat "$here/first.jsonl"
exit 1
