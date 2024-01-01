import random
from time import time
import os
from test_equality import test_equality
import subprocess

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
    binary = "rust" if "rust" in folder else "cpp"
    print(f"{binary} took {elapsed}")

def test(n: int):
    fasta = generate(n)
    write_fasta(fasta)
    run(CPP_FOLDER, "./IUPACpal")
    run(RUST_FOLDER, "cargo run --")
    # run(RUST_FOLDER, "./iupacpal")
    test_equality(CPP_FOLDER, RUST_FOLDER)

for _ in range(1):
    test(int(1e5))