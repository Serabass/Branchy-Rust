#!/bin/sh
# Run all .branchy examples randomly, several times each, collect compile/run errors.
cd /app
cargo build 2>&1
BIN="/cache/target/debug/branchy"
RUNS=3
FAILS=""
for f in examples/*.branchy; do
  for i in $(seq 1 $RUNS); do
    if ! out=$($BIN run "$f" 2>&1); then
      FAILS="${FAILS}\n=== FAIL: $f (run $i) ===\n$out\n"
    fi
  done
done
# Shuffle and run again (different order)
for i in $(seq 1 2); do
  for f in $(find examples -name "*.branchy" | shuf); do
    if ! out=$($BIN run "$f" 2>&1); then
      FAILS="${FAILS}\n=== FAIL: $f (shuffled run $i) ===\n$out\n"
    fi
  done
done
if [ -n "$FAILS" ]; then
  printf "ERRORS FOUND:%s\n" "$FAILS"
  exit 1
fi
echo "All example runs OK (no compilation errors)."
