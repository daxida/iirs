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

# perf test for banana
ptestb:
  cargo build --profile=release-with-debug
  sudo perf record -g target/release-with-debug/iupacpal -s banana -m 3 -g 5
  sudo perf report

# test alys
testalys:
  cargo run --release -- \
    -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20

# perf test for alys
ptestalys:
  cargo build --profile=release-with-debug
  sudo perf record -g target/release-with-debug/iupacpal -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20
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
  sudo perf record -g target/release-with-debug/iupacpal -f tests/test_data/rand10000000.fasta -m 5 -M 100 -g 10 -x 2
  sudo perf report

# test that the results of the rust / cpp binaries are the same
pytest-correct:
  python3 etc/test.py --size 1_000 --ntests 20
  python3 etc/test.py --size 5_000 --ntests 10
  python3 etc/test.py --size 20_000 --ntests 5

# test the performance in both binaries
pytest-performance:
  python3 etc/test.py --size 1_000_000 --ntests 1

logs:
  cargo build --release
  cargo run --release --bin logs

printlogs:
  cargo build --release
  cargo run --release --bin logs
  python3 bench/heatmaps.py

bench:
  cargo build --release
  cargo run --release --bin bench