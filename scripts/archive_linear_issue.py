#!/usr/bin/env python3
"""Generate a per-ticket Linear archive file and update archive indexes."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


TICKET_RE = re.compile(r"^RIOTBOX-\d+$")


def die(message: str) -> None:
    print(f"archive_linear_issue: {message}", file=sys.stderr)
    raise SystemExit(1)


def info(message: str) -> None:
    print(f"archive_linear_issue: {message}")


def repo_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        stdout=subprocess.PIPE,
    )
    return Path(result.stdout.strip())


def load_dotenv(path: Path) -> None:
    if not path.exists():
        return
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip().strip('"').strip("'")
        os.environ.setdefault(key, value)


def iso_date(value: str | None) -> str:
    if not value:
        return "Unknown"
    normalized = value.replace("Z", "+00:00")
    return datetime.fromisoformat(normalized).date().isoformat()


def today_iso() -> str:
    return datetime.now(timezone.utc).date().isoformat()


def validate_iso_date(name: str, value: str) -> None:
    try:
        parsed = datetime.strptime(value, "%Y-%m-%d")
    except ValueError:
        die(f"{name} must use YYYY-MM-DD, got {value!r}")
    if parsed.date().isoformat() != value:
        die(f"{name} must use YYYY-MM-DD, got {value!r}")


def repo_full_name(root: Path) -> str:
    result = subprocess.run(
        ["git", "remote", "get-url", "origin"],
        check=True,
        cwd=root,
        text=True,
        stdout=subprocess.PIPE,
    )
    url = result.stdout.strip()
    if url.startswith("git@github.com:"):
        return url.removeprefix("git@github.com:").removesuffix(".git")
    if url.startswith("https://github.com/"):
        return url.removeprefix("https://github.com/").removesuffix(".git")
    die(f"could not infer GitHub repo from origin remote: {url}")


def graphql(endpoint: str, token: str, query: str, variables: dict[str, object]) -> dict:
    payload = json.dumps({"query": query, "variables": variables}).encode()
    request = urllib.request.Request(
        endpoint,
        data=payload,
        headers={
            "Authorization": token,
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        die(f"Linear request failed: {exc.code} {detail}")
    data = json.loads(body)
    if data.get("errors"):
        die(json.dumps(data["errors"], indent=2))
    return data["data"]


def github_json(path: str, token: str | None) -> dict:
    headers = {
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    request = urllib.request.Request(f"https://api.github.com{path}", headers=headers)
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            return json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as exc:
        detail = exc.read().decode("utf-8", errors="replace")
        die(f"GitHub request failed: {exc.code} {detail}")


@dataclass
class LinearIssue:
    identifier: str
    title: str
    url: str
    created: str
    started: str
    finished: str
    status: str
    project: str
    milestone: str
    branch_name: str
    assignee: str
    labels: str
    description: str


def fetch_linear_issue(ticket: str) -> LinearIssue:
    token = os.getenv("LINEAR_API_TOKEN")
    if not token:
        die("LINEAR_API_TOKEN is required")
    endpoint = os.getenv("LINEAR_GRAPHQL_ENDPOINT", "https://api.linear.app/graphql")
    query = """
    query Issue($id: String!) {
      issue(id: $id) {
        identifier
        title
        url
        description
        createdAt
        startedAt
        completedAt
        canceledAt
        archivedAt
        branchName
        state { name type }
        project { name }
        projectMilestone { name }
        assignee { displayName name }
        labels { nodes { name } }
      }
    }
    """
    issue = graphql(endpoint, token, query, {"id": ticket}).get("issue")
    if not issue:
        die(f"Linear issue not found: {ticket}")

    state = issue.get("state") or {}
    state_type = state.get("type")
    status = "Done" if state_type == "completed" else state.get("name") or "Unknown"
    finished = (
        iso_date(issue.get("completedAt"))
        if issue.get("completedAt")
        else iso_date(issue.get("canceledAt") or issue.get("archivedAt"))
    )
    assignee = issue.get("assignee") or {}
    labels = sorted(label["name"] for label in (issue.get("labels") or {}).get("nodes", []))
    return LinearIssue(
        identifier=issue["identifier"],
        title=issue["title"],
        url=issue["url"],
        description=issue.get("description") or "",
        created=iso_date(issue.get("createdAt")),
        started=iso_date(issue.get("startedAt")),
        finished=finished,
        status=status,
        project=(issue.get("project") or {}).get("name") or "None",
        milestone=(issue.get("projectMilestone") or {}).get("name") or "None",
        branch_name=issue.get("branchName") or "None",
        assignee=assignee.get("name") or assignee.get("displayName") or "Unassigned",
        labels=", ".join(f"`{label}`" for label in labels) if labels else "None",
    )


@dataclass
class PullRequest:
    number: int
    url: str
    branch: str
    merge_commit: str
    merged: bool


def fetch_pr(root: Path, pr_number: int) -> PullRequest:
    repo = repo_full_name(root)
    payload = github_json(f"/repos/{repo}/pulls/{pr_number}", os.getenv("GITHUB_TOKEN"))
    return PullRequest(
        number=pr_number,
        url=payload.get("html_url") or f"https://github.com/{repo}/pull/{pr_number}",
        branch=(payload.get("head") or {}).get("ref") or "None",
        merge_commit=payload.get("merge_commit_sha") or "Unknown",
        merged=bool(payload.get("merged_at")),
    )


def md_list(values: list[str]) -> str:
    return "\n".join(f"- {value}" for value in values)


def archive_body(
    issue: LinearIssue,
    *,
    branch: str,
    linear_branch: str,
    pr: PullRequest | None,
    merge_commit: str,
    finished: str,
    deleted_from_linear: str,
    verification: list[str],
    docs_touched: list[str],
    followups: str,
    why: str,
    shipped: list[str],
    notes: list[str],
) -> str:
    pr_field = "None"
    if pr:
        pr_field = f"`#{pr.number} ({pr.url})`"
    elif merge_commit != "None":
        pr_field = "None"

    return f"""# `{issue.identifier}` {issue.title}

