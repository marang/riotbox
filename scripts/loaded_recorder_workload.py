"""One known loaded workload for the reserved isolated operator, no public launcher."""

from dataclasses import dataclass

from loaded_recorder_evidence import PROTOCOL_SHA256, load_json_bytes, load_protocol, require, sha256
from loaded_recorder_fixture import GeneratedFixture
from silent_host_evidence import EvidenceError


@dataclass(frozen=True)
class LoadedRecorderWorkload:
    fixture: GeneratedFixture
    expected_binary_sha256: str

    def admit(self, root, binary, diagnostics):
        if type(self.fixture) is not GeneratedFixture or diagnostics is None:
            raise EvidenceError("loaded workload requires admitted generated fixture and diagnostics")
        protocol = load_protocol(root)
        require(sha256(binary.read_bytes()), self.expected_binary_sha256, "recorder binary")
        self.fixture.verify_unchanged()
        return protocol

    def prepare_cases(self, owner):
        return [self.fixture.prepare_case(bars, owner) for bars in (2, 8, 16)]

    def admission(self):
        return {"schema": "riotbox.loaded_recorder_attempt.v1", "protocol_sha256": PROTOCOL_SHA256,
                "fixture_report_sha256": self.fixture.report_sha256,
                "fixture_owner": str(self.fixture.owner), "quality_proof": False,
                "evidence_scope": "source_free_loaded_recording_adapter_preparation"}

    def run_case(self, binary, case, prefix, host, sink, protocol, environment, diagnostics):
        from loaded_recorder_process import run_recording
        require(sha256(binary.read_bytes()), self.expected_binary_sha256, "recorder binary")
        self.fixture.verify_case_before_launch(case)
        result = run_recording(binary, case, self.fixture, prefix, host, sink, protocol,
                               environment, diagnostics=diagnostics)
        # This existing mode loads only Core Session metadata, not App audio or
        # host/device services. Keep its semantic validator authoritative too.
        require(sha256(binary.read_bytes()), self.expected_binary_sha256, "recorder binary")
        data = host.text([str(binary), "--live-recording-readiness-report", "--session",
                          str(case.session_path)]).encode("utf-8")
        with prefix.with_suffix(".readiness.json").open("xb") as output:
            output.write(data)
        readiness = load_json_bytes(data)
        for key, expected in {
            "mode": "live_recording_readiness_report", "ready": True, "status": "ready",
            "writes_files": False, "runtime_master_recording_ready": True,
            "session_path": str(case.session_path), "readiness_blockers": [],
        }.items():
            require(readiness.get(key), expected, "Core readiness " + key)
        require(readiness["receipt"]["receipt_id"], result["result"]["receipt"]["receipt_id"],
                "Core readiness receipt")
        require(readiness["receipt"]["created_by_action"], result["marker"]["action_id"],
                "Core readiness action")
        self.fixture.verify_unchanged()
        require(sha256(binary.read_bytes()), self.expected_binary_sha256, "recorder binary")
        result["core_readiness_sha256"] = sha256(data)
        return result
