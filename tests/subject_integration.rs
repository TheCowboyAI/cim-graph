//! Integration tests for cim-domain subject module integration

#[cfg(feature = "nats")]
mod tests {
    use cim_graph::events::{
        build_event_subject, build_graph_subscription, build_type_subscription,
        build_all_events_subscription, GraphType, EventType,
    };
    use uuid::Uuid;
    
    #[test]
    fn test_subject_patterns() {
        let aggregate_id = Uuid::new_v4();
        
        // Test event subjects
        let subject = build_event_subject(
            GraphType::Workflow,
            aggregate_id,
            EventType::Created
        );
        assert_eq!(
            subject,
            format!("cim.graph.workflow.{}.created", aggregate_id)
        );
        
        // Test graph subscription
        let graph_sub = build_graph_subscription(GraphType::Ipld, aggregate_id);
        assert_eq!(
            graph_sub,
            format!("cim.graph.ipld.{}.*", aggregate_id)
        );
        
        // Test type subscription
        let type_sub = build_type_subscription(GraphType::Context);
        assert_eq!(type_sub, "cim.graph.context.>");
        
        // Test all events subscription
        let all_sub = build_all_events_subscription();
        assert_eq!(all_sub, "cim.graph.>");
    }
    
    #[test]
    fn test_subject_hierarchy() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        
        // Different aggregates should have different subjects
        let sub1 = build_event_subject(GraphType::Concept, id1, EventType::NodeAdded);
        let sub2 = build_event_subject(GraphType::Concept, id2, EventType::NodeAdded);
        assert_ne!(sub1, sub2);
        
        // Different event types should have different subjects
        let sub3 = build_event_subject(GraphType::Composed, id1, EventType::Created);
        let sub4 = build_event_subject(GraphType::Composed, id1, EventType::Deleted);
        assert_ne!(sub3, sub4);
        
        // Different graph types should have different subjects
        let sub5 = build_event_subject(GraphType::Workflow, id1, EventType::Updated);
        let sub6 = build_event_subject(GraphType::Context, id1, EventType::Updated);
        assert_ne!(sub5, sub6);
    }
}
