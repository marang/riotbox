"""Source-free count evidence across the three professional-suite surfaces."""

from collections import defaultdict
import unittest

import generate_dense_break_performance_pack as dense
import generate_professional_output_suite as suite
import generate_professional_source_wav_pack as source_wav
import validate_professional_output_suite_contract as validator


SURFACES = (
    ("dense_break", "hook_chop_riff_reverse_count", "hook_chop_riff_hit_count",
     "dense_hook_chop_riff_reverse_missing"),
    ("pro_pressure_source_matrix", "min_dense_hook_chop_riff_reverse_count",
     "min_dense_hook_chop_riff_hit_count", "matrix_dense_hook_chop_riff_reverse_missing"),
    ("professional_source_wav_pack", "tonal_hook_chop_riff_reverse_count",
     "tonal_hook_chop_riff_hit_count", "source_wav_tonal_hook_chop_riff_reverse_missing"),
)


def child(child_id, counts):
    proofs = [{"hook_chop_riff_reverse_count": count,
               "hook_chop_riff_hit_count": 12} for count in counts]
    if child_id == "dense_break":
        return {"proof": proofs[0]}
    if child_id == "pro_pressure_source_matrix":
        return {"cases": [{"proof": proof,
                           "pressure_lift_policy": {"source_family": "dense_break"}}
                          for proof in proofs]}
    return {"cases": [{"proof": proof, "source_family": "tonal_hook"}
                      for proof in proofs]}


class HookChopDiagnosticCountTests(unittest.TestCase):
    def test_zero_one_two_boundary_on_every_suite_surface(self):
        for child_id, reverse_key, hit_key, code in SURFACES:
            for count in (0, 1, 2, 2.0):
                with self.subTest(surface=child_id, count=count):
                    metrics = [{keys[1]: 2, keys[2]: 12} for keys in SURFACES]
                    index = [keys[0] for keys in SURFACES].index(child_id)
                    metrics[index][reverse_key] = count
                    failures = []
                    validator.validate_hook_chop_metrics(*metrics, failures)
                    self.assertEqual(code in failures, count < 2)

    def test_missing_malformed_and_impossible_counts_fail_closed(self):
        for child_id, reverse_key, hit_key, code in SURFACES:
            for count in (None, True, "2", 2.5, -1, float("nan"), float("inf"), 13):
                with self.subTest(surface=child_id, count=count):
                    metrics = [{keys[1]: 2, keys[2]: 12} for keys in SURFACES]
                    index = [keys[0] for keys in SURFACES].index(child_id)
                    metrics[index][reverse_key] = count
                    failures = []
                    validator.validate_hook_chop_metrics(*metrics, failures)
                    self.assertIn(code, failures)
            for missing_key in (reverse_key, hit_key):
                metrics = [{keys[1]: 2, keys[2]: 12} for keys in SURFACES]
                index = [keys[0] for keys in SURFACES].index(child_id)
                del metrics[index][missing_key]
                failures = []
                validator.validate_hook_chop_metrics(*metrics, failures)
                self.assertIn(code, failures)
            for hits in (None, True, "12", 12.5, float("inf"), 1):
                metrics = [{keys[1]: 2, keys[2]: 12} for keys in SURFACES]
                index = [keys[0] for keys in SURFACES].index(child_id)
                metrics[index][hit_key] = hits
                failures = []
                validator.validate_hook_chop_metrics(*metrics, failures)
                self.assertIn(code, failures)

    def test_producer_preserves_counts_and_minima_without_inventing_missing_proof(self):
        for child_id, reverse_key, _, _ in SURFACES:
            for count in (0, 1, 2):
                data = child(child_id, [count, 5])
                self.assertEqual(suite.key_metrics(child_id, data)[reverse_key], count)
            for count in (None, "2", 2.5, 13):
                self.assertIsNone(suite.key_metrics(child_id, child(child_id, [count, 5]))[reverse_key])
            data = child(child_id, [2])
            proof = data.get("proof") or data["cases"][0]["proof"]
            del proof["hook_chop_riff_reverse_count"]
            self.assertIsNone(suite.key_metrics(child_id, data)[reverse_key])
        for child_id, reverse_key, _, _ in SURFACES[1:]:
            self.assertIsNone(suite.key_metrics(child_id, {"cases": []})[reverse_key])

    def test_tonal_child_validator_rejects_the_one_reverse_loophole(self):
        for count in (0, 1, 2, None, 2.5, 13):
            failures = []
            source_wav.validate_tonal_case("synthetic", {
                "hook_chop_riff_reverse_count": count,
                "hook_chop_riff_hit_count": 12,
            }, failures)
            self.assertEqual("synthetic:hook_chop_riff_reverse_missing" in failures,
                             count != 2)

    def test_dense_child_uses_the_same_count_contract(self):
        for family in (dense.SOURCE_FAMILY_DENSE_BREAK, dense.SOURCE_FAMILY_TONAL_HOOK):
            for count in (0, 1, 2, None, 2.5, 13):
                proof = defaultdict(lambda: 1.0, {
                    "hook_chop_riff_reverse_count": count,
                    "hook_chop_riff_hit_count": 12,
                })
                failures = dense.failure_codes_for({}, proof, family)
                self.assertEqual("hook_chop_riff_reverse_missing" in failures, count != 2)

    def test_tonal_case_classification_cannot_bypass_count_validation(self):
        proof = {
            "hook_chop_selection_source_derived": 1,
            "hook_chop_static_distance_frames": 1024,
            "hook_chop_offset_distance_frames": 1024,
            "w30_to_source_rms_ratio": .3,
            "mc202_to_w30_rms_ratio": .3,
            "hook_chop_riff_unique_source_offset_count": 6,
            "hook_chop_riff_hit_pattern_source_derived": 1,
            "hook_chop_riff_hit_count": 12,
            "hook_chop_riff_velocity_span": .3,
            "hook_chop_source_character_score_floor": .7,
            "hook_chop_source_character_score_span": .2,
            "destructive_gesture_source_derived": 1,
            "destructive_static_distance_frames": 1024,
            "destructive_offset_distance_frames": 1024,
        }
        for count in (0, 1, 2, None, 2.5, 13):
            proof["hook_chop_riff_reverse_count"] = count
            self.assertEqual(source_wav.is_tonal_professional_case({
                "source_family": "tonal_hook", "proof": proof,
            }), count == 2)

    def test_aggregation_validates_every_case_and_preserves_family_scope(self):
        for child_id, reverse_key, _, _ in SURFACES[1:]:
            data = child(child_id, [5, None])
            self.assertIsNone(suite.key_metrics(child_id, data)[reverse_key])
            data = child(child_id, [3, 2])
            if child_id == "pro_pressure_source_matrix":
                other = {"pressure_lift_policy": {"source_family": "pad_noise"}}
            else:
                other = {"source_family": "sparse_bass_pressure"}
            data["cases"].append(other)
            self.assertEqual(suite.key_metrics(child_id, data)[reverse_key], 2)


if __name__ == "__main__":
    unittest.main()
