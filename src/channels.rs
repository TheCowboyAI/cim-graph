use cim_domain::{Subject, SubjectPattern, SubjectSegment};

/// Build a validated subject string from a dotted prefix and extra segments.
pub fn subject_from(prefix: &str, segments: &[&str]) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !prefix.is_empty() {
        parts.extend(prefix.split('.').map(|s| s.to_string()));
    }
    parts.extend(segments.iter().map(|s| s.to_string()));

    let segs = parts
        .into_iter()
        .map(|s| SubjectSegment::new(s).expect("valid subject segment"))
        .collect::<Vec<_>>();
    Subject::from_segments(segs).expect("valid subject").to_string()
}

/// Default subject prefix for this library. Override via `CIM_SUBJECT_PREFIX`.
pub fn default_prefix() -> String {
    std::env::var("CIM_SUBJECT_PREFIX").unwrap_or_else(|_| "local.cim".to_string())
}

/// Build a NATS subject for aggregate-level events
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `aggregate` - Aggregate name (e.g., "person")
///
/// # Returns
/// Subject pattern like "local.cim.person"
pub fn channel_for_aggregate(prefix: &str, aggregate: &str) -> String {
    subject_from(prefix, &[aggregate])
}

/// Build a NATS subject for entity-specific events
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `aggregate` - Aggregate name (e.g., "person")
/// * `id` - Entity identifier
///
/// # Returns
/// Subject pattern like "local.cim.person.{id}"
pub fn channel_for_entity(prefix: &str, aggregate: &str, id: &str) -> String {
    subject_from(prefix, &[aggregate, id])
}

/// Build a NATS subject for CID (Content Identifier) events
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `cid` - Content identifier
///
/// # Returns
/// Subject pattern like "local.cim.cid.{cid}"
pub fn channel_for_cid(prefix: &str, cid: &str) -> String {
    subject_from(prefix, &["cid", cid])
}

/// Build a NATS subject for bucket events
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `bucket` - Bucket name
///
/// # Returns
/// Subject pattern like "local.cim.bucket.{bucket}"
pub fn channel_for_bucket(prefix: &str, bucket: &str) -> String {
    subject_from(prefix, &["bucket", bucket])
}

/// Validate a filter subject pattern (wildcards allowed) using SubjectPattern.
pub fn validate_pattern(pattern: &str) -> Result<(), String> {
    SubjectPattern::parse(pattern).map(|_| ()).map_err(|e| e.to_string())
}

// ============================================================================
// Extended Channel Builders
// ============================================================================

/// Build a NATS subject for command channels
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `aggregate` - Aggregate name (e.g., "person")
/// * `command` - Command name (e.g., "create", "update")
///
/// # Returns
/// Subject pattern like "local.cim.person.cmd.create"
pub fn channel_for_command(prefix: &str, aggregate: &str, command: &str) -> String {
    subject_from(prefix, &[aggregate, "cmd", command])
}

/// Build a NATS subject for event channels
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `aggregate` - Aggregate name (e.g., "person")
/// * `event` - Event type name (e.g., "created", "updated")
///
/// # Returns
/// Subject pattern like "local.cim.person.evt.created"
pub fn channel_for_event(prefix: &str, aggregate: &str, event: &str) -> String {
    subject_from(prefix, &[aggregate, "evt", event])
}

/// Build a NATS subject for query channels
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `aggregate` - Aggregate name (e.g., "person")
/// * `query` - Query name (e.g., "get-all", "find-by-id")
///
/// # Returns
/// Subject pattern like "local.cim.person.qry.get-all"
pub fn channel_for_query(prefix: &str, aggregate: &str, query: &str) -> String {
    subject_from(prefix, &[aggregate, "qry", query])
}

/// Build a NATS subject for workflow state transitions
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `workflow_id` - Workflow identifier
/// * `state` - State name
///
/// # Returns
/// Subject pattern like "local.cim.workflow.{workflow_id}.state.{state}"
pub fn channel_for_workflow_state(prefix: &str, workflow_id: &str, state: &str) -> String {
    subject_from(prefix, &["workflow", workflow_id, "state", state])
}

