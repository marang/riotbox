"""Verify the bounded dependency-only correction against published source hashes."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
vendor = root / "vendor/dawproject"
expected = json.loads((vendor / "upstream-source-sha256.json").read_text())
# ZIP 9 makes filename decoding fallible. The sole library adaptation must
# propagate that error; reverse only this reviewed delta before hash comparison.
read_path = "src/read.rs"
read_source = (vendor / read_path).read_text()
patched = ".map(|name| name.map(|name| name.into_owned()))\n            .collect::<Result<Vec<String>, _>>()?"
original = ".map(|f| f.to_string())\n            .collect::<Vec<String>>()"
if read_source.count(patched) != 1:
    raise SystemExit("unexpected dawproject ZIP 9 filename adaptation")
actual = {
    path.relative_to(vendor).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
    for path in (vendor / "src").rglob("*") if path.is_file()
}
actual[read_path] = hashlib.sha256(read_source.replace(patched, original).encode()).hexdigest()
if actual != expected:
    raise SystemExit("vendored dawproject library differs from published 0.11.0 source")
print("dawproject: published library source hashes match after the reviewed ZIP 9 adaptation")
