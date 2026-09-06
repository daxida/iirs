from typing import Literal

RepeatType = Literal["inverted", "mirror", "direct", "complement"]

class SearchParams:
    def __init__(
        self,
        min_len: int,
        max_len: int,
        max_gap: int,
        mismatches: int,
        repeat_type: RepeatType = "inverted",
    ) -> None: ...

def find_repeats(
    params: SearchParams,
    seq: str,
) -> list[tuple[int, int, int]]: ...
def find_irs(
    params: SearchParams,
    seq: str,
) -> list[tuple[int, int, int]]: ...
