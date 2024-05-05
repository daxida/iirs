# iupacpal

iupacpal is an exact tool for efficient identification of inverted repeats in IUPAC-encoded DNA sequences as substrings of a large text, allowing also for potential mismatches and gaps.

This is a rust port of [IUPACpal](https://github.com/steven31415/IUPACpal), result of their work on this [paper](https://www.researchgate.net/publication/349110200_IUPACpal_efficient_identification_of_inverted_repeats_in_IUPAC-encoded_DNA_sequences).

Compared to the original this version is faster, platform-independent and modular, facilitating the creation of customized format outputs. It does not require `cmake`, `libdivsufsort` nor `sdsl`.

## How to use

TODO - but mainly it works much like the original with an extra flag for the type of output format.

There are many examples in the justfile.

This also works as a library: The `find_irs` function is exported (think of crates.io).

## How to install

### (Option 1) Download executable

Download the latest binary from [releases](https://github.com/daxida/iupacpal/releases) and extract it somewhere on your `$PATH`.

### (Option 2) Build executable

```
$ git clone https://github.com/daxida/iupacpal
$ cd iupacpal
$ cargo build --release
$ // The binary will be located at `target/release/iupacpal`
```

### (Option 3) Build from source:

```
$ git clone https://github.com/daxida/iupacpal
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