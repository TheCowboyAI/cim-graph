# Progress Dashboard Generation

## Overview

Since `progress.json` follows EventStore patterns and is less human-readable, we generate a `progress.md` dashboard that provides a clear view of project status.

## Dashboard Features

The generated dashboard includes:
- Project overview and current phase
- Phase progress percentages
- Current phase requirements checklist
- Recent events table
- Artifacts created
- Event timeline (Gantt chart)
- Event causation chain (Mermaid diagram)
- Statistics by phase

## Generation Process

### Manual Generation
```bash
.claude/scripts/generate-progress-dashboard.sh
```

### Automatic Generation
The dashboard should be regenerated whenever progress.json is updated:

1. After adding events to progress.json
2. Before committing changes
3. As part of the progress update workflow

## Dashboard Location
- **Source**: `doc/progress/progress.json` (EventStore)
- **Output**: `doc/progress/progress.md` (Human-readable dashboard)

## Important Notes
- The dashboard is auto-generated - DO NOT edit progress.md manually
- All changes should be made to progress.json as events
- The dashboard provides projections and visualizations of the event stream
- Commit both progress.json and progress.md together

## Integration with Workflow

When updating progress:
1. Add event to progress.json
2. Run dashboard generator
3. Commit both files together

Example:
```bash
# After updating progress.json
.claude/scripts/generate-progress-dashboard.sh

# Commit both files
git add doc/progress/progress.json doc/progress/progress.md
git commit -m "feat: Complete architecture diagrams"
```

## Dashboard Sections

### Project Overview
- Current phase and description
- Last update timestamp
- Key metrics

### Phase Progress
- Percentage completion for each phase
- Visual progress indicators

### Current Phase Requirements
- Checklist of completed items
- Remaining tasks for current phase

### Recent Events
- Last 5 events in reverse chronological order
- Shows event name, phase, and commit hash

### Event Timeline
- Gantt chart showing when events occurred
- Visual representation of project timeline

### Event Causation Chain
- Mermaid diagram showing causal relationships
- Traces how events led to other events

### Statistics
- Event count by phase
- Total commits and artifacts

## Customization

To modify the dashboard format, edit:
`.claude/scripts/generate-progress-dashboard.sh`

The script uses `jq` to extract data from the EventStore and formats it into Markdown with Mermaid diagrams.