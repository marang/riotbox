#!/usr/bin/env python3
"""Hermetic Stage-A legacy contract fixtures v1, not historical attestation."""

import argparse
from contextlib import contextmanager
import copy
import hashlib
from pathlib import Path
import sys
from types import ModuleType


ROOT = Path(__file__).resolve().parents[1]
PINS = {
    "scripts/validate_percussive_force_stage_a_protocol_v3.py": "89a3f594accbf97d3af3775f8c806edec5779bd67fffe9f4cd0c3cd73a5d0323",
    "scripts/validate_percussive_force_stage_a_protocol_v4.py": "f89eed62e0bca86c01b5ec17e0f5ed4a2fe937e9f9ea2a6db5f2f7164c6a5cbe",
    "scripts/validate_percussive_force_stage_a_protocol_v5.py": "dbe1f6fd5882874859a9041c7b0a46fa47d2b890927133bb24e74ffebf0f4d5d",
    "docs/benchmarks/percussive_force_stage_a_protocol_v2.json": "b6b35cb14ef34be7f9b7bb6b2bf076ba84842c56914485937f088539e6217878",
    "docs/benchmarks/percussive_force_stage_a_protocol_v3.json": "21f716e3aeb6c9198671e34be21e225585a41135f875bccedb3c57df625e7eb4",
    "docs/benchmarks/percussive_force_stage_a_protocol_v4.json": "0f7bff0744a0b229136f192b93ea1e537849c30f2906ab9247e279baf567e724",
    "docs/benchmarks/percussive_force_stage_a_protocol_v5.json": "455440aabc1a433bbc7fbcc2093b85f6d1c66e1bba081526e082c50ed8248519",
    "docs/benchmarks/percussive_force_stage_a_source_pool_v1.json": "9c729eaac4156cb556d9f2538635e04829b3ffec3ad012d742fb765ad1e2a8ba",
    "docs/benchmarks/percussive_force_stage_a_source_pool_v2.json": "e82f21c965678ba8fe1937ee0652ad8456ac73ddb66c2956a871fc26d0404505",
    "docs/benchmarks/percussive_force_development_matrix_template_v4.json": "0bdff640827e92fba9675ab9ede69dea5789bf0a6f3702bcf84254a9eec85df3",
    "docs/benchmarks/percussive_force_development_matrix_template_v5.json": "2fcdcab61b72858419ae53aa9f74fb7a4fb6fd61c3889c6511d0c385b887201e",
    "docs/benchmarks/source_holdout_rotation_v3.json": "9e5e03ad64319061a4baaa6cee7c40fc5e993171b0d11003ec29767f273bc502",
}


def pinned_validators():
    # Check code as well as metadata before importing/executing legacy modules.
    checked = {}
    for relative, expected in PINS.items():
        path = ROOT / relative
        if path.resolve() != path or not path.is_file():
            raise ValueError(f"tracked fixture input is not a regular unlinked path: {relative}")
        payload = path.read_bytes()
        if hashlib.sha256(payload).hexdigest() != expected:
            raise ValueError(f"tracked fixture input pin changed: {relative}")
        checked[relative] = payload
    # Execute exactly the checked source, never ignored/stale __pycache__ bytes.
    modules = []
    for version in (3, 4, 5):
        name = f"validate_percussive_force_stage_a_protocol_v{version}"
        relative = f"scripts/{name}.py"
        module = ModuleType(name)
        module.__file__ = str(ROOT / relative)
        sys.modules[name] = module
        exec(compile(checked[relative], module.__file__, "exec"), module.__dict__)
        modules.append(module)
    return tuple(modules)


def synthetic_v3_terminal():
    return {
        "schema": "riotbox.synthetic_legacy_terminal_contract_fixture.v1",
        "synthetic_contract_fixture": True,
        "state": "rejected_fail_closed", "requests_attempted": 7, "requests_completed": 6,
        "failure": "pool[7]: PCM fmt chunk must be exactly 16 bytes",
        "pcm_sample_iteration": False, "source_audio_playback": False,
        "holdout_audio_access": False, "commercial_reference_access": False,
    }


