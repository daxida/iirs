"""Tests for the python bindings.

pip install ./py-iirs pytest
pytest py-iirs/tests

or:

just testpy
"""

import pytest

from iirs import SearchParams, find_irs, find_repeats


def arms(seq, repeat):
    """The two arms of a `(start, end, gap)` triple."""
    start, end, gap = repeat
    arm_len = (end + 1 - start - gap) // 2
    return seq[start:start + arm_len], seq[end + 1 - arm_len:end + 1]


def test_finds_an_inverted_repeat():
    params = SearchParams(3, 6, 2, 0)
    assert find_irs(params, "acbbgt") == [(0, 5, 0)]


def test_parameters_can_be_named():
    params = SearchParams(min_len=3, max_len=6, max_gap=2, mismatches=0)
    assert find_irs(params, "acbbgt") == [(0, 5, 0)]


def test_triples_describe_the_arms():
    seq = "acgtacgt"
    (repeat,) = find_irs(SearchParams(4, 4, 0, 0), seq)
    # "acgt" is its own reverse complement, so both arms read the same way round
    assert arms(seq, repeat) == ("acgt", "acgt")


def test_nothing_to_find():
    # "a" is complementary to "t", never to itself
    assert find_irs(SearchParams(3, 6, 0, 0), "aaaaaa") == []


def test_max_len_truncates():
    seq = "acgtacgt"
    assert find_irs(SearchParams(2, 4, 0, 0), seq) == [(0, 7, 0), (0, 3, 0), (4, 7, 0)]
    assert find_irs(SearchParams(2, 2, 0, 0), seq) == [(0, 3, 0), (2, 5, 0), (4, 7, 0)]


def test_mismatches_are_allowed():
    # the third pair out of the centre, "g" against "a", does not match
    seq = "acgtaagt"
    assert find_irs(SearchParams(4, 4, 0, 0), seq) == []
    assert find_irs(SearchParams(4, 4, 0, 1), seq) == [(0, 7, 0)]


def test_sequences_are_iupac_and_case_insensitive():
    assert find_irs(SearchParams(3, 6, 2, 0), "ACBBGT") == [(0, 5, 0)]
    # "n" stands for any base, so it is complementary to itself
    assert find_irs(SearchParams(2, 4, 0, 0), "nnnn") == [(0, 3, 0)]
    with pytest.raises(ValueError):
        find_irs(SearchParams(3, 6, 2, 0), "jj")


def test_invalid_parameters():
    with pytest.raises(ValueError):
        SearchParams(0, 4, 2, 0)  # min_len below two
    with pytest.raises(ValueError):
        SearchParams(4, 2, 2, 0)  # min_len over max_len
    with pytest.raises(ValueError):
        SearchParams(4, 4, 2, 4)  # mismatches not below min_len


# "aacc" against, in order, itself, its reverse, its complement and its reverse
# complement, always with a "ct" gap. The gap has to mismatch on both of its sides, or the
# arms would rather grow into it than stop.
SEQS = {
    "direct": "aaccctaacc",
    "mirror": "aaccctccaa",
    "complement": "aaccctttgg",
    "inverted": "aaccctggtt",
}


def test_each_repeat_type_finds_its_own():
    for repeat_type, seq in SEQS.items():
        params = SearchParams(4, 4, 2, 0, repeat_type=repeat_type)
        assert find_repeats(params, seq) == [(0, 9, 2)], repeat_type
        for other, other_seq in SEQS.items():
            if other != repeat_type:
                assert find_repeats(params, other_seq) == [], f"{repeat_type} in {other}"


def test_repeat_type_defaults_to_inverted():
    assert find_repeats(SearchParams(4, 4, 2, 0), SEQS["inverted"]) == [(0, 9, 2)]


def test_repeat_type_is_case_insensitive():
    params = SearchParams(4, 4, 2, 0, repeat_type="DIRECT")
    assert find_repeats(params, SEQS["direct"]) == [(0, 9, 2)]


def test_invalid_repeat_type():
    with pytest.raises(ValueError):
        SearchParams(4, 4, 2, 0, repeat_type="nonsense")


def test_find_irs_is_an_alias():
    params = SearchParams(4, 4, 2, 0, repeat_type="direct")
    assert find_irs(params, SEQS["direct"]) == find_repeats(params, SEQS["direct"])
