"""Tests for the python bindings.

pip install ./py-iirs pytest
pytest py-iirs/tests

or:

just testpy
"""

import pytest

from iirs import SearchParams, find_irs


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
