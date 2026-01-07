#!/usr/bin/env bash
set -euo pipefail

MAIN_BRANCH="main"
PAGES_BRANCH="gh-pages"
REMOTE="origin"

ROOT_DIR="$(git rev-parse --show-toplevel 2>/dev/null)" || {
  echo "❌ Not inside a git repository."
  exit 1
}

cd "$ROOT_DIR"

ORIG_BRANCH="$(git rev-parse --abbrev-ref HEAD)"

cleanup() {
  # Always try to return to the original branch
  git switch -q "$ORIG_BRANCH" 2>/dev/null || true
}
trap cleanup EXIT

echo "📍 Repo: $ROOT_DIR"
echo "🔎 Current branch: $ORIG_BRANCH"

# Ensure clean working tree
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "❌ Working tree has uncommitted changes. Commit or stash before deploying."
  exit 1
fi

echo "🔄 Fetching from $REMOTE..."
git fetch "$REMOTE" --prune

# Ensure main branch exists locally and remotely
git show-ref --verify --quiet "refs/heads/$MAIN_BRANCH" || {
  echo "❌ Local branch '$MAIN_BRANCH' not found."
  exit 1
}
git show-ref --verify --quiet "refs/remotes/$REMOTE/$MAIN_BRANCH" || {
  echo "❌ Remote branch '$REMOTE/$MAIN_BRANCH' not found."
  exit 1
}

# Switch to main first (so checks are consistent)
git switch -q "$MAIN_BRANCH"

# Check if local main is behind origin/main
LOCAL_MAIN="$(git rev-parse "$MAIN_BRANCH")"
REMOTE_MAIN="$(git rev-parse "$REMOTE/$MAIN_BRANCH")"
BASE_MAIN="$(git merge-base "$MAIN_BRANCH" "$REMOTE/$MAIN_BRANCH")"

if [[ "$LOCAL_MAIN" != "$REMOTE_MAIN" && "$LOCAL_MAIN" == "$BASE_MAIN" ]]; then
  echo "❌ '$MAIN_BRANCH' is behind '$REMOTE/$MAIN_BRANCH'."
  echo "   Please pull/rebase first, then rerun."
  exit 1
fi

echo "✅ '$MAIN_BRANCH' is up-to-date (or ahead/diverged). Pulling latest..."
git pull --ff-only "$REMOTE" "$MAIN_BRANCH"

echo "🏗️  Running build: deno task build"
deno task build

# Ensure pages branch exists
git show-ref --verify --quiet "refs/heads/$PAGES_BRANCH" || {
  echo "❌ Local branch '$PAGES_BRANCH' not found."
  echo "   Create it first: git switch -c $PAGES_BRANCH"
  exit 1
}

echo "🌿 Switching to '$PAGES_BRANCH'..."
git switch -q "$PAGES_BRANCH"

echo "🔀 Fast-forward merging '$MAIN_BRANCH' into '$PAGES_BRANCH'..."
git merge --ff-only "$MAIN_BRANCH"

# Copy dist -> docs (replace docs contents)
if [[ ! -d "dist" ]]; then
  echo "❌ dist/ not found. Did 'deno task build' output to dist/?"
  exit 1
fi

echo "📦 Copying dist/ -> docs/ (sync)..."
rm -rf docs
mkdir -p docs
# Copy contents of dist into docs
cp -R dist/. ../docs/

# Commit if changed
if git diff --quiet; then
  echo "✅ No changes to commit 
::contentReference[oaicite:0]{index=0}
