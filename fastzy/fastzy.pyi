import typing


class Searcher:
    def __init__(
        self,
        file_path: str,
        separator: str,
    ) -> None: ...

    def search(
        self,
        pattern: str,
        max_distance: int,
    ) -> typing.List[str]: ...

    @staticmethod
    def mbleven(
        first_string: str,
        second_string: str,
        max_distance: int,
    ) -> bool: ...

    @staticmethod
    def wagner_fischer(
        first_string: str,
        second_string: str,
        max_distance: int,
    ) -> bool: ...
