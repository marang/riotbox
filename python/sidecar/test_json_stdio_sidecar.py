import re
import unittest

import json_stdio_sidecar as sidecar


class SidecarClockContractTests(unittest.TestCase):
    def test_graph_generation_uses_injected_clock_once(self) -> None:
        calls = []

        def fixed_clock() -> str:
            calls.append("called")
            return "2030-01-02T03:04:05.678Z"

        response = sidecar.handle_message(
            {
                "type": "build_source_graph_stub",
                "request_id": "req-clock",
                "source": {
                    "source_id": "src-clock",
                    "path": "fixture.wav",
                    "content_hash": "sha256:clock",
                    "duration_seconds": 4.0,
                    "sample_rate": 48000,
                    "channel_count": 2,
                    "decode_profile": "NormalizedStereo",
                },
                "analysis_seed": 7,
            },
            clock=fixed_clock,
        )

        self.assertEqual(calls, ["called"])
        self.assertEqual(
            response["graph"]["provenance"]["generated_at"],
            "2030-01-02T03:04:05.678Z",
        )

    def test_production_clock_emits_utc_rfc3339_milliseconds(self) -> None:
        self.assertRegex(
            sidecar.utc_generated_at(),
            re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$"),
        )


if __name__ == "__main__":
    unittest.main()
