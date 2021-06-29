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
                file_path=tmp_file.name,
                separator=separator,
            )
            results = searcher.search(
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
                file_path='missing_file_path',
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
            'fourthhhhh.line',
            'fifth.line',
            'forthlines',
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
                'forthlines',
            ],
        )

        self.assert_fuzzy_string_search(
            lines=lines,
            separator='.',
            pattern='forthline',
            max_distance=1,
            expected_results=[
                'fourthline',
                'forthlines',
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
                'fourthhhhh.line',
                'fifth.line',
                'fourthline',
            ],
        )

    def test_wagner_fischer(
        self,
    ):
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('a', '', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('', 'a', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('a', 'b', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('b', 'a', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('a', 'aa', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('aa', 'a', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('ab', 'ad', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('abcd', 'abdd', 1),
        )

        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('ab', 'cd', 1),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('abcd', 'abef', 1),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('abcdefghijk', 'abcdefghiii', 1),
        )

        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('', '', 0),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('1', '1', 0),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('1', '2', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('12', '12', 0),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('123', '12', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('1234', '1', 3),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('1234', '1233', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('1248', '1349', 2),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('', '12345', 5),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('5677', '1234', 4),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('123456', '12345', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('13579', '12345', 4),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('123', '', 3),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('kitten', 'mittens', 2),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('אבא', 'אמא', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.wagner_fischer('אﺑא', 'אמא', 1),
        )

        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('1234', '1', 2),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('1248', '1349', 1),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('', '12345', 4),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('5677', '1234', 3),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('13579', '12345', 3),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('123', '', 2),
        )
        self.assertFalse(
            expr=fastzy.Searcher.wagner_fischer('kitten', 'mittens', 1),
        )

    def test_mbleven(
        self,
    ):
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('a', '', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('', 'a', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('a', 'b', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('b', 'a', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('a', 'aa', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('aa', 'a', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('ab', 'ad', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('abcd', 'abdd', 1),
        )

        self.assertFalse(
            expr=fastzy.Searcher.mbleven('ab', 'cd', 1),
        )
        self.assertFalse(
            expr=fastzy.Searcher.mbleven('abcd', 'abef', 1),
        )
        self.assertFalse(
            expr=fastzy.Searcher.mbleven('abcdefghijk', 'abcdefghiii', 1),
        )

        self.assertTrue(
            expr=fastzy.Searcher.mbleven('', '', 0),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('1', '1', 0),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('1', '2', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('12', '12', 0),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('123', '12', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('1234', '1', 3),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('1234', '1233', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('1248', '1349', 2),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('123456', '12345', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('123', '', 3),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('kitten', 'mittens', 2),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('אבא', 'אמא', 1),
        )
        self.assertTrue(
            expr=fastzy.Searcher.mbleven('אﺑא', 'אמא', 1),
        )

        self.assertFalse(
            expr=fastzy.Searcher.mbleven('1234', '1', 2),
        )
        self.assertFalse(
            expr=fastzy.Searcher.mbleven('1248', '1349', 1),
        )
        self.assertFalse(
            expr=fastzy.Searcher.mbleven('5677', '1234', 3),
        )
        self.assertFalse(
            expr=fastzy.Searcher.mbleven('13579', '12345', 3),
        )
        self.assertFalse(
            expr=fastzy.Searcher.mbleven('123', '', 2),
        )
        self.assertFalse(
            expr=fastzy.Searcher.mbleven('kitten', 'mittens', 1),
        )
