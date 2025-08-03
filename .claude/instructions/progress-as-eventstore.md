# Progress.json as EventStore - MANDATORY

## Core Principle

**progress.json IS your EventStore** - It is the single source of truth and system of record for ALL work done. Every action that changes the project state MUST be recorded as an event following proper event sourcing patterns.

## Event Schema

Every event MUST follow this base format:

```json
{
  "event_id": "550e8400-e29b-41d4-a716-446655440001",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440002",
  "causation_id": "550e8400-e29b-41d4-a716-446655440000",
  "event_name": "DesignDocumentCreated",
  "timestamp": "2025-08-03T10:30:00Z",
  "git_commit": "abc123def456",
  "data": {
    "phase": "DESIGN",
    "artifacts": ["doc/design/cim-graph-design.md"],
    "description": "Created unified graph abstraction design document"
  }
}
```

## Event Naming Convention

Events MUST be named in **past tense** describing what happened:
- `ProjectInitialized`
- `DesignPhaseStarted`
- `ArchitectureDiagramCreated`
- `TraitSystemDesigned`
- `ImplementationPlanned`
- `CodeImplemented`
- `TestsWritten`
- `BuildVerified`
- `PhaseCompleted`

## Event Relationships

### correlation_id
- Groups related events together
- All events for a single feature/task share the same correlation_id
- Example: All events for "Design CimGraph trait system" task

### causation_id
- Links to the event that caused this event
- Creates a causal chain of events
- First event in a chain has null causation_id
- Example: `DesignPhaseStarted` causes `ArchitectureDiagramCreated`

## Workflow - MANDATORY SEQUENCE

### 1. Start New Task Group
```bash
# Generate new correlation_id for the task group
CORRELATION_ID=$(uuidgen)

# First event has no causation_id
cat doc/progress/progress.json | jq '.events'
```

### 2. Record Start Event
```json
{
  "event_id": "$(uuidgen)",
  "correlation_id": "$CORRELATION_ID",
  "causation_id": null,
  "event_name": "DesignTaskStarted",
  "timestamp": "$(date -Iseconds)",
  "git_commit": null,
  "data": {
    "task": "Design CimGraph trait system",
    "phase": "DESIGN"
  }
}
```

### 3. Do The Work
- Create files
- Write documentation
- Implement code

### 4. Commit The Work
```bash
git add -A
git commit -m "feat: Design CimGraph trait system"
GIT_COMMIT=$(git rev-parse HEAD)
```

### 5. Record Completion Event
```json
{
  "event_id": "$(uuidgen)",
  "correlation_id": "$CORRELATION_ID",
  "causation_id": "$PREVIOUS_EVENT_ID",
  "event_name": "TraitSystemDesigned",
  "timestamp": "$(date -Iseconds)",
  "git_commit": "$GIT_COMMIT",
  "data": {
    "artifacts": ["doc/design/trait-system.md"],
    "description": "Completed CimGraph trait system design"
  }
}
```

## Progress.json Structure

```json
{
  "project": "cim-graph",
  "current_phase": "DESIGN",
  "last_event_id": "550e8400-e29b-41d4-a716-446655440010",
  "events": [
    {
      "event_id": "550e8400-e29b-41d4-a716-446655440001",
      "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
      "causation_id": null,
      "event_name": "ProjectInitialized",
      "timestamp": "2025-08-03T09:00:00Z",
      "git_commit": "abc123",
      "data": {
        "phase": "INITIALIZE"
      }
    },
    {
      "event_id": "550e8400-e29b-41d4-a716-446655440002",
      "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
      "causation_id": "550e8400-e29b-41d4-a716-446655440001",
      "event_name": "DesignPhaseStarted",
      "timestamp": "2025-08-03T10:00:00Z",
      "git_commit": null,
      "data": {
        "phase": "DESIGN"
      }
    }
  ],
  "projections": {
    "current_state": {
      "phase": "DESIGN",
      "completed_artifacts": [
        "doc/design/cim-graph-design.md"
      ],
      "pending_artifacts": [
        "doc/design/architecture.md",
        "doc/design/event-flows.md"
      ]
    },
    "statistics": {
      "total_events": 10,
      "events_by_phase": {
        "INITIALIZE": 3,
        "DESIGN": 7
      },
      "commits": 5
    }
  }
}
```

## Event Patterns

### Starting a Phase
```json
{
  "event_name": "DesignPhaseStarted",
  "causation_id": "<previous-phase-completed-event-id>"
}
```

### Creating Artifacts
```json
{
  "event_name": "ArchitectureDiagramCreated",
  "causation_id": "<design-phase-started-event-id>",
  "data": {
    "artifact": "doc/design/architecture.md"
  }
}
```

### Completing Work
```json
{
  "event_name": "TraitSystemImplemented",
  "git_commit": "def456",
  "data": {
    "files": ["src/traits.rs", "src/lib.rs"]
  }
}
```

### Phase Transitions
```json
{
  "event_name": "DesignPhaseCompleted",
  "causation_id": "<last-design-artifact-event-id>"
}
```

## Rules

1. **Events are immutable** - Never modify existing events
2. **Events are append-only** - Only add new events
3. **Every git commit needs an event** - No work without events
4. **Use proper UUIDs** - Not sequential IDs
5. **Past tense event names** - Describe what happened
6. **Maintain causation chain** - Link related events

## Common Event Types

### Phase Events
- `InitializePhaseStarted`
- `InitializePhaseCompleted`
- `DesignPhaseStarted`
- `DesignPhaseCompleted`
- `PlanPhaseStarted`
- `PlanPhaseCompleted`

### Work Events
- `FileCreated`
- `DocumentationWritten`
- `CodeImplemented`
- `TestsAdded`
- `BugFixed`
- `RefactoringCompleted`

### Review Events
- `DesignReviewed`
- `CodeReviewed`
- `TestsPassed`
- `BuildSucceeded`

## Querying Events

### Find events by correlation
```bash
cat doc/progress/progress.json | jq '.events[] | select(.correlation_id == "550e8400-e29b-41d4-a716-446655440000")'
```

### Find events by phase
```bash
cat doc/progress/progress.json | jq '.events[] | select(.data.phase == "DESIGN")'
```

### Get causal chain
```bash
# Starting from an event, trace back through causation_ids
```

## Enforcement

Before ANY work:
1. Generate correlation_id for task group
2. Record start event
3. Do the work
4. Commit to git
5. Record completion event with git_commit

This is NOT optional. Every action MUST be recorded as an event.