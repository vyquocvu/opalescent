#!/usr/bin/env bash
# Compares compile time, run time and peak RSS of Opalescent vs Rust vs Ruby.
# Usage: benchmarks/run.sh   (run from repo root; needs rustc, ruby, python3)
# Env: REPS (default 5) sets timed repetitions; one untimed warmup always runs.
set -euo pipefail
cd "$(dirname "$0")/.."

REPS=${REPS:-5}
cargo build --release --quiet
OPAL=target/release/opalescent
[ -x "$OPAL" ] || OPAL=target/release/opal
OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT

if [[ "$(uname)" == "Darwin" ]]; then
  rss() { /usr/bin/time -l "$@" 2>&1 >/dev/null | awk '/maximum resident/ {printf "%.3f", $1/1048576}'; }
else
  rss() { /usr/bin/time -f '%M' "$@" 2>&1 >/dev/null | awk '{printf "%.3f", $1/1024}'; }
fi
now() { python3 -c 'import time; print(time.time())'; }
elapsed() { python3 -c "print(f'{$2 - $1:.6f}')"; }
summarize() {
  python3 -c '
import statistics, sys
values = [float(v) for v in sys.argv[1:]]
print(f"{statistics.mean(values):.3f} +/- {statistics.pstdev(values):.3f}")
' "$@"
}

# One untimed warmup, then REPS timed runs.
time_stats() {
  "$@" >/dev/null
  local -a samples=()
  for ((i = 0; i < REPS; i++)); do
    local a b
    a=$(now)
    "$@" >/dev/null
    b=$(now)
    samples+=("$(elapsed "$a" "$b")")
  done
  summarize "${samples[@]}"
}

rss_stats() {
  local -a samples=()
  for ((i = 0; i < REPS; i++)); do
    samples+=("$(rss "$@")")
  done
  summarize "${samples[@]}"
}

opal_compile=$(time_stats "$OPAL" build benchmarks/collatz.opal -o "$OUT/opal_bin")
rust_compile=$(time_stats rustc --edition=2021 -C opt-level=3 benchmarks/collatz.rs -o "$OUT/rust_bin")

printf '%-8s %-18s %-18s %-18s %s\n' impl 'compile_s (sd)' 'run_s (sd)' 'rss_MB (sd)' output
for impl in opal rust ruby; do
  case $impl in
    opal) cmd=("$OUT/opal_bin"); c=$opal_compile ;;
    rust) cmd=("$OUT/rust_bin"); c=$rust_compile ;;
    ruby) cmd=(ruby benchmarks/collatz.rb); c="n/a" ;;
  esac
  printf '%-8s %-18s %-18s %-18s %s\n' \
    "$impl" "$c" "$(time_stats "${cmd[@]}")" "$(rss_stats "${cmd[@]}")" "$("${cmd[@]}")"
done
