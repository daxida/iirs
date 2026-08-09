# How the algorithm works

## Degenerate (IUPAC) bases

DNA sequences from real data aren't always a clean `A`/`C`/`G`/`T`. The IUPAC alphabet adds 11 extra symbols for *ambiguous* positions: for instance `R` means "A or G", `N` means "any base".

`iirs` uses the *permissive* rule: two IUPAC symbols are considered complementary if **at least one** pair of bases they could represent is complementary. This is implemented as a small precomputed 15×15 table (`MatchMatrix`), so checking a match at runtime is a single array lookup, not a set computation. This could potentially be changed by using a different match matrix implementation.

## The main idea: turn the search into a mirror-matching problem

The naive way to find IRs is: for every possible center point in the sequence, walk outward in both directions and check if the two sides stay complementary. That works, but re-scanning from scratch at every center is slow for long sequences.

`iirs` speeds this up with a classic trick. It builds one combined string:

```
s = seq + "$" + revcomp(seq) + "#"
```

`revcomp(seq)` is the reverse complement of the whole input sequence, and `$`/`#` are separators that never match anything else (they exist so the algorithm never accidentally treats a match as crossing between the two halves). With `seq = "ACGNGT"`, this becomes `s = "ACGNGT$ACNCGT#"`.

The key property: an inverted repeat in `seq` now shows up as a **direct, extendable match** between a suffix in the first half of `s` and a suffix in the second half. Walking outward from a center in `seq` becomes walking *forward* through matching characters in `s` — a much more standard string-matching problem, for which fast tools already exist.

## Suffix array, LCP, and RMQ: answering "how far do these match?" instantly

To exploit that, `iirs` builds three structures over `s`, once, before looking at any candidate IR:

- **Suffix array**: all suffixes of `s`, sorted alphabetically. Suffixes that share a long prefix end up next to each other in this order.
- **LCP array** (Longest Common Prefix): for each pair of neighbouring suffixes in that sorted order, how many characters they share at the start.
- **RMQ** (Range Minimum Query) over the LCP array: a structure that can answer, for any two suffixes, "how long is their common prefix?" in constant time, by finding the smallest LCP value between their positions in the sorted order.

The payoff: instead of comparing characters one by one to see how far a match extends, `iirs` asks the RMQ structure directly and gets the answer immediately. This is what makes scanning every possible center of every possible IR affordable.

## The "kangaroo method": jumping over exact matches, landing on mismatches

Real sequences rarely form a perfect IR for their entire length — a few mismatches are usually allowed (this is the `-x` / `--mismatches` option). So for each candidate center, `iirs` needs to walk outward and find *where the real mismatches are*, while ignoring long exact-match stretches in between.

This is the **kangaroo method**: instead of stepping one character at a time, it uses the RMQ to jump straight to the end of the current matching stretch (like a kangaroo hopping over the boring parts), checks the single character where the match breaks, and records that position as a mismatch if the two IUPAC symbols there aren't complementary. It repeats this until it has used up the allowed mismatch budget or reached the end of the sequence.

The result is a short list of mismatch positions for that center, instead of a full character-by-character trace.

## From mismatch positions to valid IRs

Given that list of mismatch positions, `iirs` slides two pointers across it to find every stretch that:

- has no more than the allowed number of mismatches (`-x`),
- is at least `--min-len` long and at most `--max-len` long,
- has a gap between its two arms no larger than `--max-gap`.

Each valid stretch found this way becomes one reported IR: a `(start, end, gap)` triple. If a candidate would exceed `--max-len`, it's trimmed down to size rather than discarded, as long as trimming doesn't leave it ending awkwardly in the middle of a mismatch.

## Putting it together

1. Read the FASTA sequence.
2. Build `s = seq + "$" + revcomp(seq) + "#"`.
3. Build the suffix array, LCP array, and RMQ structure over `s`; once.
4. For every possible center position, use the kangaroo method to get its mismatch list.
5. Slide a two-pointer window over each mismatch list to collect every valid IR.
6. Sort and output the results.

