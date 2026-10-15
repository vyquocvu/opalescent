#!/usr/bin/env bash
# Compares compile time, run time and peak RSS of Opalescent vs Rust vs Ruby.
# Usage: benchmarks/run.sh   (run from repo root; needs rustc, ruby)
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --release --quiet
OPAL=target/release/opalescent
[ -x "$OPAL" ] || OPAL=target/release/opal
OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT

if [[ "$(uname)" == "Darwin" ]]; then
  rss() { /usr/bin/time -l "$@" 2>&1 >/dev/null | awk '/maximum resident/ {printf "%.1f", $1/1048576}'; }
else
  rss() { /usr/bin/time -f '%M' "$@" 2>&1 >/dev/null | awk '{printf "%.1f", $1/1024}'; }
fi
now() { python3 -c 'import time; print(time.time())'; }
elapsed() { python3 -c "print(f'{$2 - $1:.3f}')"; }

t0=$(now); "$OPAL" build benchmarks/collatz.opal -o "$OUT/opal_bin" >/dev/null; t1=$(now)
opal_compile=$(elapsed "$t0" "$t1")
t0=$(now); rustc --edition=2021 -C opt-level=3 benchmarks/collatz.rs -o "$OUT/rust_bin"; t1=$(now)
rust_compile=$(elapsed "$t0" "$t1")

run_time() { local a; a=$(now); "$@" >/dev/null; local b; b=$(now); elapsed "$a" "$b"; }

printf '%-10s %-12s %-10s %-10s %s\n' impl compile_s run_s rss_MB output
for impl in opal rust ruby; do
  case $impl in
    opal) cmd=("$OUT/opal_bin"); c=$opal_compile ;;
    rust) cmd=("$OUT/rust_bin"); c=$rust_compile ;;
    ruby) cmd=(ruby benchmarks/collatz.rb); c=0 ;;
  esac
  printf '%-10s %-12s %-10s %-10s %s\n' "$impl" "$c" "$(run_time "${cmd[@]}")" "$(rss "${cmd[@]}")" "$("${cmd[@]}")"
done
