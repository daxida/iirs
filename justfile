# Clean trash files
clean:
  rm -rf iirs.out
  rm -f rand.fasta
  rm -f IUPACpal.out
  rm -f perf.data*
  rm -rf tmp

# Test input.fasta - testseq
test:
  cargo run --release -- \
    -s testseq -m 3 -g 5

# Test input.fasta - banana
testbanana:
  cargo run --release -- \
    -s banana -m 3 -g 5

# Test input.fasta - star
teststar:
  cargo run --release -- \
    -s tstar -m 2 -g 5 -F csv

# Test truncation edge case
testedge:
  cargo run --release -- \
    -f tests/test_data/truncation_edge_case.fasta -m 8 -M 100 -g 10 -x 6

# Test ALL_SEQUENCES with --output-file
testallseq:
  mkdir tmp
  cargo run --release -- \
    -s ALL_SEQUENCES -m 3 -g 5 -o "tmp"

# Test ALL_SEQUENCES with --output-file when only one sequence is present
testallseqone:
  cargo run --release -- \
    -f tests/test_data/test3.fasta -s ALL_SEQUENCES -m 3 -g 5 -o "tmp"

# Test alys
testalys *features:
  cargo run --release --features '{{ features }}' -- \
    -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20 -q

# Perf test for alys (sequential)
testalys-perf *features:
  cargo build --profile=release-with-debug --features '{{ features }}'
  sudo perf record -g "target/release-with-debug/iirs" -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20
  sudo perf report

# Test full N (stress test the algorithm and not the writing)
testn *features:
  cargo run --release --features '{{ features }}' -- \
    -f tests/test_data/200000N.fasta -m 2 -M 100 -g 20 -x 1

BENCH_RUN := "cargo run --release --quiet --manifest-path 'bench/Cargo.toml'"

# Build with features
build *features:
  cargo build --release --features '{{ features }}'

# Write results.csv
compare *features: (build features)
  {{ BENCH_RUN }} --bin logs -- --write

# Test that the results of the rust / cpp binaries are the same
compare-correct *features: (build features)
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 1000 20
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 5000 10
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 20000 5

# Test how the results of the rust / cpp binaries perform
compare-performance *features: (build features)
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 1000000 1

# Does not play nice with parallel
# Make a heatmap
heatmap *features: (build features)
  {{ BENCH_RUN }} --bin logs -- --write
  python3 bench/heatmaps.py

# Bench against alys
bench *features: (build features)
  {{ BENCH_RUN }} --bin bench
