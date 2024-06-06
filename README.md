# iirs

IIRS is an [Iupac](https://en.wikipedia.org/wiki/International_Union_of_Pure_and_Applied_Chemistry) Inverted RepeatS finder written in rust (rs), ported from [IUPACpal](https://github.com/steven31415/IUPACpal), result of this [paper](https://www.researchgate.net/publication/349110200_IUPACpal_efficient_identification_of_inverted_repeats_in_IUPAC-encoded_DNA_sequences).

That is, an exact tool for efficient identification of Inverted Repeats (IRs) in IUPAC-encoded DNA sequences as substrings of a large text, allowing also for potential mismatches and gaps.

Compared to the original, this version is faster, platform-independent and modular, facilitating the creation of customized format outputs. It does not require `cmake`, `libdivsufsort` nor `sdsl`.

## How to use the binary

The command line shares much of the functionality of the original IUPACpal. Typing `iirs --help` will return:

```
Usage: iirs [OPTIONS]

Options:
  -f <INPUT_FILE>         Input filename (FASTA) [default: input.fasta]
  -s <SEQ_NAMES>          Input sequence names [default: seq0]
  -m <MIN_LEN>            Minimum length [default: 10]
  -M <MAX_LEN>            Maximum length [default: 100]
  -g <MAX_GAP>            Maximum permissible gap [default: 100]
  -x <MISMATCHES>         Maximum permissible mismatches [default: 0]
  -o <OUTPUT_FILE>        Output filename [default: iirs.out]
  -F <OUTPUT_FORMAT>      Output format (classic, csv or custom) [default: classic]
  -h, --help              Print help
```

With the notable differences being support for multiple sequence names and the output format.

```
iirs -f input.fasta -s 't1 t2' -g 5 -F csv
```

Many more practical examples can be found in the justfile.

## How to install the binary

### (Option 1) Download executable

Download the latest binary from [releases](https://github.com/daxida/iirs/releases) and extract it somewhere on your `$PATH`.

### (Option 2) Build executable

```
$ git clone https://github.com/daxida/iirs
$ cd iirs
$ cargo build --release
$ // The binary will be located at `target/release/iirs`
```

### (Option 3) Build from source:

```
$ git clone https://github.com/daxida/iirs
$ cargo install --path=.
```

## Branches

TODO

- master (Sparse table)
- parallel (parallelize the main loop - not ideal if we want to parallelize over sequences, but THE FASTEST if we only query one sequence)
- [tabulation](https://github.com/daxida/rmq-tabulation) (maybe better than master? needs testing)
- visualize && custom > ignore

## Testing

- `cargo test` for unit tests.
- [Justfile](https://github.com/casey/just) for individual tests against sequences. Some use the Linux profiler [perf](https://en.wikipedia.org/wiki/Perf_(Linux)).
- `bench.rs` benches against a single file. To use together with `just bench` after modifying the parameters in `bench.rs`.
- `logs.rs` benches against the cpp binary. You will need a [IUPACpal](https://github.com/steven31415/IUPACpal) binary (and they only support Linux). The binary is expected to be in the bench folder, but that can be changed in `logs.rs` and `validate.py`. 
- For instance, create the heatmaps with `just printlogs`. You can also modify the `steps` in which the binaries are compared. A heatmap will be created per `size_seq` (size of sequence). **NOTE**: There is a `requirements.txt` that needs to be previously installed, with libraries like plotly to be able to print the heatmaps.

## Links
* [IUPACpal](https://github.com/steven31415/IUPACpal)
* [divsufsort](https://github.com/y-256/libdivsufsort) and [dismantling divsufsort](https://arxiv.org/pdf/1710.01896.pdf)
* [libdivsufsort port in rust](https://github.com/fasterthanlime/stringsearch?tab=readme-ov-file)
* [Alternative RMQ implementations](https://github.com/birc-stormtroopers/rmq)