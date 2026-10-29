#!/usr/bin/env bash
set -euo pipefail
: "${GH_TOKEN:?GH_TOKEN erforderlich}"
REPO=erwintwin31-lab/appmenu
gh api -X PUT "repos/$REPO" -f allow_squash_merge=true -f allow_merge_commit=false -f allow_rebase_merge=false
for spec in "bug:d03638:Bug" "enhancement:a2eeef:Feature" "ci:0e8a16:CI" "documentation:0075ca:Dokumentation" "good first issue:7057ff:Guter Einstieg" "help wanted:008672:Hilfe gesucht"; do
  IFS=: read -r name color desc <<<"$spec"
  gh label create "$name" --repo "$REPO" --color "$color" --description "$desc" || true
done
gh api -X PUT "repos/$REPO/branches/main/protection" --input - <<'JSON'
{
  "required_status_checks": {"strict": true, "contexts": ["CI / gate"]},
  "enforce_admins": false,
  "required_pull_request_reviews": {"required_approving_review_count": 1, "dismiss_stale_reviews": true},
  "restrictions": null,
  "allow_force_pushes": false,
  "allow_deletions": false
}
JSON
echo "Repository-Governance aktiv."
