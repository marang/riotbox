"""Current-runtime phase adapter; generated controls, no actual host grant."""

import unittest

import loaded_recorder_prospective_test_support as support
from loaded_recorder_evidence import load_json_bytes, require


class LoadedProspective1589Tests(support.ProspectiveFenceTests):
    CONFIG = support.PhaseConfig(
        ticket="RIOTBOX-1589",
        phase_path="artifacts/audio_qa/local-loaded-recorder-riotbox-1589-v1-2026-10-16",
        document_name="loaded_recorder_host_riotbox_1589_v1.md",
        contract_name="loaded_recorder_host_riotbox_1589_v1.json",
        contract_sha256="a3f8a71e91a9a5f1892c39ab24b69666d8cd9a048cfcda97a920363617fc80c0",
        loaded_sha256="4cf9514cf33057d67e268f4ee6d234e083f4703d25a2f3d8537de2b3c99564ed",
        observation_version=support.LoadedObservationVersion.V2,
        protocol_files=(
            "loaded_recorder_host_riotbox_1589_v1.json",
            "loaded_recorder_observation_v2.json",
            "loaded_recorder_observation_v1.json",
            "generated_live_master_load_fixture_v1.json",
            "silent_host_observation_v3.json",
            "silent_host_observation_v4.json",
        ),
    )

    def test_phase_changes_only_owner_and_predecessor_not_observation_policy(self):
        contracts = support.ROOT / "docs/benchmarks"
        previous = load_json_bytes((contracts / "loaded_recorder_host_riotbox_1581_v2.json").read_bytes())
        current = load_json_bytes((contracts / self.CONFIG.contract_name).read_bytes())
        expected = {**previous,
            "predecessor_phase_protocol_sha256": "197cbc4fb478c7d7bca2495909ba6acaeb846e26a262721a8d933df6a221f141",
            "owner_ticket": self.CONFIG.ticket, "phase_path": self.CONFIG.phase_path}
        require(current, expected, "unchanged V2 observation policy")

    def test_executable_fence_changes_only_prospective_identity(self):
        document = (support.ROOT / "docs/benchmarks/loaded_recorder_host_riotbox_1581_v2.md").read_text()
        previous = document.split("```python\n", 1)[1].split("\n```", 1)[0]
        expected = (previous.replace("RIOTBOX-1581", self.CONFIG.ticket)
                    .replace("riotbox-1581-v2-2026-10-16", "riotbox-1589-v1-2026-10-16")
                    .replace("loaded_recorder_host_riotbox_1581_v2.json", self.CONFIG.contract_name)
                    .replace("197cbc4fb478c7d7bca2495909ba6acaeb846e26a262721a8d933df6a221f141",
                             self.CONFIG.contract_sha256))
        self.assertEqual(self.fence.co_code, compile(expected, "<predecessor>", "exec").co_code)
        current = (support.ROOT / "docs/benchmarks" / self.CONFIG.document_name).read_text()
        self.assertEqual(current.split("```python\n", 1)[1].split("\n```", 1)[0], expected)

    def test_consumed_predecessor_approval_cannot_authorize_this_owner(self):
        for ticket, version, protocol in (
            ("RIOTBOX-1579", "v1", "9a2a924fa0eab54d592d9d349ca72bd22eec6abba59bcb53b1e62a19be1272c4"),
            ("RIOTBOX-1581", "v2", "197cbc4fb478c7d7bca2495909ba6acaeb846e26a262721a8d933df6a221f141"),
        ):
            with self.subTest(ticket=ticket):
                path = f"artifacts/audio_qa/local-loaded-recorder-{ticket.lower()}-{version}-2026-10-16"
                self.authorization.update(owner_ticket=ticket, phase_path=str(self.root / path),
                                          phase_protocol_sha256=protocol)
                self.write_authorization()
                self.binding["authorization_sha256"] = support.operator.digest(self.authorization_path)
                self.write_binding()
                self.rejected_before_fixture()


if __name__ == "__main__":
    unittest.main()
