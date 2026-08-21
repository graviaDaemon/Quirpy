#!/usr/bin/env bash
set -euo pipefail

OWNER="graviaDaemon"
REPO="Quirpy"
API="https://api.github.com"
DESCRIPTION="A QR code generator that builds its codes by hand — Rust, cross-platform, fully local"

DRY_RUN=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=1 ;;
    -h|--help)
      echo "usage: $0 [--dry-run]"
      echo
      echo "Applies Quirpy's GitHub repository settings, labels and rulesets."
      echo "Reads the token from \$QUIRPY_GH_TOKEN, falling back to ~/.quirpy-gh-token."
      echo "See scripts/README.md for how to create that token."
      exit 0
      ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

TOKEN="${QUIRPY_GH_TOKEN:-}"
if [ -z "$TOKEN" ] && [ -r "$HOME/.quirpy-gh-token" ]; then
  TOKEN=$(tr -d '\r\n' < "$HOME/.quirpy-gh-token")
fi
if [ -z "$TOKEN" ]; then
  cat >&2 <<'MSG'
No token found.

Set QUIRPY_GH_TOKEN, or place a fine-grained personal access token in ~/.quirpy-gh-token.
The token needs, on this repository only:

  Administration  Read and write
  Issues          Read and write
  Metadata        Read-only (mandatory)

scripts/README.md walks through creating, storing and revoking it.
MSG
  exit 1
fi

say() { printf '\n== %s\n' "$1"; }
plan() { printf '   %s\n' "$1"; }

# api <METHOD> <PATH> [<JSON BODY>] -> prints "<http_code>\n<body>"
api() {
  local method="$1" path="$2" body="${3:-}"
  local tmp
  tmp=$(mktemp)
  local args=(-sS -o "$tmp" -w '%{http_code}'
    -X "$method" -H "Accept: application/vnd.github+json"
    -H "Authorization: Bearer $TOKEN"
    -H "X-GitHub-Api-Version: 2022-11-28")
  if [ -n "$body" ]; then
    args+=(-H "Content-Type: application/json" -d "$body")
  fi
  local code
  code=$(curl "${args[@]}" "$API$path")
  printf '%s\n' "$code"
  cat "$tmp"
  rm -f "$tmp"
}

# call <label> <METHOD> <PATH> [<BODY>] [<extra ok code>]
call() {
  local label="$1" method="$2" path="$3" body="${4:-}" extra_ok="${5:-}"
  if [ "$DRY_RUN" -eq 1 ]; then
    printf '   DRY-RUN %s %s  (%s)\n' "$method" "$path" "$label"
    [ -n "$body" ] && printf '%s\n' "$body" | jq . | sed 's/^/           /'
    return 0
  fi
  local out code
  out=$(api "$method" "$path" "$body")
  code=$(printf '%s' "$out" | head -n 1)
  case "$code" in
    2*) printf '   ok  (%s) %s\n' "$code" "$label" ;;
    *)
      if [ -n "$extra_ok" ] && [ "$code" = "$extra_ok" ]; then
        printf '   ok  (%s, already present) %s\n' "$code" "$label"
      else
        printf '   FAILED (%s) %s\n' "$code" "$label" >&2
        printf '%s\n' "$out" | tail -n +2 >&2
        return 1
      fi
      ;;
  esac
}

if [ "$DRY_RUN" -eq 1 ]; then
  echo "DRY RUN — nothing will be changed. Target: $OWNER/$REPO"
else
  echo "Applying settings to $OWNER/$REPO"
fi

say "1. Repository settings"
plan "squash-merge only, delete branch on merge, squash commit from PR title + body"
plan "issues on, discussions on, wiki off, projects off, description set"
SETTINGS=$(jq -n --arg d "$DESCRIPTION" '{
  description: $d,
  allow_squash_merge: true,
  allow_merge_commit: false,
  allow_rebase_merge: false,
  delete_branch_on_merge: true,
  squash_merge_commit_title: "PR_TITLE",
  squash_merge_commit_message: "PR_BODY",
  has_issues: true,
  has_discussions: true,
  has_wiki: false,
  has_projects: false
}')
call "repository settings" PATCH "/repos/$OWNER/$REPO" "$SETTINGS"

