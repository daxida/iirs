import random
from time import time
import os
from test_equality import test_equality
import subprocess
from statistics import mean

IUPAC_SYMBOLS = "acgturyswkmbdhvn*-"
FILE_NAME = "rand.fasta"
MAX_GAP = 10
CPP_FOLDER = "IUPACpal"
# RUST_FOLDER = "iupacpal-rust/target/debug" # For compiled binary
RUST_FOLDER = "iupacpal-rust" # For cargo

def generate(size: int) -> str:
    seq = ''.join(random.choices(IUPAC_SYMBOLS, k=size))
    return f">seq0\n{seq}"

def write_fasta(fasta: str):
    with open(f"{CPP_FOLDER}/{FILE_NAME}", "w") as f:
        f.write(fasta)
    with open(f"{RUST_FOLDER}/{FILE_NAME}", "w") as f:
        f.write(fasta)

def run(folder: str, cmd_beginning:str):
    start = time()
    os.chdir(folder)
    command = f"{cmd_beginning} -f {FILE_NAME} -g {MAX_GAP}"
    output = subprocess.run(command, shell=True, capture_output=True)
    stdout = output.stdout.decode()
    stderr = output.stderr.decode()
    # if stderr:
    #     print(f"Stderr: {stderr}")
    #     exit(0)
    if "Error" in stdout:
        print(f"Error: {stdout}")
        exit(0)
    original_folder = "/".join([".."] * (1 + folder.count("/")))
    os.chdir(original_folder)
    elapsed = time() - start
    if "rust" in folder:
        binary = "rust"
        rust_timings.append(elapsed)
    else:
        binary = "cpp"
        cpp_timings.append(elapsed)
    print(f"{binary} took {elapsed}")

def test(n: int):
    fasta = generate(n)
    write_fasta(fasta)
    run(CPP_FOLDER, "./IUPACpal")
    run(RUST_FOLDER, "cargo run --")
    # run(RUST_FOLDER, "./iupacpal")
    test_equality(CPP_FOLDER, RUST_FOLDER)

cpp_timings = []
rust_timings = []
for _ in range(10):
    test(int(1e5))

cpp_avg_time = mean(cpp_timings)
rust_avg_time = mean(rust_timings)
print(f"cpp average: {cpp_avg_time}")
print(f"rust average: {rust_avg_time}")