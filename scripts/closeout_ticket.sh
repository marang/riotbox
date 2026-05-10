#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: scripts/closeout_ticket.sh --ticket RIOTBOX-123 [options]

Safe closeout helper for completed Riotbox tickets.

Required:
  --ticket RIOTBOX-123        ticket identifier to verify and close out

Options:
  --branch NAME               feature branch to delete after PR merge is verified
  --pr NUMBER                 GitHub PR number used to verify branch deletion safety
  --delete-linear             delete the Linear issue after archive handoff verification
  --delete-remote-branch      delete origin/BRANCH after PR merge verification
  --delete-local-branch       delete local BRANCH after PR merge verification
  --mem-status                run MemPalace status after cleanup
  --mem-status-timeout SEC    bound optional MemPalace status; default: 120
  --execute                   perform cleanup actions; default is dry-run
  --dry-run                   print actions without mutating anything
  -h, --help                  show this help

Examples:
  scripts/archive_linear_issue.py --ticket RIOTBOX-123 --pr 99 --why "..." --shipped "..." --execute
  scripts/closeout_ticket.sh --ticket RIOTBOX-123
  scripts/closeout_ticket.sh --ticket RIOTBOX-123 --branch feature/riotbox-123-example --pr 99
  scripts/closeout_ticket.sh --ticket RIOTBOX-123 --branch feature/riotbox-123-example --pr 99 --delete-linear --delete-remote-branch --delete-local-branch --execute
EOF
}

die() {
  echo "closeout_ticket: $*" >&2
  exit 1
}

info() {
  echo "closeout_ticket: $*"
}

repo_root() {
  git rev-parse --show-toplevel 2>/dev/null
}

run_cmd() {
  if [ "$execute" -eq 1 ]; then
    "$@"
  else
    printf 'dry-run:'
    printf ' %q' "$@"
    printf '\n'
  fi
}

run_optional_timeout_cmd() {
  local timeout_seconds="$1"
  shift
  if [ "$execute" -eq 0 ]; then
    if command -v timeout >/dev/null 2>&1; then
      printf 'dry-run: timeout %qs' "$timeout_seconds"
    else
      printf 'dry-run:'
    fi
    printf ' %q' "$@"
    printf '\n'
    return 0
  fi

  if command -v timeout >/dev/null 2>&1; then
    if timeout "${timeout_seconds}s" "$@"; then
      return 0
    fi
    status="$?"
  else
    if "$@"; then
      return 0
    fi
    status="$?"
  fi

  if [ "$status" -eq 124 ]; then
    info "optional command timed out after ${timeout_seconds}s, continuing: $*"
  else
    info "optional command failed with status $status, continuing: $*"
  fi
}

