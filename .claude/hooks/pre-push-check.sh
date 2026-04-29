#!/bin/bash
set -euo pipefail

block() {
  cat <<JSON
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"$1"}}
JSON
  exit 0
}

echo "Running pre-push quality checks..." >&2

# gitleaks
echo "  gitleaks..." >&2
gitleaks protect --staged --no-banner >&2 || block "gitleaks: secrets detected"

# cargo fmt
echo "  cargo fmt check..." >&2
cargo fmt -- --check >&2 2>&1 || block "cargo fmt: formatting issues found"

# cargo clippy
echo "  cargo clippy..." >&2
cargo clippy -- -D warnings >&2 2>&1 || block "cargo clippy: warnings found"

echo "All checks passed." >&2
exit 0
