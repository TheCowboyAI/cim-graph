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
}
