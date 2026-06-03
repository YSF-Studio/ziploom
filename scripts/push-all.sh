#!/bin/bash
# Push with per-repo README swap
# Usage: ./scripts/push-all.sh "commit message"

set -e

MSG="${1:-update}"
BASE="/home/kali/ysf-forensic-suite"
cd "$BASE"

# Save the real root README.md that should be in the monorepo
cp README.md README-monorepo.md

# Function to push to a specific remote with its own README
push_repo() {
    local remote="$1"
    local repo_name="$2"
    local readme_file="README-${repo_name}.md"
    
    echo "=== Pushing to $remote ($repo_name) ==="
    
    # Swap root README with repo-specific one
    cp "$readme_file" README.md
    git add README.md
    git commit --amend --no-edit --allow-empty 2>/dev/null || true
    
    # Force push to remote
    git push "$remote" main -f
}

# Push to all three repos with their own README
push_repo "origin" "ziploom"
push_repo "analysisloom" "analysisloom"
push_repo "collectionloom" "collectionloom"

# Restore monorepo README
cp README-monorepo.md README.md
git add README.md
git commit --amend --no-edit --allow-empty 2>/dev/null || true
rm README-monorepo.md

echo "=== Done! All repos pushed with their own README ==="
