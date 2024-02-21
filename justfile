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
  sudo perf record -g target/debug/iupacpal -s banana -m 3 -g 5
  sudo perf report

# test alys
testalys:
  cargo run --release -- \
    -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20

# perf test for alys
ptestalys:
  cargo build --profile=release-with-debug
  sudo perf record -g target/debug/main -f tests/test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20
  sudo perf report

# test for randIUPAC10000
testrand:
  cargo run --profile=release-with-debug -- \
    -f tests/test_data/randIUPAC10000.fasta -m 3 -M 100 -g 0 -x 0

# perf test for randIUPAC10000
ptestrand:
  cargo build --profile=release-with-debug
  sudo perf record -g target/debug/main -f tests/test_data/randIUPAC10000.fasta -m 3 -M 100 -g 0 -x 0
  sudo perf report

# test that the results of the rust / cpp binaries are the same
pytest-correct:
  python3 etc/test.py --size 1_000 --ntests 20
  python3 etc/test.py --size 5_000 --ntests 10
  python3 etc/test.py --size 20_000 --ntests 5

# test the performance in both binaries
pytest-performance:
  python3 etc/test.py --size 1_000_000 --ntests 1

bench-correct:
  cargo build --release
  cargo build --release --bin bench
  ./target/release/bench --size-fasta 1000 --n-tests 20 -g 100 -x 2
  ./target/release/bench --size-fasta 5000 --n-tests 10 -g 100 -x 2
  ./target/release/bench --size-fasta 20000 --n-tests 5 -g 100 -x 2