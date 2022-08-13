import unittest
import foo


class FooTest(unittest.TestCase):
    def test_success(self) -> None:
        actual = foo.add(1, 2)
        expected = 3
        self.assertEqual(actual, expected)

    def test_failure(self) -> None:
        actual = foo.add(1, 2)
        expected = 4
        self.assertEqual(actual, expected)
