clean:
  rm -f *iirs.out
  rm -f rand.fasta
  rm -f IUPACpal.out
  rm -f perf.data*
  
# Simple tests over input.fasta
test:
  cargo run --release -- \
    -s testseq -m 3 -g 5

testbanana:
  cargo run --release -- \
    -s banana -m 3 -g 5

teststar:
  cargo run --release -- \
    -s tstar -m 2 -g 5 -F csv

# Test edge case
testedge:
  cargo run --release -- \
    -f tests/test_data/truncation_edge_case.fasta -m 8 -M 100 -g 10 -x 6

MAIN_RUN := "cargo run --quiet --release"
MAIN_RUN_PARALLEL := "cargo run --quiet --release --features parallel"
MAIN_RUN_PARALLEL_TABULATION := "cargo run --quiet --release --features 'parallel tabulation'"
BINARY_RELEASE_WITH_DEBUG := "target/release-with-debug/iirs"

# Perf test for banana
testbanana-perf:
  cargo build --profile=release-with-debug
  sudo perf record -g {{ BINARY_RELEASE_WITH_DEBUG }} -s banana -m 3 -g 5
  sudo perf report

# Test alys
# `just testalys` will run the sequential version
# `just testalys par` will run the parallel version
# `just testalys partab` will run the parallel + tabulation version
testalys arg="":
  {{ if arg == "par" \
    { MAIN_RUN_PARALLEL } \
  else if arg == "partab" \
    { MAIN_RUN_PARALLEL_TABULATION} \
  else { MAIN_RUN } }} -- \
    -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20 -q

# Perf test for alys (sequential)
testalys-perf:
  cargo build --profile=release-with-debug
  sudo perf record -g {{ BINARY_RELEASE_WITH_DEBUG }} -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20
  sudo perf report

# Test full N (stress test the algorithm and not the writing)
testn:
  cargo run --release -- \
    -f tests/test_data/200000N.fasta -m 2 -M 100 -g 20 -x 1

# Test for rand1000000 (1e6)
testrand arg="":
  {{ if arg == "par" { MAIN_RUN_PARALLEL } else { MAIN_RUN } }} -- \
    -f tests/test_data/rand1000000.fasta -m 5 -M 100 -g 10 -x 2

# Perf test for rand1000000
testrand-perf:
  cargo build --profile=release-with-debug
  sudo perf record -g {{ BINARY_RELEASE_WITH_DEBUG }} -f tests/test_data/rand1000000.fasta -m 5 -M 100 -g 10 -x 2
  sudo perf report

MAIN_BUILD := "cargo build --quiet --release"
MAIN_BUILD_PARALLEL := "cargo build --quiet --release --features parallel"
BENCH_RUN := "cargo run --quiet --manifest-path 'bench/Cargo.toml' --release"

# Write results.csv
compare arg="":
  {{ if arg == "par" { MAIN_BUILD_PARALLEL } else { MAIN_BUILD } }} 
  {{ BENCH_RUN }} --bin logs -- --write

# Test that the results of the rust / cpp binaries are the same
compare-correct arg="":
  {{ if arg == "par" { MAIN_BUILD_PARALLEL } else { MAIN_BUILD } }} 
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 1000 20
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 5000 10
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 20000 5

# Test how the results of the rust / cpp binaries perform
compare-performance arg="":
  {{ if arg == "par" { MAIN_BUILD_PARALLEL } else { MAIN_BUILD } }}
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 1000000 1

printlogs:
  {{ MAIN_BUILD }}
  {{ BENCH_RUN }} --bin logs -- --write
  python3 bench/heatmaps.py

bench arg="":
  {{ if arg == "par" { MAIN_BUILD_PARALLEL } else { MAIN_BUILD } }} 
  {{ BENCH_RUN }} --bin bench