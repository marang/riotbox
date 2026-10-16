"""Prospective 1581 V2 fence adapter; no actual approval or host invocation."""

import unittest

import loaded_recorder_prospective_test_support as support
from loaded_recorder_evidence import load_json_bytes, require


class LoadedProspective1581Tests(support.ProspectiveFenceTests):
    CONFIG = support.PhaseConfig(
        ticket="RIOTBOX-1581",
        phase_path="artifacts/audio_qa/local-loaded-recorder-riotbox-1581-v2-2026-10-16",
        document_name="loaded_recorder_host_riotbox_1581_v2.md",
        contract_name="loaded_recorder_host_riotbox_1581_v2.json",
        contract_sha256="197cbc4fb478c7d7bca2495909ba6acaeb846e26a262721a8d933df6a221f141",
        loaded_sha256="4cf9514cf33057d67e268f4ee6d234e083f4703d25a2f3d8537de2b3c99564ed",
        observation_version=support.LoadedObservationVersion.V2,
        protocol_files=(
            "loaded_recorder_host_riotbox_1581_v2.json",
            "loaded_recorder_observation_v2.json",
            "loaded_recorder_observation_v1.json",
            "generated_live_master_load_fixture_v1.json",
            "silent_host_observation_v3.json",
            "silent_host_observation_v4.json",
        ),
    )

    def test_phase_preserves_every_predecessor_limit_and_false_quality_claim(self):
        contracts = support.ROOT / "docs/benchmarks"
        previous = load_json_bytes((contracts / "loaded_recorder_host_riotbox_1579_v1.json").read_bytes())
        current = load_json_bytes((contracts / self.CONFIG.contract_name).read_bytes())
        expected = {**previous,
            "schema": "riotbox.loaded_recorder_host_phase.v2", "schema_version": 2,
            "predecessor_phase_protocol_sha256": "9a2a924fa0eab54d592d9d349ca72bd22eec6abba59bcb53b1e62a19be1272c4",
            "owner_ticket": self.CONFIG.ticket, "phase_path": self.CONFIG.phase_path,
            "loaded_protocol_sha256": self.CONFIG.loaded_sha256}
        require(current, expected, "complete V2 phase inheritance")

    def test_v1_loaded_hash_cannot_substitute_for_the_explicit_v2_binding(self):
        self.binding["loaded_protocol_sha256"] = support.PROTOCOL_SHA256
        self.write_binding()
        self.rejected_before_fixture()

    def test_v1_contract_bytes_cannot_substitute_for_the_v2_protocol_file(self):
        protocols = self.root / "docs/benchmarks"
        (protocols / "loaded_recorder_observation_v2.json").write_bytes(
            (protocols / "loaded_recorder_observation_v1.json").read_bytes())
        self.rejected_before_fixture()


if __name__ == "__main__":
    unittest.main()
