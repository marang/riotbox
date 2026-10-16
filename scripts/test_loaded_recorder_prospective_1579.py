"""Frozen 1579 V1 fence adapter; all external work is generated and mocked."""

import unittest

import loaded_recorder_prospective_test_support as support


class LoadedProspective1579Tests(support.ProspectiveFenceTests):
    CONFIG = support.PhaseConfig(
        ticket="RIOTBOX-1579",
        phase_path="artifacts/audio_qa/local-loaded-recorder-riotbox-1579-v1-2026-10-15",
        document_name="loaded_recorder_host_riotbox_1579_v1.md",
        contract_name="loaded_recorder_host_riotbox_1579_v1.json",
        contract_sha256="9a2a924fa0eab54d592d9d349ca72bd22eec6abba59bcb53b1e62a19be1272c4",
        loaded_sha256="687bea31dfa903a299fc14b9e01567bf34b3cd4536220397f2cbb6f780e943ca",
        observation_version=support.LoadedObservationVersion.V1,
        protocol_files=(
            "loaded_recorder_host_riotbox_1579_v1.json",
            "loaded_recorder_observation_v1.json",
            "generated_live_master_load_fixture_v1.json",
            "silent_host_observation_v3.json",
            "silent_host_observation_v4.json",
        ),
    )


if __name__ == "__main__":
    unittest.main()
