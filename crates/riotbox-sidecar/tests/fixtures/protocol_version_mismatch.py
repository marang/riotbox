import json
import sys

request = json.loads(sys.stdin.readline())
print(
    json.dumps(
        {
            "type": "pong",
            "request_id": request["request_id"],
            "protocol_version": "99.0",
            "sidecar_version": "fixture",
        }
    ),
    flush=True,
)
