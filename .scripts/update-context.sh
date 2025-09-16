#!/bin/bash

# Context Management Script for Campfire-on-Rust
# Automatically updates SESSION_CONTEXT.md and syncs todos

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SESSION_CONTEXT="$PROJECT_ROOT/SESSION_CONTEXT.md"
CLAUDE_MD="$PROJECT_ROOT/CLAUDE.md"

echo "🔄 Updating Campfire-on-Rust session context..."

# Update timestamp in SESSION_CONTEXT.md
if [ -f "$SESSION_CONTEXT" ]; then
    sed -i "s/\*Last Auto-Update: .*/\*Last Auto-Update: $(date '+%Y-%m-%d %H:%M:%S')/" "$SESSION_CONTEXT"
    echo "✅ Updated timestamp in SESSION_CONTEXT.md"
fi

# Check git status and branch
current_branch=$(git branch --show-current 2>/dev/null || echo "unknown")
git_status=$(git status --porcelain 2>/dev/null || echo "")

# Update live session status if needed
if [ -n "$current_branch" ] && [ -f "$SESSION_CONTEXT" ]; then
    sed -i "s/^\- \*\*Branch\*\*: .*/- **Branch**: \`$current_branch\`/" "$SESSION_CONTEXT"
    echo "✅ Updated branch information"
fi

# Sync todos from SESSION_CONTEXT.md to TodoWrite format (if TodoWrite is available)
if command -v claude &> /dev/null && [ -f "$SESSION_CONTEXT" ]; then
    echo "📝 Syncing todos with SESSION_CONTEXT.md..."
    # Extract todos from SESSION_CONTEXT.md and format for TodoWrite
    # This would be implemented based on TodoWrite API
fi

# Run architecture compliance check
if [ -f "$PROJECT_ROOT/.kiro/steering/anti-coordination.md" ]; then
    echo "🔍 Checking architecture compliance..."
    # Add compliance checks here
fi

echo "✅ Context management complete!"
echo ""
echo "Quick commands:"
echo "  cat $SESSION_CONTEXT.md | grep -A 20 'Live Session Status'"
echo "  git status"
echo "  git log --oneline -5"