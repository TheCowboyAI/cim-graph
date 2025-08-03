#!/usr/bin/env bash
# Generate progress dashboard from EventStore (progress.json)

set -euo pipefail

PROGRESS_JSON="doc/progress/progress.json"
PROGRESS_MD="doc/progress/progress.md"

if [ ! -f "$PROGRESS_JSON" ]; then
    echo "Error: $PROGRESS_JSON not found"
    exit 1
fi

# Extract data from progress.json
PROJECT=$(jq -r '.project' "$PROGRESS_JSON")
DESCRIPTION=$(jq -r '.description' "$PROGRESS_JSON")
CURRENT_PHASE=$(jq -r '.current_phase' "$PROGRESS_JSON")
TOTAL_EVENTS=$(jq -r '.projections.statistics.total_events' "$PROGRESS_JSON")
TOTAL_COMMITS=$(jq -r '.projections.statistics.commits' "$PROGRESS_JSON")
LAST_UPDATE=$(jq -r '.events[-1].timestamp' "$PROGRESS_JSON")

# Generate dashboard
cat > "$PROGRESS_MD" << 'EOF'
# CIM Graph - Progress Dashboard

> Auto-generated from EventStore (`progress.json`) - DO NOT EDIT MANUALLY

EOF

# Project Overview
cat >> "$PROGRESS_MD" << EOF
## 📊 Project Overview

**Project**: $PROJECT
**Description**: $DESCRIPTION
**Current Phase**: **$CURRENT_PHASE**
**Last Updated**: $LAST_UPDATE

## 📈 Metrics

| Metric | Value |
|--------|-------|
| Total Events | $TOTAL_EVENTS |
| Git Commits | $TOTAL_COMMITS |
| Current Phase | $CURRENT_PHASE |

EOF

# Phase Progress
echo "## 🎯 Phase Progress" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"

# Get phase completion from projections
jq -r '.projections.current_state.phase_completion | to_entries[] | "- **\(.key)**: \(.value)% complete"' "$PROGRESS_JSON" >> "$PROGRESS_MD"

echo "" >> "$PROGRESS_MD"

# Current Phase Requirements
echo "## 📋 Current Phase Requirements ($CURRENT_PHASE)" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"

if jq -e ".projections.phase_requirements.$CURRENT_PHASE" "$PROGRESS_JSON" > /dev/null 2>&1; then
    echo "### ✅ Completed" >> "$PROGRESS_MD"
    jq -r ".projections.phase_requirements.$CURRENT_PHASE.completed[]? | \"- [x] \" + ." "$PROGRESS_JSON" >> "$PROGRESS_MD" 2>/dev/null || echo "- None yet" >> "$PROGRESS_MD"
    
    echo "" >> "$PROGRESS_MD"
    echo "### ⏳ Remaining" >> "$PROGRESS_MD"
    jq -r ".projections.phase_requirements.$CURRENT_PHASE.remaining[]? | \"- [ ] \" + ." "$PROGRESS_JSON" >> "$PROGRESS_MD" 2>/dev/null || echo "- None" >> "$PROGRESS_MD"
fi

echo "" >> "$PROGRESS_MD"

# Recent Events
echo "## 🔄 Recent Events" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"
echo "| Timestamp | Event | Phase | Commit |" >> "$PROGRESS_MD"
echo "|-----------|-------|-------|--------|" >> "$PROGRESS_MD"

jq -r '.events[-5:] | reverse | .[] | "| \(.timestamp | split("T")[0]) | \(.event_name) | \(.data.phase) | \(.git_commit[0:7]) |"' "$PROGRESS_JSON" >> "$PROGRESS_MD"

echo "" >> "$PROGRESS_MD"

# Artifacts Created
echo "## 📁 Artifacts Created" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"

jq -r '.projections.current_state.completed_artifacts[] | "- `" + . + "`"' "$PROGRESS_JSON" >> "$PROGRESS_MD"

echo "" >> "$PROGRESS_MD"

# Event Timeline
echo "## 📅 Event Timeline" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"
echo '```mermaid' >> "$PROGRESS_MD"
echo 'gantt' >> "$PROGRESS_MD"
echo '    title Project Event Timeline' >> "$PROGRESS_MD"
echo '    dateFormat YYYY-MM-DD' >> "$PROGRESS_MD"
echo '    section Events' >> "$PROGRESS_MD"

# Generate gantt entries from events
jq -r '.events[] | "    \(.event_name) : \(.timestamp | split("T")[0])"' "$PROGRESS_JSON" >> "$PROGRESS_MD"

echo '```' >> "$PROGRESS_MD"

echo "" >> "$PROGRESS_MD"

# Causal Chain
echo "## 🔗 Event Causation Chain" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"
echo '```mermaid' >> "$PROGRESS_MD"
echo 'graph TD' >> "$PROGRESS_MD"

# Generate causation chain
jq -r '.events[] | 
  if .causation_id == null then 
    "    \(.event_id[0:8])[\(.event_name)]"
  else
    "    \(.causation_id[0:8]) --> \(.event_id[0:8])[\(.event_name)]"
  end' "$PROGRESS_JSON" >> "$PROGRESS_MD"

echo '```' >> "$PROGRESS_MD"

echo "" >> "$PROGRESS_MD"

# Statistics by Phase
echo "## 📊 Statistics by Phase" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"
echo "| Phase | Event Count |" >> "$PROGRESS_MD"
echo "|-------|-------------|" >> "$PROGRESS_MD"

jq -r '.projections.statistics.events_by_phase | to_entries[] | "| \(.key) | \(.value) |"' "$PROGRESS_JSON" >> "$PROGRESS_MD"

echo "" >> "$PROGRESS_MD"

# Footer
echo "---" >> "$PROGRESS_MD"
echo "" >> "$PROGRESS_MD"
echo "_Generated on: $(date -I)_" >> "$PROGRESS_MD"

echo "✅ Dashboard generated: $PROGRESS_MD"