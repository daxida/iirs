# iirs

IIRS is an [Iupac](https://en.wikipedia.org/wiki/International_Union_of_Pure_and_Applied_Chemistry) Inverted RepeatS finder written in rust (rs), ported from [IUPACpal](https://github.com/steven31415/IUPACpal), result of this [paper](https://www.researchgate.net/publication/349110200_IUPACpal_efficient_identification_of_inverted_repeats_in_IUPAC-encoded_DNA_sequences).

That is, an exact tool for efficient identification of Inverted Repeats (IRs) in IUPAC-encoded DNA sequences as substrings of a large text, allowing also for potential mismatches and gaps.

Compared to the original, this version is faster, platform-independent and modular, facilitating the creation of customized format outputs. It does not require `cmake` nor `sdsl`. It uses [divsufsort](https://github.com/fasterthanlime/stringsearch/tree/master/crates/divsufsort) instead of [libdivsufsort](https://github.com/y-256/libdivsufsort).

## How to use the binary

The command line shares much of the functionality of the original IUPACpal. 

Type `iirs --help` for a full description.

The notable differences are:
- Support for multiple sequence names.
- `ALL_SEQUENCES` argument for processing all the sequences in the input file.
- Output format.

```
iirs -f input.fasta -s 't1 t2' -g 5 -F csv
iirs -f input.fasta --seq-names t1 --max-gap 5 --output-format csv
iirs -f input.fasta -s ALL_SEQUENCES -g 5 -m 3 -F csv
```

Many more practical examples can be found in the justfile.

## How to install the binary

You can either build from source:

```
$ cargo install iirs
```

Or download the latest binary from [releases](https://github.com/daxida/iirs/releases) and extract it somewhere on your `$PATH`.

## Features

The default uses a Sparse Table implementation for the range minimum query, and it is sequential over IR centers. To change this behaviour you can use the features `tabulation`, `parallel` or a combination of both. This may result in a significant speed increase:

```
cargo install iirs --features "parallel tabulation"
```

## Extra

It can also be used as a library both in rust and python.

```
cargo add iirs [--features X]
```

Or to python, after cloning the repo, via (no wheels yet):

```
pip install py-iirs/
```

Both libraries are minimal and only contain a struct / class `SearchParams` that does some bound checking, and a `find_irs` function.

## Testing

- `cargo test` for unit tests.
- [Justfile](https://github.com/casey/just) for individual tests against sequences. Some use the Linux profiler [perf](https://en.wikipedia.org/wiki/Perf_(Linux)). To see the full list of commands use `just -l`. 
- `bench.rs` benches against a single file. To use together with `just bench` after modifying the parameters in `bench.rs`. To test against different features you can add them as arguments: `just bench parallel` or `just bench parallel tabulation`.
- `logs.rs` benches against the cpp binary. You will need a [IUPACpal](https://github.com/steven31415/IUPACpal) binary (and they only support Linux). The binary is expected to be in the bench folder, but that can be changed in `logs.rs` and `validate.py`. 
- Note that `just heatmap` requires the python libraries listed in `bench/requirements.txt`.