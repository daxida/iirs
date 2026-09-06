# iirs [![Crates.io](https://img.shields.io/crates/v/iirs.svg)](https://crates.io/crates/iirs)

IIRS is an [Iupac](https://en.wikipedia.org/wiki/International_Union_of_Pure_and_Applied_Chemistry) Inverted RepeatS finder, ported to rust from [IUPACpal](https://github.com/steven31415/IUPACpal), result of this [paper](https://www.researchgate.net/publication/349110200_IUPACpal_efficient_identification_of_inverted_repeats_in_IUPAC-encoded_DNA_sequences).

That is, an exact tool for efficient identification of Inverted Repeats ([IRs](https://en.wikipedia.org/wiki/Inverted_repeat)) in IUPAC-encoded DNA sequences, allowing also for potential mismatches and gaps.

On top of inverted repeats, it can also find the three other kinds of repeat obtained by reading the second arm backwards or not, and complementing it or not:

| `--repeat-type` | Second arm | Example |
| --- | --- | --- |
| `inverted` (default) | reverse complement | `aacc` ... `ggtt` |
| `mirror` | reverse | `aacc` ... `ccaa` |
| `direct` | as is | `aacc` ... `aacc` |
| `complement` | complement | `aacc` ... `ttgg` |

Compared to the original, this version is faster, platform-independent and modular, facilitating the creation of customized format outputs.

A short introduction to the [algorithm](https://github.com/daxida/iirs/blob/master/docs/algorithm.md) can be found in the docs folder.

## Installation

You can either build from source:

```console
$ cargo install iirs
```

Or download the latest binary from [releases](https://github.com/daxida/iirs/releases) and extract it somewhere on your `$PATH`.

## Usage

The command line shares much of the functionality of the original IUPACpal. The notable differences are:
- Support for multiple sequence names.
- `ALL_SEQUENCES` argument for processing all the sequences in the input file.
- Output format.

 You can always run `iirs --help` for a full description.

```console
// Scan sequences t1 and t2 in the file input.fasta, with csv output format
$ iirs -f input.fasta -s 't1 t2' -g 5 -F csv

// Same command as above, with long flags for clarity
$ iirs -f input.fasta --seq-names 't1 t2' --max-gap 5 --output-format csv

// Scan all sequences of the fasta file
$ iirs -f input.fasta -s ALL_SEQUENCES -g 5 -m 3 -F csv

// Look for direct repeats instead of inverted ones
$ iirs -f input.fasta -g 5 -m 3 -r direct
```

Many more practical examples can be found in the [justfile](https://github.com/casey/just).

## Features

The default uses a linear space block decomposition for the range minimum query (rmq), and it is sequential over IR centers. To change this behaviour you can use the features `tabulation` (to change the rmq implementation), `parallel` (to run in parallel over IR centers) or a combination of both. This may result in a significant speed increase:

```console
$ cargo install iirs --features "parallel tabulation"
```

## Library

iirs can also be used as a library, both in rust and in python. Both are minimal: a `SearchParams` struct / class that does some bound checking, and a `find_repeats` function.

### Rust

```console
$ cargo add iirs [--features X]
```

```rust
use iirs::{RepeatType, SearchParams, find_repeats};

let params = SearchParams::new(3, 6, 2, 0)?;
assert_eq!(find_repeats(&params, b"acbbgt")?, vec![(0, 5, 0)]);

// The other three kinds are selected with `with_repeat_type`
let params = params.with_repeat_type(RepeatType::Direct);
assert_eq!(find_repeats(&params, b"acgacg")?, vec![(0, 5, 0)]);
```

### Python

After cloning the repo (no wheels yet):

```console
$ pip install py-iirs/
```

```python
from iirs import SearchParams, find_repeats

params = SearchParams(min_len=3, max_len=6, max_gap=2, mismatches=0)
assert find_repeats(params, "acbbgt") == [(0, 5, 0)]

# The other three kinds are selected with the `repeat_type` argument, one of
# "inverted" (the default), "mirror", "direct" or "complement"
params = SearchParams(3, 6, 2, 0, repeat_type="direct")
assert find_repeats(params, "acgacg") == [(0, 5, 0)]

start, end, gap = find_repeats(params, "acgacg")[0]
arm_len = (end + 1 - start - gap) // 2
assert ("acgacg"[start:start + arm_len], "acgacg"[end + 1 - arm_len:end + 1]) == ("acg", "acg")
```

## Testing

- `cargo test` for unit tests.
- `just testpy` for the python bindings. It installs `py-iirs/` (which builds the rust
  side through maturin) together with pytest, then runs `pytest py-iirs/tests`.
- `bench.rs` benches against a single file. To use together with `just bench` after modifying the parameters in `bench.rs`. To test against different features you can add them as arguments: `just bench parallel` or `just bench parallel tabulation`.
- `logs.rs` benches against the cpp binary. You will need a [IUPACpal](https://github.com/steven31415/IUPACpal) binary (and they only support Linux). The binary is expected to be in the bench folder, but that can be changed in `logs.rs` and `validate.py`.
- Note that `just heatmap` requires the python libraries listed in `bench/requirements.txt`.