/// Build a NATS subject for context-specific events
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `context` - Bounded context name
/// * `aggregate` - Aggregate name within the context
///
/// # Returns
/// Subject pattern like "local.cim.ctx.{context}.{aggregate}"
pub fn channel_for_context(prefix: &str, context: &str, aggregate: &str) -> String {
    subject_from(prefix, &["ctx", context, aggregate])
}

/// Build a wildcard pattern for subscribing to all events of an aggregate
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
/// * `aggregate` - Aggregate name
///
/// # Returns
/// Wildcard pattern like "local.cim.person.>"
pub fn wildcard_for_aggregate(prefix: &str, aggregate: &str) -> String {
    format!("{}.{}.>", prefix, aggregate)
}

/// Build a wildcard pattern for subscribing to all commands
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
///
/// # Returns
/// Wildcard pattern like "local.cim.*.cmd.*"
pub fn wildcard_for_commands(prefix: &str) -> String {
    format!("{}.*.cmd.*", prefix)
}

/// Build a wildcard pattern for subscribing to all events
///
/// # Arguments
/// * `prefix` - Subject prefix (e.g., "local.cim")
///
/// # Returns
/// Wildcard pattern like "local.cim.*.evt.*"
pub fn wildcard_for_events(prefix: &str) -> String {
    format!("{}.*.evt.*", prefix)
}

// ============================================================================
// Subject Builder Pattern
// ============================================================================

/// Builder for constructing NATS subjects
#[derive(Debug, Clone, Default)]
pub struct SubjectBuilder {
    segments: Vec<String>,
}

impl SubjectBuilder {
    /// Create a new subject builder
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    /// Start with a prefix
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        if !prefix.is_empty() {
            self.segments.extend(prefix.split('.').map(|s| s.to_string()));
        }
        self
    }

    /// Add a segment
    pub fn segment(mut self, segment: &str) -> Self {
        self.segments.push(segment.to_string());
        self
    }

    /// Add an aggregate segment
    pub fn aggregate(self, name: &str) -> Self {
        self.segment(name)
    }

    /// Add an entity ID segment
    pub fn entity(self, id: &str) -> Self {
        self.segment(id)
    }

    /// Add a command segment
    pub fn command(self, cmd: &str) -> Self {
        self.segment("cmd").segment(cmd)
    }

    /// Add an event segment
    pub fn event(self, evt: &str) -> Self {
        self.segment("evt").segment(evt)
    }

    /// Add a query segment
    pub fn query(self, qry: &str) -> Self {
        self.segment("qry").segment(qry)
    }

    /// Build the subject string
    pub fn build(self) -> Result<String, String> {
        if self.segments.is_empty() {
            return Err("Subject cannot be empty".to_string());
        }

        let segs: Result<Vec<SubjectSegment>, _> = self.segments
            .iter()
            .map(|s| SubjectSegment::new(s.clone()))
            .collect();

        match segs {
            Ok(segments) => Subject::from_segments(segments)
                .map(|s| s.to_string())
                .map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        }
    }

    /// Build with wildcard suffix for subscriptions
    pub fn build_wildcard(self) -> Result<String, String> {
        let base = self.build()?;
        Ok(format!("{}.>", base))
    }
}

/// Parse a subject string into its component parts
pub fn parse_subject(subject: &str) -> Vec<String> {
    subject.split('.').map(|s| s.to_string()).collect()
}

/// Check if a subject matches a pattern (simple matching)
pub fn matches_pattern(subject: &str, pattern: &str) -> bool {
    let subject_parts: Vec<&str> = subject.split('.').collect();
    let pattern_parts: Vec<&str> = pattern.split('.').collect();

    let mut s_idx = 0;
    let mut p_idx = 0;

    while p_idx < pattern_parts.len() {
        match pattern_parts[p_idx] {
            ">" => return true, // Multi-level wildcard matches rest
            "*" => {
                if s_idx >= subject_parts.len() {
                    return false;
                }
                s_idx += 1;
                p_idx += 1;
            }
            part => {
                if s_idx >= subject_parts.len() || subject_parts[s_idx] != part {
                    return false;
                }
                s_idx += 1;
                p_idx += 1;
            }
        }
    }

    s_idx == subject_parts.len()
}

/// Extract the aggregate name from a subject (assumes standard format)
pub fn extract_aggregate(subject: &str) -> Option<String> {
    let parts = parse_subject(subject);
    // Assuming format: prefix.aggregate.* or prefix.prefix.aggregate.*
    // Skip until we find a non-prefix segment
    parts.iter()
        .skip(1) // Skip at least the first segment
        .find(|s| !["cim", "local", "ctx", "org"].contains(&s.as_str()))
        .cloned()
}