- Ticket: `{issue.identifier}`
- Title: `{issue.title}`
- Linear issue: `{issue.url}`
- Project: `{issue.project}`
- Milestone: `{issue.milestone}`
- Status: `{issue.status}`
- Created: `{issue.created}`
- Started: `{issue.started}`
- Finished: `{finished}`
- Branch: `{branch}`
- Linear branch: `{linear_branch}`
- Assignee: `{issue.assignee}`
- Labels: {issue.labels}
- PR: {pr_field}
- Merge commit: `{merge_commit}`
- Deleted from Linear: `{deleted_from_linear}`
- Verification: {"; ".join(f"`{item}`" for item in verification) if verification else "`Not recorded`"}
- Docs touched: {", ".join(f"`{item}`" for item in docs_touched) if docs_touched else "`None`"}
- Follow-ups: `{followups}`

## Why This Ticket Existed

{why}

## What Shipped

{md_list(shipped)}

## Notes

{md_list(notes) if notes else "- None"}
"""


def insert_index_entry(index_path: Path, ticket: str, title: str) -> bool:
    entry = f"- [{ticket}.md](./{ticket}.md)\n  {title}"
    if not index_path.exists():
        index_path.write_text(
            "# Linear Issue Archive Index\n\n"
            "This index tracks archived Linear ticket history that has been removed from the active Linear workspace.\n\n"
            "## Archive Files\n\n"
            f"{entry}\n"
            "- Use [TEMPLATE.md](./TEMPLATE.md) for one-file ticket entries.\n",
            encoding="utf-8",
        )
        return True
    text = index_path.read_text(encoding="utf-8")
    if f"[{ticket}.md]" in text:
        return False
    marker = "- Use [TEMPLATE.md](./TEMPLATE.md) for one-file ticket entries."
    if marker in text:
        text = text.replace(marker, f"{entry}\n{marker}")
    else:
        text = text.rstrip() + f"\n{entry}\n"
    index_path.write_text(text, encoding="utf-8")
    return True


def insert_month_entry(month_path: Path, ticket: str) -> bool:
    entry = f"- [{ticket}.md](./{ticket}.md)"
    if not month_path.exists():
        month_path.write_text(
            f"# {month_path.stem} Linear Ticket Archive\n\n"
            "This month archive is split into one file per archived Linear ticket. "
            "Default repo searches exclude this archive through `.rgignore`; use explicit archive search when ticket history is needed.\n\n"
            "## Ticket Files\n\n"
            f"{entry}\n",
            encoding="utf-8",
        )
        return True
    text = month_path.read_text(encoding="utf-8")
    if entry in text:
        return False
    month_header = "## Ticket Files\n"
    if month_header in text:
        prefix, suffix = text.split(month_header, 1)
        lines = [line for line in suffix.strip().splitlines() if line.strip()]
        lines.append(entry)
        lines.sort(key=lambda line: int(re.search(r"RIOTBOX-(\d+)", line).group(1)))
        text = prefix + month_header + "\n" + "\n".join(lines) + "\n"
    else:
        text = text.rstrip() + f"\n\n## Ticket Files\n\n{entry}\n"
    month_path.write_text(text, encoding="utf-8")
    return True


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate a Riotbox Linear ticket archive and update indexes."
    )
    parser.add_argument("--ticket", required=True, help="Linear issue identifier, e.g. RIOTBOX-123")
    parser.add_argument("--branch", help="actual repo branch used for the work")
    parser.add_argument("--linear-branch", help="Linear-generated branch name")
    parser.add_argument("--pr", type=int, help="GitHub PR number")
    parser.add_argument("--merge-commit", help="merge commit hash")
    parser.add_argument("--finished", help="finished date, YYYY-MM-DD")
    parser.add_argument("--deleted-from-linear", default=today_iso(), help="deletion date, YYYY-MM-DD")
    parser.add_argument("--verification", action="append", default=[], help="verification command/result")
    parser.add_argument("--docs-touched", action="append", default=[], help="doc path touched by the ticket")
    parser.add_argument("--follow-ups", default="None", help="follow-up tickets or open questions")
    parser.add_argument("--why", help="why this ticket existed")
    parser.add_argument("--shipped", action="append", default=[], help="concrete shipped behavior")
    parser.add_argument("--notes", action="append", default=[], help="important note or tradeoff")
    parser.add_argument("--allow-placeholders", action="store_true", help="allow TODO placeholders")
    parser.add_argument("--force", action="store_true", help="overwrite an existing archive file")
    parser.add_argument("--execute", action="store_true", help="write files; default is dry-run")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    ticket = args.ticket.upper()
    if not TICKET_RE.match(ticket):
        die(f"ticket must look like RIOTBOX-123, got {args.ticket!r}")
    root = repo_root()
    os.chdir(root)
    load_dotenv(root / ".env.local")

    issue = fetch_linear_issue(ticket)
    pr = fetch_pr(root, args.pr) if args.pr else None

    branch = args.branch or (pr.branch if pr else issue.branch_name)
    linear_branch = args.linear_branch or issue.branch_name
    merge_commit = args.merge_commit or (pr.merge_commit if pr else "None")
    finished = args.finished or issue.finished
    why = args.why or issue.description.strip()
    shipped = args.shipped

    validate_iso_date("--deleted-from-linear", args.deleted_from_linear)
    if args.finished:
        validate_iso_date("--finished", args.finished)

    if pr and not pr.merged:
        die(f"PR #{pr.number} is not merged; archive generation is for closeout-ready tickets")
    if not args.allow_placeholders:
        if not why:
            die("--why is required when the Linear description is empty")
        if not shipped:
            die("at least one --shipped entry is required")
        if finished == "Unknown":
            die("--finished is required when Linear has no completed/canceled/archive date")
    else:
        why = why or "TODO: summarize why this ticket existed before closeout."
        shipped = shipped or ["TODO: summarize shipped behavior before closeout."]

    archive_dir = root / "docs" / "archive" / "linear_issues"
    archive_file = archive_dir / f"{ticket}.md"
    month = finished[:7] if finished != "Unknown" else today_iso()[:7]
    month_file = archive_dir / f"{month}.md"
    index_file = archive_dir / "index.md"

    if archive_file.exists() and not args.force:
        die(f"archive file already exists: {archive_file}")

    body = archive_body(
        issue,
        branch=branch,
        linear_branch=linear_branch,
        pr=pr,
        merge_commit=merge_commit,
        finished=finished,
        deleted_from_linear=args.deleted_from_linear,
        verification=args.verification,
        docs_touched=args.docs_touched,
        followups=args.follow_ups,
        why=why,
        shipped=shipped,
        notes=args.notes,
    )

    if not args.execute:
        info(f"dry-run: would write {archive_file}")
        info(f"dry-run: would update {month_file}")
        info(f"dry-run: would update {index_file}")
        print(body)
        return 0

    archive_dir.mkdir(parents=True, exist_ok=True)
    archive_file.write_text(body, encoding="utf-8")
    insert_month_entry(month_file, ticket)
    insert_index_entry(index_file, ticket, issue.title)
    info(f"wrote {archive_file}")
    info(f"updated {month_file}")
    info(f"updated {index_file}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