def run_v4(v3, v4):
    documents = tuple(v4.load(path)[0] for path in (v4.PROTOCOL_PATH, v4.POOL_PATH, v4.MATRIX_PATH))
    base = (*documents, synthetic_v3_terminal())
    v4.validate(*base)
    v4.fixtures(base)  # All six existing mutations, unchanged.
    terminal_controls(v3, lambda access: v4.validate(*documents, access), base[-1], "v3", (
        ("commercial", lambda value: value.__setitem__("commercial_reference_access", True)),
    ))


def terminal_controls(v3, validate, base, prefix, extra):
    cases = (
        ("state", lambda value: value.__setitem__("state", "admitted")),
        ("attempted", lambda value: value.__setitem__("requests_attempted", 0)),
        ("completed", lambda value: value.__setitem__("requests_completed", 99)),
        ("failure", lambda value: value.__setitem__("failure", "wrong_terminal_reason")),
        ("pcm", lambda value: value.__setitem__("pcm_sample_iteration", True)),
        ("playback", lambda value: value.__setitem__("source_audio_playback", True)),
        ("holdout", lambda value: value.__setitem__("holdout_audio_access", True)),
        *extra,
    )
    for name, mutate in cases:
        changed = copy.deepcopy(base)
        mutate(changed)
        try:
            validate(changed)
        except v3.ContractError:
            print(f"PASS terminal mutation {prefix}_{name}")
        else:
            raise v3.ContractError(f"terminal mutation unexpectedly passed: {prefix}_{name}")


def synthetic_v4_terminal():
    return {
        "schema": "riotbox.synthetic_legacy_terminal_contract_fixture.v1",
        "synthetic_contract_fixture": True,
        "state": "rejected_fail_closed", "requests_attempted": 1, "requests_completed": 0,
        "failure": "request_7_strict_header_incoherent_fmt_extension_size",
        "entries": [{"ordinal": 7, "state": "rejected_strict_header_incoherent_fmt_extension_size"}],
        "pcm_sample_iteration": False, "source_audio_playback": False, "holdout_audio_access": False,
    }


@contextmanager
def synthetic_v3_loader(v4):
    """Only v5.validate's exact nested terminal-receipt load is substituted."""
    original = v4.load
    tracked = {v4.PROTOCOL_PATH, v4.POOL_PATH, v4.MATRIX_PATH}

    def fixture_load(path):
        if path == v4.ACCESS_PATH:
            # No historical raw bytes exist or are asserted by this fixture.
            return synthetic_v3_terminal(), b""
        if path not in tracked:
            raise ValueError(f"unregistered synthetic fixture load: {path}")
        return original(path)

    v4.load = fixture_load
    try:
        yield
    finally:
        v4.load = original


def run_v5(v3, v4, v5):
    protocol = v3.load_json(ROOT / v5.PROTOCOL)[0]
    access = synthetic_v4_terminal()
    with synthetic_v3_loader(v4):
        v5.validate(protocol, access)
        # The four existing v5 --fixtures mutations, unchanged.
        for name, mutation in (
            ("request", lambda value: value["source_pool_v3"]["exact_request_ordinals"].append(16)),
            ("retry", lambda value: value["access"].__setitem__("automatic_retries", True)),
            ("algorithm", lambda value: value["unchanged_contracts"].__setitem__("algorithm_source_raw_sha256", "0" * 64)),
            ("matrix", lambda value: value["qualification_and_matrix"].__setitem__("cross_product", "wrong")),
        ):
            changed = copy.deepcopy(protocol)
            mutation(changed)
            try:
                v5.validate(changed, access)
            except v3.ContractError:
                print(f"PASS mutation {name}")
            else:
                raise v3.ContractError(f"mutation unexpectedly passed: {name}")
        terminal_controls(v3, lambda value: v5.validate(protocol, value), access, "v4", (
            ("ordinal", lambda value: value["entries"][0].__setitem__("ordinal", 8)),
            ("entry_state", lambda value: value["entries"][0].__setitem__("state", "admitted")),
        ))


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", choices=("v4", "v5"), required=True)
    args = parser.parse_args(argv)
    try:
        v3, v4, v5 = pinned_validators()
        if args.contract == "v4":
            run_v4(v3, v4)
        else:
            run_v5(v3, v4, v5)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"FAIL: {error}")
        return 1
    print(f"PASS: synthetic Stage-A-{args.contract} contract fixtures v1")
    print("historical_receipts_verified=false")
    print("source_audio_accessed=false")
    print("holdout_audio_accessed=false")
    print("commercial_reference_accessed=false")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
