import unittest
import tempfile

import fastzy


class FastzyTestCase(
    unittest.TestCase,
):
    def assert_fuzzy_string_search(
        self,
        lines,
        separator,
        pattern,
        max_distance,
        expected_results,
    ):
        with tempfile.NamedTemporaryFile('w') as tmp_file:
            for line in lines:
                tmp_file.write(line + '\n')
            tmp_file.flush()

            searcher = fastzy.Searcher(
                input_file_path=tmp_file.name,
                separator=separator,
            )
            results = searcher.lookup(
                pattern=pattern,
                max_distance=max_distance,
            )
            self.assertCountEqual(
                first=results,
                second=expected_results,
            )

    def test_file_not_found(
        self,
    ):
        with self.assertRaises(
            expected_exception=RuntimeError,
        ):
            fastzy.Searcher(
                input_file_path='missing_file_path',
                separator='',
            )

    def test_sanity(
        self,
    ):
        lines = [
            'firstline',
            'secondline',
            'thirdline',
            'fourthline',
            'fifthline',
            'first.line',
            'second.line',
            'third.line',
            'fourth.line',
            'fifth.line',
        ]

        self.assert_fuzzy_string_search(
            lines=lines,
            separator='',
            pattern='a',
            max_distance=1,
            expected_results=[],
        )

        self.assert_fuzzy_string_search(
            lines=lines,
            separator='',
            pattern='forthline',
            max_distance=1,
            expected_results=[
                'fourthline',
            ],
        )

        self.assert_fuzzy_string_search(
            lines=lines,
            separator='',
            pattern='firstline',
            max_distance=1,
            expected_results=[
                'firstline',
                'first.line',
            ],
        )

        self.assert_fuzzy_string_search(
            lines=lines,
            separator='.',
            pattern='fist',
            max_distance=1,
            expected_results=[
                'first.line',
            ],
        )

        self.assert_fuzzy_string_search(
            lines=lines,
            separator='.',
            pattern='fourth',
            max_distance=3,
            expected_results=[
                'fourth.line',
                'fifth.line',
            ],
        )

        self.assert_fuzzy_string_search(
            lines=lines,
            separator='.',
            pattern='fourth',
            max_distance=4,
            expected_results=[
                'first.line',
                'fourth.line',
                'fifth.line',
            ],
        )
