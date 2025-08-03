#!/bin/bash

# Add planning phase transition event to progress.json

cat > /tmp/planning_events.json << 'EOF'
    {
      "event_id": "b8f4d2ac-7e91-4c8a-b3f5-9c8e7a4d5f12",
      "correlation_id": "cd7bb105-d931-4e89-bb48-eb07194cb541",
      "causation_id": "3f4cc9df-5bb6-4c3e-ba9f-e604615ae36a",
      "event_name": "RequirementsDocumentationCompleted",
      "timestamp": "2025-08-03T12:30:00-07:00",
      "git_commit": "e0a9599f0c8e5c4d7b9a2e6f1a3b8c9d5e7f2a4b",
      "data": {
        "phase": "DESIGN",
        "artifacts": [
          "doc/design/user-stories.md",
          "doc/design/acceptance-tests.md",
          "doc/design/test-strategy.md",
          "doc/design/domain-model.md",
          "doc/design/event-flows.md"
        ],
        "description": "Completed comprehensive requirements documentation with graph-based specifications"
      }
    },
    {
      "event_id": "c9e5f3bd-8c02-4d89-a4f6-ad9f8b5c6e13",
      "correlation_id": "cd7bb105-d931-4e89-bb48-eb07194cb541",
      "causation_id": "b8f4d2ac-7e91-4c8a-b3f5-9c8e7a4d5f12",
      "event_name": "PlanningDocumentationCompleted",
      "timestamp": "2025-08-03T13:00:00-07:00",
      "git_commit": "884b60d8f7e4c5b9a3d6e8f2b1c7a9d5e6f3b8c",
      "data": {
        "phase": "DESIGN",
        "artifacts": [
          "doc/design/implementation-plan.md",
          "doc/design/api-contracts.md",
          "doc/design/development-workflow.md"
        ],
        "description": "Completed implementation planning and API contract definitions"
      }
    },
    {
      "event_id": "d1a6f4ce-9d13-4e9a-b5g7-be0g9c6d7f14",
      "correlation_id": "cd7bb105-d931-4e89-bb48-eb07194cb541",
      "causation_id": "c9e5f3bd-8c02-4d89-a4f6-ad9f8b5c6e13",
      "event_name": "AggregateDesignCompleted",
      "timestamp": "2025-08-03T13:30:00-07:00",
      "git_commit": "d1a5141f9e8d7c6b5a4f3e2d1c9b8a7f6e5d4c3b",
      "data": {
        "phase": "DESIGN",
        "artifacts": [
          "doc/design/aggregate-state-transitions.md",
          "doc/design/aggregate-transaction-tests.md"
        ],
        "description": "Completed aggregate state machines and transaction acceptance tests"
      }
    },
    {
      "event_id": "a7e9c3df-8b12-4f76-9cd2-ef45a823b91e",
      "correlation_id": "e8f9d4bc-7a23-4d8e-9b5c-af67b9c8e5d2",
      "causation_id": "d1a6f4ce-9d13-4e9a-b5g7-be0g9c6d7f14",
      "event_name": "PhaseTransitionInitiated",
      "timestamp": "2025-08-03T14:00:00-07:00",
      "git_commit": null,
      "data": {
        "phase": "PLANNING",
        "from_phase": "DESIGN",
        "to_phase": "PLANNING",
        "artifacts": [
          "doc/planning/planning-kickoff.md",
          "doc/planning/sprint-plan.md"
        ],
        "description": "Transitioned from DESIGN to PLANNING phase with complete documentation"
      }
    }
EOF

# Update progress.json with new events
jq --slurpfile new_events /tmp/planning_events.json '
  .current_phase = "PLANNING" |
  .last_event_id = "a7e9c3df-8b12-4f76-9cd2-ef45a823b91e" |
  .events += $new_events |
  .projections.current_state.phase = "PLANNING" |
  .projections.statistics.total_events = 17 |
  .projections.statistics.events_by_phase.DESIGN = 13 |
  .projections.statistics.events_by_phase.PLANNING = 1 |
  .projections.statistics.commits = 11 |
  .projections.statistics.artifacts_created = 31 |
  .projections.phase_requirements.PLANNING = {
    "required": [
      "sprint_plan",
      "resource_allocation",
      "task_breakdown",
      "timeline"
    ],
    "completed": [
      "planning_kickoff",
      "sprint_plan"
    ],
    "remaining": [
      "resource_allocation",
      "task_breakdown"
    ]
  }
' doc/progress/progress.json > /tmp/progress_updated.json

mv /tmp/progress_updated.json doc/progress/progress.json

# Clean up
rm -f /tmp/planning_events.json

echo "Successfully added planning phase events to progress.json"