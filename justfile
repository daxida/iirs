clean:
  rm -f *iirs.out
  rm -f rand.fasta
  rm -f IUPACpal.out
  
# test testseq in input.fasta
test:
  cargo run --release -- \
    -s testseq -m 3 -g 5

# test banana in input.fasta
testb:
  cargo run --release -- \
    -s banana -m 3 -g 5

# test MCHU in input.fasta
testm:
  cargo run --release -- \
    -s MCHU -m 3 -g 5 -F csv

testt:
  cargo run --release -- \
    -s t2 -m 2 -g 5 -F csv

teststar:
  cargo run --release -- \
    -s tstar -m 2 -g 5 -F csv

testedge:
  cargo run --release -- \
    -f tests/test_data/truncation_edge_case.fasta -m 8 -M 100 -g 10 -x 6

BINARY_RELEASE_WITH_DEBUG := "target/release-with-debug/iirs"

# perf test for banana
ptestb:
  cargo build --profile=release-with-debug
  sudo perf record -g {{ BINARY_RELEASE_WITH_DEBUG }} -s banana -m 3 -g 5
  sudo perf report

# test alys
testalys:
  cargo run --release -- \
    -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20

# perf test for alys
ptestalys:
  cargo build --profile=release-with-debug
  sudo perf record -g {{ BINARY_RELEASE_WITH_DEBUG }} -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20
  sudo perf report

# test full N (stress test the algorithm and not the writing)
testn:
  cargo run --release -- \
    -f tests/test_data/200000N.fasta -m 2 -M 100 -g 20 -x 1

# test for rand10000000 (1e7)
testrand:
  cargo run --release -- \
    -f tests/test_data/rand10000000.fasta -m 5 -M 100 -g 10 -x 2

# perf test for rand10000000
ptestrand:
  cargo build --profile=release-with-debug
  sudo perf record -g {{ BINARY_RELEASE_WITH_DEBUG }} -f tests/test_data/rand10000000.fasta -m 5 -M 100 -g 10 -x 2
  sudo perf report

BENCH_RUN := "cargo run --quiet --manifest-path 'bench/Cargo.toml' --release"

# write results.csv
compare:
  cargo build --quiet --release 
  {{ BENCH_RUN }} --bin logs -- --write

# test that the results of the rust / cpp binaries are the same
compare-correct:
  cargo build --quiet --release
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 1000 20
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 5000 10
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 20000 5

# test how the results of the rust / cpp binaries perform
compare-performance:
  cargo build --quiet --release
  {{ BENCH_RUN }} --bin logs -- --verbose --random-bench 1000000 1

printlogs:
  cargo build --release
  {{ BENCH_RUN }} --bin logs -- --write
  python3 bench/heatmaps.py

bench:
  cargo build --release
  {{ BENCH_RUN }} --bin bench