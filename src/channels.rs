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

pub fn channel_for_aggregate(prefix: &str, aggregate: &str) -> String {
    subject_from(prefix, &[aggregate])
}

pub fn channel_for_entity(prefix: &str, aggregate: &str, id: &str) -> String {
    subject_from(prefix, &[aggregate, id])
}

pub fn channel_for_cid(prefix: &str, cid: &str) -> String {
    subject_from(prefix, &["cid", cid])
}

pub fn channel_for_bucket(prefix: &str, bucket: &str) -> String {
    subject_from(prefix, &["bucket", bucket])
}

/// Validate a filter subject pattern (wildcards allowed) using SubjectPattern.
pub fn validate_pattern(pattern: &str) -> Result<(), String> {
    SubjectPattern::parse(pattern).map(|_| ()).map_err(|e| e.to_string())
}