protected_branch() {
  case "$1" in
    main|master|develop|development|trunk|release|release/*|hotfix/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

github_repo_full_name() {
  local url
  url="$(git remote get-url origin)"
  case "$url" in
    git@github.com:*.git)
      url="${url#git@github.com:}"
      echo "${url%.git}"
      ;;
    https://github.com/*.git)
      url="${url#https://github.com/}"
      echo "${url%.git}"
      ;;
    https://github.com/*)
      echo "${url#https://github.com/}"
      ;;
    *)
      return 1
      ;;
  esac
}

verify_pr_merged_for_branch() {
  local pr_number="$1"
  local branch_name="$2"
  local repo
  local endpoint
  local response
  local curl_args

  repo="$(github_repo_full_name)" || die "could not infer GitHub repo from origin remote"
  endpoint="https://api.github.com/repos/${repo}/pulls/${pr_number}"
  curl_args=(--silent --show-error --fail "$endpoint")
  if [ "${GITHUB_TOKEN:-}" != "" ]; then
    curl_args+=(-H "Authorization: Bearer ${GITHUB_TOKEN}")
  fi

  response="$(curl "${curl_args[@]}")"
  python3 - "$response" "$branch_name" "$pr_number" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
branch = sys.argv[2]
pr_number = sys.argv[3]

merged_at = payload.get("merged_at")
state = payload.get("state")
head_ref = (payload.get("head") or {}).get("ref")

if state != "closed" or not merged_at:
    raise SystemExit(f"PR #{pr_number} is not merged")
if head_ref != branch:
    raise SystemExit(
        f"PR #{pr_number} head branch mismatch: expected {branch!r}, got {head_ref!r}"
    )
PY
}

ticket=""
branch=""
pr_number=""
delete_linear=0
delete_remote_branch=0
delete_local_branch=0
mem_status=0
mem_status_timeout=120
execute=0

while [ "$#" -gt 0 ]; do
  case "$1" in
    --ticket)
      ticket="${2:-}"
      shift 2
      ;;
    --branch)
      branch="${2:-}"
      shift 2
      ;;
    --pr)
      pr_number="${2:-}"
      shift 2
      ;;
    --delete-linear)
      delete_linear=1
      shift
      ;;
    --delete-remote-branch)
      delete_remote_branch=1
      shift
      ;;
    --delete-local-branch)
      delete_local_branch=1
      shift
      ;;
    --mem-status)
      mem_status=1
      shift
      ;;
    --mem-status-timeout)
      mem_status_timeout="${2:-}"
      shift 2
      ;;
    --execute)
      execute=1
      shift
      ;;
    --dry-run)
      execute=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

[ "$ticket" != "" ] || {
  usage
  exit 1
}

[[ "$ticket" =~ ^RIOTBOX-[0-9]+$ ]] || die "ticket must look like RIOTBOX-123, got '$ticket'"
[[ "$mem_status_timeout" =~ ^[0-9]+$ ]] || die "--mem-status-timeout must be a positive integer"
[ "$mem_status_timeout" -gt 0 ] || die "--mem-status-timeout must be greater than zero"

root="$(repo_root)" || die "must run inside a git repository"
cd "$root"

if [ -f .env.local ]; then
  set -a
  # shellcheck disable=SC1091
  . ./.env.local
  set +a
fi

archive_file="docs/archive/linear_issues/${ticket}.md"
[ -f "$archive_file" ] || die "archive file missing: $archive_file"
rg --no-ignore -n "^- Ticket: \`${ticket}\`" "$archive_file" >/dev/null \
  || die "archive metadata missing in $archive_file"
rg --no-ignore -n "${ticket}.md" docs/archive/linear_issues/index.md >/dev/null \
  || die "archive index does not mention ${ticket}.md"
if rg --no-ignore -n "TODO: summarize (why this ticket existed|shipped behavior) before closeout" "$archive_file" >/dev/null; then
  die "archive still contains generator TODO placeholders: $archive_file"
fi
info "archive handoff ok for $ticket"

if [ "$delete_remote_branch" -eq 1 ] || [ "$delete_local_branch" -eq 1 ]; then
  [ "$branch" != "" ] || die "--branch is required for branch deletion"
  protected_branch "$branch" && die "refusing to delete protected branch: $branch"
  [ "$pr_number" != "" ] || die "--pr is required for branch deletion safety"
  if [ "$execute" -eq 1 ]; then
    verify_pr_merged_for_branch "$pr_number" "$branch"
    info "PR #$pr_number is merged and matches branch $branch"
  else
    info "dry-run: would verify PR #$pr_number is merged and matches branch $branch"
  fi
fi

if [ "$delete_remote_branch" -eq 1 ]; then
  run_cmd git fetch origin main --prune
  if git ls-remote --exit-code --heads origin "$branch" >/dev/null 2>&1; then
    run_cmd git push origin --delete "$branch"
  else
    info "remote branch not present, skipping: origin/$branch"
  fi
fi

if [ "$delete_local_branch" -eq 1 ]; then
  current_branch="$(git branch --show-current)"
  [ "$current_branch" != "$branch" ] || die "refusing to delete current branch: $branch"
  if git show-ref --verify --quiet "refs/heads/$branch"; then
    run_cmd git branch -D "$branch"
  else
    info "local branch not present, skipping: $branch"
  fi
fi

if [ "$delete_linear" -eq 1 ]; then
  run_cmd scripts/linear_issue_delete.sh "$ticket"
fi

if [ "$mem_status" -eq 1 ]; then
  if command -v just >/dev/null 2>&1; then
    run_optional_timeout_cmd "$mem_status_timeout" just mem-status
  else
    run_optional_timeout_cmd "$mem_status_timeout" scripts/mempalace.sh status
  fi
fi

if [ "$execute" -eq 1 ]; then
  info "closeout complete for $ticket"
else
  info "dry-run complete for $ticket; pass --execute to mutate Linear or branches"
fi
