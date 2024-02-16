# test testseq in input.fasta
test:
  cargo run --release -- -s testseq -m 3 -g 5

# test banana in input.fasta
testb:
  cargo run --release -- -s banana -m 3 -g 5

# perf test for banana
ptestb:
  cargo build
  sudo perf record -g target/debug/iupacpal -s banana -m 3 -g 5
  sudo perf report

# test alys
testalys:
  cargo run --release -- -f test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20

# perf test for alys
ptestalys:
  cargo build
  sudo perf record -g target/debug/iupacpal -f test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 100 -g 20
  sudo perf report

# perf test for randIUPAC1000000
ptestrand:
  cargo build
  sudo perf record -g target/debug/iupacpal -f test_data/randIUPAC1000000.fasta -m 3 -M 100 -g 20
  sudo perf report

# test that the results of the rust / cpp binaries are the same
pytest-correct:
  python3 etc/test.py --size 5_000 --ntests 10

# test the performance in both binaries
pytest-performance:
  python3 etc/test.py --size 1_000_000 --ntests 1

tall:
  cargo run --release -- -s ALL -m 3 -g 5 -F custom_csv_mini

tmul:
  cargo run --release -- -f test_data/test_multiple.fna -s ALL -m 3 -g 5 -F custom_csv_mini