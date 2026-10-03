"""Frozen per-source input budgets, not Python heap or process-wide ceilings."""

from enum import Enum


SOURCE_WAV_MAX_ENCODED_BYTES_V1 = 256 * 1024 * 1024
SOURCE_WAV_MAX_DECODED_SAMPLES_V1 = 64 * 1024 * 1024


class SourceResource(Enum):
    ENCODED_BYTES = "encoded bytes"
    DECODED_INTERLEAVED_SAMPLES = "decoded interleaved samples"


class SourceResourceLimitError(ValueError):
    def __init__(self, resource: SourceResource, required: int, limit: int) -> None:
        self.resource = resource
        self.required = required
        self.limit = limit
        super().__init__(
            f"source WAV {resource.value} exceed admission limit: required {required}, limit {limit}"
        )


def check_source_limit(resource: SourceResource, required: int, limit: int) -> None:
    if required > limit:
        raise SourceResourceLimitError(resource, required, limit)
