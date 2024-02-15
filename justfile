test:
  cargo run --release -- -s testseq -m 3 -g 5

testb:
  cargo run --release -- -s banana -m 3 -g 5

test-alys:
  cargo run --release -- -f test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 1000000 -g 20

perftest:
  sudo perf record -g target/release/iupacpal -f test_data/alys.fna -s NZ_CP059564.1 -m 3 -M 1000000 -g 20
  sudo perf report