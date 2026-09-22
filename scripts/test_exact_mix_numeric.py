"""Source-free boundary tests for the predicates used by both shell validators."""

import copy
import itertools
import json
from pathlib import Path
import struct
import subprocess
import unittest


SCRIPTS = Path(__file__).resolve().parent


def f32(value):
    return struct.unpack("f", struct.pack("f", value))[0]


class ExactMixNumericTests(unittest.TestCase):
    def accepts(self, value, predicate):
        result = subprocess.run(
            ["jq", "-L", str(SCRIPTS), "-e",
             'include "exact_mix_numeric"; ' + predicate],
            input=json.dumps(value), text=True, capture_output=True, check=False,
        )
        self.assertIn(result.returncode, (0, 1), result.stderr)
        self.assertIn(result.stdout.strip(), ("true", "false"))
        return result.returncode == 0

    def boundary(self):
        return {
            "window_ms": 10, "window_frames": 480,
            "boundary_step": 0.1, "local_adjacent_step_p99": 0.03,
            "boundary_to_local_p99_ratio": 3.0,
            "post_boundary_attack_rms": 0.04,
            "boundary_to_attack_rms_ratio": 2.5,
            "thresholds": {
                "max_boundary_step": f32(0.2),
                "max_boundary_to_local_p99_ratio": 4.0,
                "max_boundary_to_attack_rms_ratio": 4.0,
            },
        }

    def test_fill_boundary_matches_rust_strict_three_way_conjunction(self):
        fields = ("boundary_step", "boundary_to_local_p99_ratio",
                  "boundary_to_attack_rms_ratio")
        # Below / equal / above each maximum: equality must pass, even when
        # the other two are exceeded. This catches the old shell AND-of-maxima.
        for factors in itertools.product((0.5, 1.0, 1.5), repeat=3):
            with self.subTest(factors=factors):
                boundary = self.boundary()
                for field, factor in zip(fields, factors):
                    boundary[field] = boundary["thresholds"]["max_" + field] * factor
                expected = not all(factor > 1 for factor in factors)
                self.assertEqual(self.accepts(boundary, "fill_exit_boundary_ok(48000)"), expected)

    def test_fill_uses_recorded_thresholds_and_sample_rate(self):
        boundary = self.boundary()
        boundary["window_frames"] = 441
        boundary["boundary_step"] = 0.3
        boundary["boundary_to_local_p99_ratio"] = 5.0
        boundary["boundary_to_attack_rms_ratio"] = 5.0
        self.assertFalse(self.accepts(boundary, "fill_exit_boundary_ok(44100)"))
        boundary["thresholds"]["max_boundary_to_attack_rms_ratio"] = 6.0
        self.assertTrue(self.accepts(boundary, "fill_exit_boundary_ok(44100)"))
        self.assertFalse(self.accepts(boundary, "fill_exit_boundary_ok(48000)"))

    def test_missing_malformed_and_nonfinite_boundary_evidence_fails_closed(self):
        original = self.boundary()
        for field in ("boundary_step", "local_adjacent_step_p99",
                      "boundary_to_local_p99_ratio", "post_boundary_attack_rms",
                      "boundary_to_attack_rms_ratio", "window_ms", "window_frames"):
            for invalid in (None, "0.1", True, -1, float("nan"), float("inf")):
                with self.subTest(field=field, invalid=invalid):
                    value = copy.deepcopy(original)
                    value[field] = invalid
                    self.assertFalse(self.accepts(value, "fill_exit_boundary_ok(48000)"))
        for field in original["thresholds"]:
            for invalid in (None, 0, -1, "4", True, float("inf")):
                value = copy.deepcopy(original)
                value["thresholds"][field] = invalid
                self.assertFalse(self.accepts(value, "fill_exit_boundary_ok(48000)"))

    def test_alpha_equality_polarity_legacy_and_explicit_threshold(self):
        for maximum in (f32(0.985), 0.8):
            for sign in (-1, 1):
                for offset, expected in ((-1e-6, True), (0, True), (1e-6, False)):
                    value = {
                        "max_hook_to_changed_return_correlation": maximum,
                        "hook_to_changed_return_correlation": sign * (maximum + offset),
                    }
                    self.assertEqual(self.accepts(value, "alpha_return_correlation_ok"), expected)
                    if maximum == f32(0.985):
                        del value["max_hook_to_changed_return_correlation"]
                        self.assertEqual(self.accepts(value, "alpha_return_correlation_ok"), expected)

    def test_invalid_alpha_threshold_never_uses_legacy_fallback(self):
        for invalid in (None, True, "0.985", 0, -1, 1.1, float("inf")):
            value = {"hook_to_changed_return_correlation": 0.1,
                     "max_hook_to_changed_return_correlation": invalid}
            self.assertFalse(self.accepts(value, "alpha_return_correlation_ok"))
        for invalid in (None, True, "0.1", float("nan"), float("inf")):
            self.assertFalse(self.accepts({"hook_to_changed_return_correlation": invalid},
                                          "alpha_return_correlation_ok"))

    def test_format_matches_graph_not_generator_fixture(self):
        for rate in (44100, 48000, 96000):
            value = {"sample_rate": 48000, "channel_count": 2,
                     "source": {"sample_rate": rate, "channel_count": 1}}
            graph = {"source": value["source"].copy()}
            predicate = "exact_source_format_ok(" + json.dumps(graph) + ")"
            self.assertTrue(self.accepts(value, predicate))
            value["source"]["sample_rate"] += 1
            self.assertFalse(self.accepts(value, predicate))
            value["source"]["sample_rate"] = rate
            value["source"]["channel_count"] = 2
            self.assertFalse(self.accepts(value, predicate))

    def test_clean_path_cannot_hide_limiting_or_malformed_counts(self):
        value = {"threshold": f32(0.92), "ceiling": f32(0.985),
                 "limited_sample_count": 0, "applied": False,
                 "pre": {"clip_count": 0}, "post": {"clip_count": 0}}
        self.assertTrue(self.accepts(value, "exact_limiter_ok(0)"))
        for invalid in (None, True, "0", -1, 0.1, 1):
            value["limited_sample_count"] = invalid
            self.assertFalse(self.accepts(value, "exact_limiter_ok(0)"))
        value["limited_sample_count"] = 1
        value["applied"] = True
        self.assertFalse(self.accepts(value, "exact_limiter_ok(1)"))
        value["limited_sample_count"] = 0
        value["applied"] = False
        value["pre"]["clip_count"] = 1
        self.assertFalse(self.accepts(value, "exact_limiter_ok(0)"))

    def test_pack_thresholds_are_explicit_finite_domains_not_fixture_values(self):
        value = {"max_exact_mix_limited_sample_count": 0,
                 "min_mix_rms": 0.012, "min_monitor_delta_rms": 0.006,
                 "min_isolated_tr909_regression_rms": 0.008,
                 "max_source_monitor_silence_ratio": 0.03}
        self.assertTrue(self.accepts(value, "exact_pack_thresholds_ok"))
        for field in value:
            for invalid in (None, True, "0", -1, float("nan"), float("inf")):
                with self.subTest(field=field, invalid=invalid):
                    broken = value.copy()
                    broken[field] = invalid
                    self.assertFalse(self.accepts(broken, "exact_pack_thresholds_ok"))


if __name__ == "__main__":
    unittest.main()
