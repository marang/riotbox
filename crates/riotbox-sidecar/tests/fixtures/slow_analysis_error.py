import json
import sys
import time

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
request = json.loads(sys.stdin.readline())
time.sleep(0.075)
print(
    json.dumps(
        {
            "type": "error",
            "request_id": request["request_id"],
            "code": "fixture_complete",
            "message": "slow analysis response completed inside its own budget",
            "retryable": False,
        }
    ),
    flush=True,
)