say "2. Topics"
plan "rust, qr-code, qr-generator, egui, gui, cross-platform"
TOPICS='{"names":["rust","qr-code","qr-generator","egui","gui","cross-platform"]}'
call "topics" PUT "/repos/$OWNER/$REPO/topics" "$TOPICS"

say "3. Private vulnerability reporting"
plan "enable, so the SECURITY.md advisory link works"
call "private vulnerability reporting" PUT "/repos/$OWNER/$REPO/private-vulnerability-reporting"

say "4. Labels"
plan "internal, no-issue-needed, needs-triage (existing labels are left alone)"
label() {
  local name="$1" color="$2" desc="$3"
  local body
  body=$(jq -n --arg n "$name" --arg c "$color" --arg d "$desc" \
    '{name: $n, color: $c, description: $d}')
  call "label $name" POST "/repos/$OWNER/$REPO/labels" "$body" 422
}
label "internal" "ededed" "Refactors, CI, tooling — anything users do not see"
label "no-issue-needed" "c5def5" "Trivial change exempt from the linked-issue check"
label "needs-triage" "fbca04" "Not yet looked at by the maintainer"

# ruleset <name> <json>  — creates, or updates the existing ruleset with that name
ruleset() {
  local name="$1" body="$2" id=""
  if [ "$DRY_RUN" -eq 0 ]; then
    local out code
    out=$(api GET "/repos/$OWNER/$REPO/rulesets")
    code=$(printf '%s' "$out" | head -n 1)
    case "$code" in
      2*) id=$(printf '%s' "$out" | tail -n +2 | jq -r --arg n "$name" \
             '[.[] | select(.name == $n)][0].id // empty') ;;
      *) printf '   FAILED (%s) listing rulesets\n' "$code" >&2
         printf '%s\n' "$out" | tail -n +2 >&2
         return 1 ;;
    esac
  fi
  if [ -n "$id" ]; then
    call "ruleset '$name' (update id $id)" PUT "/repos/$OWNER/$REPO/rulesets/$id" "$body"
  else
    call "ruleset '$name' (create)" POST "/repos/$OWNER/$REPO/rulesets" "$body"
  fi
}

say "5. Branch ruleset 'main'"
plan "PRs required (0 approvals), 'ci' and 'pr-linked-issue' required and up to date"
plan "force-push and deletion blocked; repository admin bypasses"
MAIN_RULESET=$(jq -n '{
  name: "main",
  target: "branch",
  enforcement: "active",
  conditions: { ref_name: { include: ["~DEFAULT_BRANCH"], exclude: [] } },
  bypass_actors: [{ actor_id: 5, actor_type: "RepositoryRole", bypass_mode: "always" }],
  rules: [
    { type: "deletion" },
    { type: "non_fast_forward" },
    { type: "pull_request",
      parameters: {
        required_approving_review_count: 0,
        dismiss_stale_reviews_on_push: false,
        require_code_owner_review: false,
        require_last_push_approval: false,
        required_review_thread_resolution: false
      } },
    { type: "required_status_checks",
      parameters: {
        strict_required_status_checks_policy: true,
        required_status_checks: [
          { context: "ci" },
          { context: "pr-linked-issue" }
        ]
      } }
  ]
}')
ruleset "main" "$MAIN_RULESET"

say "6. Tag ruleset 'release tags'"
plan "only the repository admin may create, delete or move refs/tags/v*"
TAG_RULESET=$(jq -n '{
  name: "release tags",
  target: "tag",
  enforcement: "active",
  conditions: { ref_name: { include: ["refs/tags/v*"], exclude: [] } },
  bypass_actors: [{ actor_id: 5, actor_type: "RepositoryRole", bypass_mode: "always" }],
  rules: [
    { type: "creation" },
    { type: "deletion" },
    { type: "non_fast_forward" }
  ]
}')
ruleset "release tags" "$TAG_RULESET"

echo
if [ "$DRY_RUN" -eq 1 ]; then
  echo "Dry run complete. Re-run without --dry-run to apply."
else
  cat <<'MSG'
Done.

Check Settings > Rules in the browser and confirm the bypass actor on both rulesets is
"Repository admin" — a wrong bypass actor is the one mistake here that locks you out of main.

Then delete ~/.quirpy-gh-token and revoke the token; it has no further use.
MSG
fi
