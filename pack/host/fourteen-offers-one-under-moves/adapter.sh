#!/bin/sh
# The case's adapter: prints the case's fourteen offers, its cursor and its
# completion, and exits reporting success.
set -eu
here=$(dirname "$0")
while [ $# -gt 0 ]; do
  case $1 in
  --raw-dir) shift 2 ;;
  --cursor) shift 2 ;;
  *)
    echo "adapter.sh: $1 is no flag of a run" >&2
    exit 2
    ;;
  esac
done
cat "$here/lines.jsonl"
exit 0
