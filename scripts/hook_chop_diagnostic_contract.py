"""Hook-forward diagnostic count evidence, never a musical approval.

RIOTBOX-1502 / RBX-383 reconciles the original two-reverse contract. Integer
JSON numbers and legacy integral floats are supported; absent, coerced or
impossible evidence is not manufactured into a count.
"""

import math
from collections.abc import Iterable
from typing import Any


MIN_HOOK_CHOP_RIFF_REVERSE_COUNT = 2


def _count(value: Any) -> bool:
    if type(value) is int:
        return value >= 0
    return (type(value) is float and math.isfinite(value)
            and value >= 0 and value.is_integer())


def reverse_count(reverse: Any, hits: Any) -> int | float | None:
    """Return supported evidence only if reverse hits fit inside total hits."""
    if not _count(reverse) or not _count(hits) or reverse > hits:
        return None
    return reverse


def proof_reverse_count(proof: dict[str, Any]) -> int | float | None:
    return reverse_count(proof.get("hook_chop_riff_reverse_count"),
                         proof.get("hook_chop_riff_hit_count"))


def minimum_reverse_count(proofs: Iterable[dict[str, Any]]) -> int | float | None:
    """All selected cases must have evidence; a missing case is not skipped."""
    counts = []
    for proof in proofs:
        count = proof_reverse_count(proof)
        if count is None:
            return None
        counts.append(count)
    return min(counts) if counts else None


def passes_reverse_count(reverse: Any, hits: Any) -> bool:
    count = reverse_count(reverse, hits)
    return count is not None and count >= MIN_HOOK_CHOP_RIFF_REVERSE_COUNT
