"""Metadata-only regression proof for shared suite acceptance values."""

import json
from pathlib import Path
import tempfile
import unittest

import generate_professional_output_suite as producer
import professional_output_numeric_policy as policy
import validate_professional_output_suite_contract as validator


class ProfessionalOutputNumericTests(unittest.TestCase):
    def test_shared_owners_preserve_the_previous_v1_values(self):
        expected = {
            "MIN_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO": 0.145,
            "MAX_FERAL_SOURCE_FIRST_GENERATED_TO_SOURCE_RMS_RATIO": 0.08,
            "MIN_FERAL_SOURCE_FIRST_MASKING_HEADROOM": 0.04,
            "MAX_FERAL_SUPPORT_GENERATED_TO_SOURCE_RMS_RATIO": 0.46,
            "MIN_FERAL_TR909_RENDERED_SUPPORT_CONTRIBUTION_RATIO": 0.050,
            "MIN_FERAL_TR909_RENDERED_LOW_BAND_RMS": 0.0030,
            "MIN_SOURCE_CHARACTER_WINDOW_RMS_RETENTION_RATIO": 0.98,
            "MIN_SOURCE_CHARACTER_WINDOW_SEARCHED_CASE_COUNT": 3,
            "MIN_SOURCE_CHARACTER_WINDOW_PROMOTED_CASE_COUNT": 1,
            "MIN_SOURCE_SELECTION_POLICY_CANDIDATES": 3,
            "MIN_SOURCE_SELECTION_RMS_RETENTION_RATIO": 0.60,
            "MIN_SOURCE_SELECTION_SCORE_LIFT": 0.0,
        }
        for name, value in expected.items():
            with self.subTest(name=name):
                self.assertEqual(getattr(policy, name), value)
                self.assertEqual(getattr(producer, name), value)
                self.assertEqual(getattr(validator, name), value)

    def test_generated_summary_and_validator_agree_at_inclusive_boundaries(self):
        # The producer's directory walk is limited to this fresh synthetic JSON
        # tree, never an existing source or ignored audio directory.
        for support in (0.145, 0.46, 0.144999, 0.460001):
            for source_first in (0.04, 0.040001, 0.080001):
                with self.subTest(support=support, source_first=source_first):
                    with tempfile.TemporaryDirectory() as temporary:
                        root = Path(temporary)
                        for index in range(8):
                            case = root / str(index)
                            case.mkdir()
                            (case / "manifest.json").write_text(json.dumps({
                                "metrics": {"mix_balance": {
                                    "source_first_generated_to_source_rms_ratio": source_first,
                                    "support_generated_to_source_rms_ratio": support,
                                }},
                            }))
                        summary = producer.feral_mix_balance_summary(root)
                    failures = []
                    validator.validate_feral_mix_balance_metrics(
                        {"feral_mix_balance": summary}, failures,
                    )
                    expected = 0.145 <= support <= 0.46 and source_first <= 0.04
                    self.assertEqual(summary["result"] == "pass", expected)
                    self.assertEqual(not failures, expected)
                    self.assertEqual(set(summary["failure_codes"]),
                                     set(failures) - {"feral_mix_balance_not_pass"})


if __name__ == "__main__":
    unittest.main()