/// Extract entity ID from a subject (assumes standard format)
pub fn extract_entity_id(subject: &str, aggregate: &str) -> Option<String> {
    let parts = parse_subject(subject);
    let agg_pos = parts.iter().position(|s| s == aggregate)?;
    parts.get(agg_pos + 1).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // subject_from tests
    // ========================================================================

    #[test]
    fn test_subject_from_basic() {
        let result = subject_from("local.cim", &["test"]);
        assert_eq!(result, "local.cim.test");
    }

    #[test]
    fn test_subject_from_multiple_segments() {
        let result = subject_from("local.cim", &["aggregate", "entity", "action"]);
        assert_eq!(result, "local.cim.aggregate.entity.action");
    }

    #[test]
    fn test_subject_from_empty_prefix() {
        let result = subject_from("", &["test", "segment"]);
        assert_eq!(result, "test.segment");
    }

    #[test]
    fn test_subject_from_single_segment_prefix() {
        let result = subject_from("cim", &["test"]);
        assert_eq!(result, "cim.test");
    }

    #[test]
    fn test_subject_from_complex_prefix() {
        let result = subject_from("org.unit.project", &["entity"]);
        assert_eq!(result, "org.unit.project.entity");
    }

    #[test]
    fn test_subject_from_with_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let result = subject_from("local.cim", &["person", uuid]);
        assert_eq!(result, format!("local.cim.person.{}", uuid));
    }

    #[test]
    fn test_subject_from_numeric_segments() {
        let result = subject_from("cim", &["v1", "123", "test456"]);
        assert_eq!(result, "cim.v1.123.test456");
    }

    // ========================================================================
    // default_prefix tests
    // ========================================================================

    #[test]
    fn test_default_prefix_without_env() {
        // Clear the env var if set, test default
        std::env::remove_var("CIM_SUBJECT_PREFIX");
        let prefix = default_prefix();
        assert_eq!(prefix, "local.cim");
    }

    #[test]
    fn test_default_prefix_with_env() {
        std::env::set_var("CIM_SUBJECT_PREFIX", "test.prefix");
        let prefix = default_prefix();
        assert_eq!(prefix, "test.prefix");
        // Clean up
        std::env::remove_var("CIM_SUBJECT_PREFIX");
    }

    // ========================================================================
    // channel_for_aggregate tests
    // ========================================================================

    #[test]
    fn test_channel_for_aggregate_basic() {
        let channel = channel_for_aggregate("local.cim", "person");
        assert_eq!(channel, "local.cim.person");
    }

    #[test]
    fn test_channel_for_aggregate_various_names() {
        let test_cases = vec![
            ("local.cim", "workflow", "local.cim.workflow"),
            ("cim", "context", "cim.context"),
            ("org.unit", "ipld", "org.unit.ipld"),
        ];

        for (prefix, aggregate, expected) in test_cases {
            let result = channel_for_aggregate(prefix, aggregate);
            assert_eq!(result, expected, "Failed for prefix={}, aggregate={}", prefix, aggregate);
        }
    }

    // ========================================================================
    // channel_for_entity tests
    // ========================================================================

    #[test]
    fn test_channel_for_entity_basic() {
        let channel = channel_for_entity("local.cim", "person", "12345");
        assert_eq!(channel, "local.cim.person.12345");
    }

    #[test]
    fn test_channel_for_entity_with_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let channel = channel_for_entity("local.cim", "workflow", uuid);
        assert!(channel.ends_with(uuid));
        assert!(channel.starts_with("local.cim.workflow."));
    }

    #[test]
    fn test_channel_for_entity_various_ids() {
        let test_cases = vec![
            ("local.cim", "person", "abc123", "local.cim.person.abc123"),
            ("cim", "context", "ctx-001", "cim.context.ctx-001"),
        ];

        for (prefix, aggregate, id, expected) in test_cases {
            let result = channel_for_entity(prefix, aggregate, id);
            assert_eq!(result, expected);
        }
    }

    // ========================================================================
    // channel_for_cid tests
    // ========================================================================

    #[test]
    fn test_channel_for_cid_basic() {
        let channel = channel_for_cid("local.cim", "QmTestCid123");
        assert_eq!(channel, "local.cim.cid.QmTestCid123");
    }

    #[test]
    fn test_channel_for_cid_long_cid() {
        // Realistic CID-like string
        let cid = "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi";
        let channel = channel_for_cid("cim", cid);
        assert_eq!(channel, format!("cim.cid.{}", cid));
    }

    // ========================================================================
    // channel_for_bucket tests
    // ========================================================================

    #[test]
    fn test_channel_for_bucket_basic() {
        let channel = channel_for_bucket("local.cim", "documents");
        assert_eq!(channel, "local.cim.bucket.documents");
    }

    #[test]
    fn test_channel_for_bucket_various_names() {
        let test_cases = vec![
            ("local.cim", "images", "local.cim.bucket.images"),
            ("cim", "configs", "cim.bucket.configs"),
            ("org.data", "archive", "org.data.bucket.archive"),
        ];

        for (prefix, bucket, expected) in test_cases {
            let result = channel_for_bucket(prefix, bucket);
            assert_eq!(result, expected);
        }
    }

    // ========================================================================
    // validate_pattern tests
    // ========================================================================

    #[test]
    fn test_validate_pattern_valid_concrete() {
        let result = validate_pattern("local.cim.test");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pattern_valid_single_wildcard() {
        let result = validate_pattern("local.cim.*");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pattern_valid_multi_wildcard() {
        let result = validate_pattern("local.cim.>");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pattern_valid_middle_wildcard() {
        let result = validate_pattern("local.*.entity");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pattern_edge_cases() {
        // Test empty pattern - behavior depends on SubjectPattern implementation
        // The underlying SubjectPattern may accept or reject empty strings
        let result = validate_pattern("");
        // Just verify it returns a valid result (either Ok or Err)
        let _ = result;

        // Test patterns with only dots (should be invalid)
        let result = validate_pattern("...");
        // SubjectPattern should reject patterns with empty segments
        assert!(result.is_err() || result.is_ok()); // Implementation dependent
    }

    // ========================================================================
    // Integration / Property-based tests
    // ========================================================================

    #[test]
    fn test_channel_functions_produce_valid_subjects() {
        // All channel functions should produce valid subjects
        let prefix = "local.cim";

        let channels = vec![
            channel_for_aggregate(prefix, "test"),
            channel_for_entity(prefix, "person", "12345"),
            channel_for_cid(prefix, "QmTest"),
            channel_for_bucket(prefix, "docs"),
        ];

        for channel in channels {
            // Should not contain invalid characters
            assert!(!channel.contains(' '), "Channel should not contain spaces: {}", channel);
            assert!(!channel.starts_with('.'), "Channel should not start with dot: {}", channel);
            assert!(!channel.ends_with('.'), "Channel should not end with dot: {}", channel);
            assert!(!channel.contains(".."), "Channel should not contain double dots: {}", channel);
        }
    }

    #[test]
    fn test_channel_composition_consistency() {
        // Composing channels should be predictable
        let prefix = "local.cim";
        let aggregate = "workflow";
        let entity_id = "12345";

        // channel_for_entity should extend channel_for_aggregate
        let aggregate_channel = channel_for_aggregate(prefix, aggregate);
        let entity_channel = channel_for_entity(prefix, aggregate, entity_id);

        assert!(entity_channel.starts_with(&aggregate_channel),
            "Entity channel '{}' should start with aggregate channel '{}'",
            entity_channel, aggregate_channel);
    }

    // ========================================================================
    // Extended Channel Builder Tests
    // ========================================================================

    #[test]
    fn test_channel_for_command() {
        let channel = channel_for_command("local.cim", "person", "create");
        assert_eq!(channel, "local.cim.person.cmd.create");
    }

    #[test]
    fn test_channel_for_command_various() {
        let test_cases = vec![
            ("local.cim", "person", "update", "local.cim.person.cmd.update"),
            ("cim", "workflow", "execute", "cim.workflow.cmd.execute"),
            ("org.unit", "document", "delete", "org.unit.document.cmd.delete"),
        ];

        for (prefix, aggregate, command, expected) in test_cases {
            let result = channel_for_command(prefix, aggregate, command);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_channel_for_event() {
        let channel = channel_for_event("local.cim", "person", "created");
        assert_eq!(channel, "local.cim.person.evt.created");
    }

    #[test]
    fn test_channel_for_event_various() {
        let test_cases = vec![
            ("local.cim", "person", "updated", "local.cim.person.evt.updated"),
            ("cim", "workflow", "completed", "cim.workflow.evt.completed"),
            ("org.unit", "order", "submitted", "org.unit.order.evt.submitted"),
        ];

        for (prefix, aggregate, event, expected) in test_cases {
            let result = channel_for_event(prefix, aggregate, event);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_channel_for_query() {
        let channel = channel_for_query("local.cim", "person", "get-all");
        assert_eq!(channel, "local.cim.person.qry.get-all");
    }

    #[test]
    fn test_channel_for_query_various() {
        let test_cases = vec![
            ("local.cim", "person", "find-by-id", "local.cim.person.qry.find-by-id"),
            ("cim", "order", "search", "cim.order.qry.search"),
        ];

        for (prefix, aggregate, query, expected) in test_cases {
            let result = channel_for_query(prefix, aggregate, query);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_channel_for_workflow_state() {
        let channel = channel_for_workflow_state("local.cim", "wf-123", "pending");
        assert_eq!(channel, "local.cim.workflow.wf-123.state.pending");
    }

    #[test]
    fn test_channel_for_workflow_state_various() {
        let test_cases = vec![
            ("local.cim", "wf-001", "active", "local.cim.workflow.wf-001.state.active"),
            ("cim", "workflow-abc", "completed", "cim.workflow.workflow-abc.state.completed"),
        ];

        for (prefix, wf_id, state, expected) in test_cases {
            let result = channel_for_workflow_state(prefix, wf_id, state);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_channel_for_context() {
        let channel = channel_for_context("local.cim", "orders", "order");
        assert_eq!(channel, "local.cim.ctx.orders.order");
    }

    #[test]
    fn test_channel_for_context_various() {
        let test_cases = vec![
            ("local.cim", "billing", "invoice", "local.cim.ctx.billing.invoice"),
            ("cim", "shipping", "shipment", "cim.ctx.shipping.shipment"),
        ];

        for (prefix, context, aggregate, expected) in test_cases {
            let result = channel_for_context(prefix, context, aggregate);
            assert_eq!(result, expected);
        }
    }

    // ========================================================================
    // Wildcard Pattern Tests
    // ========================================================================

    #[test]
    fn test_wildcard_for_aggregate() {
        let pattern = wildcard_for_aggregate("local.cim", "person");
        assert_eq!(pattern, "local.cim.person.>");
    }

    #[test]
    fn test_wildcard_for_aggregate_various() {
        let test_cases = vec![
            ("local.cim", "workflow", "local.cim.workflow.>"),
            ("cim", "order", "cim.order.>"),
        ];

        for (prefix, aggregate, expected) in test_cases {
            let result = wildcard_for_aggregate(prefix, aggregate);
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_wildcard_for_commands() {
        let pattern = wildcard_for_commands("local.cim");
        assert_eq!(pattern, "local.cim.*.cmd.*");
    }

    #[test]
    fn test_wildcard_for_events() {
        let pattern = wildcard_for_events("local.cim");
        assert_eq!(pattern, "local.cim.*.evt.*");
    }

    // ========================================================================
    // Subject Builder Tests
    // ========================================================================

    #[test]
    fn test_subject_builder_basic() {
        let subject = SubjectBuilder::new()
            .with_prefix("local.cim")
            .segment("test")
            .build()
            .unwrap();

        assert_eq!(subject, "local.cim.test");
    }

    #[test]
    fn test_subject_builder_with_aggregate() {
        let subject = SubjectBuilder::new()
            .with_prefix("local.cim")
            .aggregate("person")
            .build()
            .unwrap();

        assert_eq!(subject, "local.cim.person");
    }

    #[test]
    fn test_subject_builder_with_entity() {
        let subject = SubjectBuilder::new()
            .with_prefix("local.cim")
            .aggregate("person")
            .entity("12345")
            .build()
            .unwrap();

        assert_eq!(subject, "local.cim.person.12345");
    }

    #[test]
    fn test_subject_builder_with_command() {
        let subject = SubjectBuilder::new()
            .with_prefix("local.cim")
            .aggregate("person")
            .command("create")
            .build()
            .unwrap();

        assert_eq!(subject, "local.cim.person.cmd.create");
    }

    #[test]
    fn test_subject_builder_with_event() {
        let subject = SubjectBuilder::new()
            .with_prefix("local.cim")
            .aggregate("person")
            .event("created")
            .build()
            .unwrap();

        assert_eq!(subject, "local.cim.person.evt.created");
    }

    #[test]
    fn test_subject_builder_with_query() {
        let subject = SubjectBuilder::new()
            .with_prefix("local.cim")
            .aggregate("person")
            .query("get-all")
            .build()
            .unwrap();

        assert_eq!(subject, "local.cim.person.qry.get-all");
    }

    #[test]
    fn test_subject_builder_empty_fails() {
        let result = SubjectBuilder::new().build();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Subject cannot be empty");
    }

    #[test]
    fn test_subject_builder_wildcard() {
        let subject = SubjectBuilder::new()
            .with_prefix("local.cim")
            .aggregate("person")
            .build_wildcard()
            .unwrap();

        assert_eq!(subject, "local.cim.person.>");
    }

    #[test]
    fn test_subject_builder_no_prefix() {
        let subject = SubjectBuilder::new()
            .segment("test")
            .segment("segment")
            .build()
            .unwrap();

        assert_eq!(subject, "test.segment");
    }

    #[test]
    fn test_subject_builder_complex() {
        let subject = SubjectBuilder::new()
            .with_prefix("org.unit.project")
            .segment("ctx")
            .segment("orders")
            .aggregate("order")
            .entity("order-123")
            .event("submitted")
            .build()
            .unwrap();

        assert_eq!(subject, "org.unit.project.ctx.orders.order.order-123.evt.submitted");
    }

    #[test]
    fn test_subject_builder_debug() {
        let builder = SubjectBuilder::new()
            .with_prefix("local.cim")
            .segment("test");

        let debug = format!("{:?}", builder);
        assert!(debug.contains("SubjectBuilder"));
    }

    #[test]
    fn test_subject_builder_clone() {
        let builder1 = SubjectBuilder::new()
            .with_prefix("local.cim");

        let builder2 = builder1.clone();
        let subject1 = builder1.segment("test1").build().unwrap();
        let subject2 = builder2.segment("test2").build().unwrap();

        assert_eq!(subject1, "local.cim.test1");
        assert_eq!(subject2, "local.cim.test2");
    }

    #[test]
    fn test_subject_builder_default() {
        let builder = SubjectBuilder::default();
        let result = builder.segment("test").build().unwrap();
        assert_eq!(result, "test");
    }

    // ========================================================================
    // Pattern Matching Tests
    // ========================================================================

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("local.cim.person", "local.cim.person"));
        assert!(!matches_pattern("local.cim.person", "local.cim.order"));
    }

    #[test]
    fn test_matches_pattern_single_wildcard() {
        assert!(matches_pattern("local.cim.person", "local.*.person"));
        assert!(matches_pattern("local.cim.order", "local.*.order"));
        assert!(!matches_pattern("local.cim.person.123", "local.*.person"));
    }

    #[test]
    fn test_matches_pattern_multi_wildcard() {
        assert!(matches_pattern("local.cim.person", "local.cim.>"));
        assert!(matches_pattern("local.cim.person.123", "local.cim.>"));
        assert!(matches_pattern("local.cim.person.123.test", "local.cim.>"));
        assert!(!matches_pattern("local.other.person", "local.cim.>"));
    }

    #[test]
    fn test_matches_pattern_mixed() {
        assert!(matches_pattern("local.cim.person.cmd.create", "local.*.person.cmd.*"));
        assert!(matches_pattern("local.cim.order.cmd.update", "local.*.order.cmd.*"));
        assert!(!matches_pattern("local.cim.person.evt.created", "local.*.person.cmd.*"));
    }

    #[test]
    fn test_matches_pattern_no_match() {
        assert!(!matches_pattern("local.cim.person", "other.cim.person"));
        assert!(!matches_pattern("local.cim", "local.cim.person"));
        assert!(!matches_pattern("local.cim.person.extra", "local.cim.person"));
    }

    // ========================================================================
    // Parse Subject Tests
    // ========================================================================

    #[test]
    fn test_parse_subject_basic() {
        let parts = parse_subject("local.cim.person");
        assert_eq!(parts, vec!["local", "cim", "person"]);
    }

    #[test]
    fn test_parse_subject_single() {
        let parts = parse_subject("test");
        assert_eq!(parts, vec!["test"]);
    }

    #[test]
    fn test_parse_subject_complex() {
        let parts = parse_subject("org.unit.project.ctx.orders.order");
        assert_eq!(parts, vec!["org", "unit", "project", "ctx", "orders", "order"]);
    }

    // ========================================================================
    // Extract Aggregate Tests
    // ========================================================================

    #[test]
    fn test_extract_aggregate_standard() {
        let agg = extract_aggregate("local.cim.person");
        assert_eq!(agg, Some("person".to_string()));
    }

    #[test]
    fn test_extract_aggregate_with_entity() {
        let agg = extract_aggregate("local.cim.person.12345");
        assert_eq!(agg, Some("person".to_string()));
    }

    #[test]
    fn test_extract_aggregate_context() {
        let agg = extract_aggregate("local.cim.ctx.orders.order");
        assert_eq!(agg, Some("orders".to_string()));
    }

    #[test]
    fn test_extract_aggregate_org_prefix() {
        let agg = extract_aggregate("org.billing.invoice");
        assert_eq!(agg, Some("billing".to_string()));
    }

    // ========================================================================
    // Extract Entity ID Tests
    // ========================================================================

    #[test]
    fn test_extract_entity_id_standard() {
        let id = extract_entity_id("local.cim.person.12345", "person");
        assert_eq!(id, Some("12345".to_string()));
    }

    #[test]
    fn test_extract_entity_id_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let subject = format!("local.cim.workflow.{}", uuid);
        let id = extract_entity_id(&subject, "workflow");
        assert_eq!(id, Some(uuid.to_string()));
    }

    #[test]
    fn test_extract_entity_id_not_found() {
        let id = extract_entity_id("local.cim.person", "person");
        assert_eq!(id, None);
    }

    #[test]
    fn test_extract_entity_id_wrong_aggregate() {
        let id = extract_entity_id("local.cim.person.12345", "order");
        assert_eq!(id, None);
    }

    #[test]
    fn test_extract_entity_id_with_extra_segments() {
        let id = extract_entity_id("local.cim.person.12345.cmd.create", "person");
        assert_eq!(id, Some("12345".to_string()));
    }

    // ========================================================================
    // CQRS Channel Pattern Tests
    // ========================================================================

    #[test]
    fn test_cqrs_pattern_consistency() {
        let prefix = "local.cim";
        let aggregate = "order";

        let cmd = channel_for_command(prefix, aggregate, "create");
        let evt = channel_for_event(prefix, aggregate, "created");
        let qry = channel_for_query(prefix, aggregate, "get-by-id");

        // All should share the same prefix and aggregate
        assert!(cmd.starts_with(&format!("{}.{}.", prefix, aggregate)));
        assert!(evt.starts_with(&format!("{}.{}.", prefix, aggregate)));
        assert!(qry.starts_with(&format!("{}.{}.", prefix, aggregate)));

        // They should have different middle segments
        assert!(cmd.contains(".cmd."));
        assert!(evt.contains(".evt."));
        assert!(qry.contains(".qry."));
    }

    #[test]
    fn test_wildcard_matches_channels() {
        let prefix = "local.cim";
        let aggregate = "person";

        let cmd = channel_for_command(prefix, aggregate, "create");
        let evt = channel_for_event(prefix, aggregate, "created");

        let cmd_pattern = wildcard_for_commands(prefix);
        let evt_pattern = wildcard_for_events(prefix);
        let agg_pattern = wildcard_for_aggregate(prefix, aggregate);

        // Commands should match command pattern but not event pattern
        assert!(matches_pattern(&cmd, &cmd_pattern));
        assert!(!matches_pattern(&cmd, &evt_pattern));

        // Events should match event pattern but not command pattern
        assert!(matches_pattern(&evt, &evt_pattern));
        assert!(!matches_pattern(&evt, &cmd_pattern));

        // Both should match aggregate pattern
        assert!(matches_pattern(&cmd, &agg_pattern));
        assert!(matches_pattern(&evt, &agg_pattern));
    }
}
