#!/usr/bin/env bash
set -euo pipefail

if [ "${1:-}" = "" ]; then
  echo "usage: scripts/linear_issue_delete.sh <issue-id-or-identifier>" >&2
  echo "example: scripts/linear_issue_delete.sh RIOTBOX-96" >&2
  exit 1
fi

if [ "${LINEAR_API_TOKEN:-}" = "" ]; then
  echo "LINEAR_API_TOKEN is required" >&2
  exit 1
fi

issue_ref="$1"
endpoint="${LINEAR_GRAPHQL_ENDPOINT:-https://api.linear.app/graphql}"
payload="$(python - "$issue_ref" <<'PY'
import json
import sys

issue_ref = sys.argv[1]
query = """
mutation IssueDelete($id: String!) {
  issueDelete(id: $id) {
    success
    lastSyncId
  }
}
"""
print(json.dumps({
    "query": query,
    "variables": {"id": issue_ref},
    "operationName": "IssueDelete",
}))
PY
)"

response="$(
  curl --silent --show-error --fail \
    "$endpoint" \
    -H "Authorization: $LINEAR_API_TOKEN" \
    -H "Content-Type: application/json" \
    --data-raw "$payload"
)"

python - "$response" "$issue_ref" <<'PY'
import json
import sys

response = json.loads(sys.argv[1])
issue_ref = sys.argv[2]

if response.get("errors"):
    print(json.dumps(response["errors"], indent=2), file=sys.stderr)
    sys.exit(1)

payload = response.get("data", {}).get("issueDelete")
if not payload:
    print("Linear issueDelete returned no payload", file=sys.stderr)
    sys.exit(1)

if not payload.get("success"):
    print(f"Linear issueDelete did not confirm success for {issue_ref}", file=sys.stderr)
    sys.exit(1)

last_sync_id = payload.get("lastSyncId")
print(f"Deleted {issue_ref} from Linear (lastSyncId={last_sync_id})")
PY
