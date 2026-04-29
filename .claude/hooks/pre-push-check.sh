#!/bin/bash
set -euo pipefail

# hook receives tool input as JSON on stdin
INPUT=$(cat)
COMMAND=$(echo "$INPUT" | grep -o '"command":"[^"]*"' | head -1 | sed 's/"command":"//;s/"$//')

# Only run checks for git push commands
case "$COMMAND" in
  git\ push*) ;;
  *) exit 0 ;;
esac

echo "Running pre-push quality checks..." >&2

# gitleaks
echo "  gitleaks..." >&2
gitleaks protect --staged --no-banner >&2 || {
  echo '{"decision":"block","reason":"gitleaks: secrets detected"}'
  exit 0
}

# cargo fmt
echo "  cargo fmt check..." >&2
cargo fmt -- --check >&2 2>&1 || {
  echo '{"decision":"block","reason":"cargo fmt: formatting issues found"}'
  exit 0
}

# cargo clippy
echo "  cargo clippy..." >&2
cargo clippy -- -D warnings >&2 2>&1 || {
  echo '{"decision":"block","reason":"cargo clippy: warnings found"}'
  exit 0
}

echo "All checks passed." >&2
exit 0
