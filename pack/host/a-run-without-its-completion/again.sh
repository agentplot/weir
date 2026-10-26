#!/bin/sh
# The case's adapter's next run: refuses a cursor, since the run before
# it was incomplete, then prints the five offers again with its
# completion and exits reporting success.
set -eu
here=$(dirname "$0")
while [ $# -gt 0 ]; do
  case $1 in
  --raw-dir) shift 2 ;;
  --cursor)
    echo "again.sh: handed the cursor $2, and the last complete run kept none" >&2
    exit 3
    ;;
  *)
    echo "again.sh: $1 is no flag of a run" >&2
    exit 2
    ;;
  esac
done
cat "$here/again.jsonl"
exit 0
