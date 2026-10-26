#!/bin/sh
# The case's adapter: writes the case's raw material into the raw place it
# is handed and prints the case's lines, as a run of a release adapter
# would, then exits reporting success.
set -eu
here=$(dirname "$0")
raw_dir=
while [ $# -gt 0 ]; do
  case $1 in
  --raw-dir)
    raw_dir=$2
    shift 2
    ;;
  --cursor) shift 2 ;;
  *)
    echo "adapter.sh: $1 is no flag of a run" >&2
    exit 2
    ;;
  esac
done
if [ -z "$raw_dir" ]; then
  echo "adapter.sh: no raw place was handed" >&2
  exit 2
fi
cp -R "$here/raw/." "$raw_dir/"
cat "$here/lines.jsonl"
