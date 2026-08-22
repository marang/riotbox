import json
import sys

ping = json.loads(sys.stdin.readline())
print(
    json.dumps(
        {
            "type": "pong",
            "request_id": ping["request_id"],
            "protocol_version": "0.1",
            "sidecar_version": "fixture",
        }
    ),
    flush=True,
)
sys.stdin.readline()
print(
    json.dumps(
        {
            "type": "error",
            "request_id": None,
            "code": "invalid_json",
            "message": "malformed provider response",
            "retryable": False,
        }
    ),
    flush=True,
)
