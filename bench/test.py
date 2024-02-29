import os
import random
import subprocess
from argparse import ArgumentParser
from statistics import mean
from time import time

parser = ArgumentParser()
parser.add_argument("--size", type=int, help="Random fasta size")
parser.add_argument("--ntests", type=int, help="Number of tests")
args = parser.parse_args()

FILE_NAME = "rand.fasta"
MAX_GAP = 100
MISMATCHES = 2

GREEN = "\033[32m"
CYAN = "\033[36m"
MAGENTA = "\033[35m"
RESET = "\033[0m"

CPP_TIMINGS = []
RUST_TIMINGS = []


def generate_random_fasta(size: int) -> str:
    return f">seq0\n{''.join(random.choices('acgturyswkmbdhvn*-', k=size))}"


def run(cmd_beginning: str, language: str):
    start = time()

    command = f"{cmd_beginning} -f {FILE_NAME} -g {MAX_GAP} -x {MISMATCHES}"
    output = subprocess.run(command, shell=True, capture_output=True)
    stdout = output.stdout.decode()
    stderr = output.stderr.decode()

    if "panic" in stderr:
        print(f"Stderr: {stderr}")
        # exit(0)
    # print(stderr)
    # print(stdout)
    if "Error" in stdout:
        print(f"Error: {stdout}")
        exit(0)

    elapsed = round(time() - start, 4)
    if language == "RUST":
        RUST_TIMINGS.append(elapsed)
    else:
        CPP_TIMINGS.append(elapsed)

    # print(f"{language: <4} took {elapsed}")


def test_equality():
    with open(os.path.join("IUPACpal.out"), "r") as f:
        expected = f.read().strip()

    with open(os.path.join("IUPACpalrs.out"), "r") as f:
        received = f.read().strip()

    expected = expected.replace("Palindromes:\n", "")
    received = received.replace("Palindromes:\n", "")
    expected_lines = expected.split("\n\n")[1:]  # remove header
    received_lines = received.split("\n\n")[1:]

    # def block_sort(block: str):
    #     if len(block.strip()) == 0:  # empty lines
    #         return (1e9,)
    #     fst, scd, thd = block.split("\n")
    #     a, b, c = fst.split()
    #     d, e, f = thd.split()
    #     return (int(a), int(c), int(d), int(f))

    # expected_lines.sort(key=block_sort)
    # received_lines.sort(key=block_sort)

    # Line by line
    for el, rl in zip(expected_lines, received_lines):
        assert el == rl, f"Received line:\n{rl}\nbut expected:\n{el}"

    assert len(expected_lines) == len(received_lines), (
        len(expected_lines),
        len(received_lines),
    )

    print(f"{GREEN}OK{RESET}: Compared {len(expected_lines) - 1} Palindromes")


def run_tests():
    start = time()

    size_fasta = int(1e4)
    n_tests = 10

    if args.size:
        size_fasta = args.size
    if args.ntests:
        n_tests = args.ntests

    # Compile rust binary
    _ = subprocess.run("cargo build --release", shell=True, capture_output=True)

    for _ in range(n_tests):
        fasta = generate_random_fasta(size_fasta)
        with open(os.path.join(FILE_NAME), "w") as f:
            f.write(fasta)

        run("IUPACpal/IUPACpal", language="CPP")
        run("target/release/main", language="RUST")

        test_equality()

    print(f"Results for {n_tests} random tests of size {size_fasta}")
    print(f"cpp  average: {mean(CPP_TIMINGS)}")
    print(f"rust average: {mean(RUST_TIMINGS)}")

    print(f"\nAll tests finished in {time() - start}")


run_tests()

"""
10 x 1e6 (DEFAULT & -x 2)

cpp average: 30.64037811756134
rust average: 14.46787075996399

-- example for one
Elapsed time: 9 milliseconds (PRECOMP)
Elapsed time: 5799 milliseconds (END PALINDROMES)
Found n=11529468 palindromes
Search complete!
Elapsed time: 13807 milliseconds (TOTAL)
-- 1.1 GB .out file

-- RUST ONLY
10 x 1e6 (DEFAULT & -x 2 -F csv)
rust average: 9.428081846237182
"""
