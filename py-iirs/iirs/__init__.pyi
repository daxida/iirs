class SearchParams:
    def __init__(
        self,
        min_len: int,
        max_len: int,
        max_gap: int,
        mismatches: int,
    ) -> None: ...

def find_irs(
    params: SearchParams,
    seq: str,
) -> list[tuple[int, int, int]]: ...
