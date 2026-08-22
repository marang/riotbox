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
sys.stdin.readline()
time.sleep(5)
