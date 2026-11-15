#!/usr/bin/env python3
"""Non-compiling checks for E1 sample selection and regrade comparison."""
import copy
import unittest

import gate_reproduce as reproduce


class ReproductionTests(unittest.TestCase):
    def test_frozen_sample_covers_both_arms(self):
        import tarfile
        with tarfile.open(reproduce.HERE / "gate-results/records.tar.gz") as archive:
            state = reproduce.json.load(archive.extractfile("state.json"))
        self.assertEqual(reproduce.sample(state["order"]),
                         [5, 9, 26, 28, 30, 31, 39, 48, 60, 65, 93, 97])

    def test_comparison_ignores_resource_noise_but_keeps_verdicts_and_tests(self):
        original = {"structure": True, "timed_out": False, "infrastructure": False,
                    "results": [{"code": 0, "command": ["/old/tmp/test"],
                                 "memory_peak": 100, "test_log": "ok case1\n",
                                 "expected_tests": ["case1"], "observed_tests": ["case1"]}]}
        changed = copy.deepcopy(original)
        changed["results"][0].update(command=["/new/tmp/test"], memory_peak=200)
        self.assertEqual(reproduce.signature(original), reproduce.signature(changed))
        for key, value in [("code", 1), ("test_log", "ok case2\n"),
                           ("observed_tests", ["case2"])]:
            changed = copy.deepcopy(original)
            changed["results"][0][key] = value
            self.assertNotEqual(reproduce.signature(original), reproduce.signature(changed))
        for key in ("structure", "timed_out", "infrastructure"):
            changed = copy.deepcopy(original)
            changed[key] = not original[key]
            self.assertNotEqual(reproduce.signature(original), reproduce.signature(changed))


if __name__ == "__main__":
    unittest.main()
