//! Real-world example: Order Processing System
//! 
//! This example demonstrates building an order processing system using
//! the event-driven architecture with multiple bounded contexts working together.

use cim_graph::{
    core::{
        PolicyEngine,
        CidGenerationPolicy, StateValidationPolicy,
    },
    events::{
        GraphEvent, EventPayload, WorkflowPayload, ConceptPayload, 
        ContextPayload, ComposedPayload,
    },
    serde_support::{EventJournal},
};
use uuid::Uuid;

/// Represents an order in our system
#[derive(Debug)]
struct Order {
    id: Uuid,
    customer_id: String,
    items: Vec<OrderItem>,
    #[allow(dead_code)]
    status: OrderStatus,
}

#[derive(Debug)]
struct OrderItem {
    #[allow(dead_code)]
    product_id: String,
    quantity: u32,
    price: f64,
}

#[derive(Debug, PartialEq)]
#[allow(dead_code)]
enum OrderStatus {
    Submitted,
    PaymentPending,
    PaymentReceived,
    Preparing,
    Shipped,
    Delivered,
    Cancelled,
}

/// Order processing system using event-driven architecture
struct OrderProcessingSystem {
    workflow_id: Uuid,
    concept_id: Uuid,
    context_id: Uuid,
    composed_id: Uuid,
    events: Vec<GraphEvent>,
    policy_engine: PolicyEngine,
}

impl OrderProcessingSystem {
    fn new() -> Self {
        let mut policy_engine = PolicyEngine::new();
        policy_engine.add_policy(Box::new(CidGenerationPolicy));
        policy_engine.add_policy(Box::new(StateValidationPolicy));
        
        let workflow_id = Uuid::new_v4();
        let concept_id = Uuid::new_v4();
        let context_id = Uuid::new_v4();
        let composed_id = Uuid::new_v4();
        
        let mut system = Self {
            workflow_id,
            concept_id,
            context_id,
            composed_id,
            events: Vec::new(),
            policy_engine,
        };
        
        // Initialize the system
        system.initialize();
        system
    }
    
    fn initialize(&mut self) {
        let correlation_id = Uuid::new_v4();
        
        // 1. Define the order processing workflow
        self.add_event(GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: self.workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                workflow_id: self.workflow_id,
                name: "Order Processing Workflow".to_string(),
                version: "1.0.0".to_string(),
            }),
        });
        
        // Add workflow states
        let states = vec![
            ("submitted", "initial"),
            ("payment_pending", "normal"),
            ("payment_received", "normal"),
            ("preparing", "normal"),
            ("shipped", "normal"),
            ("delivered", "final"),
            ("cancelled", "final"),
        ];
        
        for (state_id, state_type) in states {
            self.add_event(GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: self.workflow_id,
                correlation_id,
                causation_id: Some(self.events.last().unwrap().event_id),
                payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                    workflow_id: self.workflow_id,
                    state_id: state_id.to_string(),
                    state_type: state_type.to_string(),
                }),
            });
        }
        
        // Add transitions
        let transitions = vec![
            ("submitted", "payment_pending", "process_payment"),
            ("payment_pending", "payment_received", "payment_confirmed"),
            ("payment_pending", "cancelled", "payment_failed"),
            ("payment_received", "preparing", "start_preparation"),
            ("preparing", "shipped", "ship_order"),
            ("shipped", "delivered", "confirm_delivery"),
            ("payment_received", "cancelled", "cancel_order"),
            ("preparing", "cancelled", "cancel_order"),
        ];
        
        for (from, to, trigger) in transitions {
            self.add_event(GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: self.workflow_id,
                correlation_id,
                causation_id: Some(self.events.last().unwrap().event_id),
                payload: EventPayload::Workflow(WorkflowPayload::TransitionAdded {
                    workflow_id: self.workflow_id,
                    from_state: from.to_string(),
                    to_state: to.to_string(),
                    trigger: trigger.to_string(),
                }),
            });
        }
        
        // 2. Define domain concepts
        let concepts = vec![
            ("order", "Order", "aggregate"),
            ("customer", "Customer", "entity"),
            ("product", "Product", "entity"),
            ("payment", "Payment", "value_object"),
            ("shipment", "Shipment", "entity"),
        ];
        
        for (concept_id, name, concept_type) in concepts {
            self.add_event(GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: self.concept_id,
                correlation_id,
                causation_id: None,
                payload: EventPayload::Concept(ConceptPayload::ConceptDefined {
                    concept_id: concept_id.to_string(),
                    name: name.to_string(),
                    definition: format!("{} in the order processing domain", concept_type),
                }),
            });
        }
        
        // Add relationships
        let relationships = vec![
            ("order", "customer", "placed_by"),
            ("order", "product", "contains"),
            ("order", "payment", "paid_with"),
            ("order", "shipment", "shipped_via"),
        ];
        
        for (from, to, relation) in relationships {
            self.add_event(GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: self.concept_id,
                correlation_id,
                causation_id: None,
                payload: EventPayload::Concept(ConceptPayload::RelationAdded {
                    source_concept: from.to_string(),
                    target_concept: to.to_string(),
                    relation_type: relation.to_string(),
                    strength: 1.0,
                }),
            });
        }
        
        // 3. Define bounded context
        self.add_event(GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: self.context_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Context(ContextPayload::BoundedContextCreated {
                context_id: "order_management".to_string(),
                name: "Order Management".to_string(),
                description: "Handles order lifecycle and processing".to_string(),
            }),
        });
        
        // 4. Compose everything together
        self.add_event(GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: self.composed_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                subgraph_id: self.workflow_id,
                graph_type: "workflow".to_string(),
                namespace: "processes".to_string(),
            }),
        });
        
        self.add_event(GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: self.composed_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Composed(ComposedPayload::SubGraphAdded {
                subgraph_id: self.concept_id,
                graph_type: "concept".to_string(),
                namespace: "domain".to_string(),
            }),
        });
    }
    
    fn add_event(&mut self, event: GraphEvent) {
        self.events.push(event);
    }
    
    /// Process a new order
    fn process_order(&mut self, order: Order) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Processing Order {} ===", order.id);
        
        let correlation_id = Uuid::new_v4();
        let instance_id = order.id;
        
        // Apply policies to validate the order
        use cim_graph::core::{GraphStateMachine, PolicyContext, PolicyMetrics};
        use std::collections::HashMap;
        let mut state_machine = GraphStateMachine::new();
        let mut ipld_chains = HashMap::new();
        let mut policy_context = PolicyContext {
            state_machine: &mut state_machine,
            ipld_chains: &mut ipld_chains,
            metrics: PolicyMetrics::default(),
        };
        
        // Create workflow instance
        self.add_event(GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: self.workflow_id,
            correlation_id,
            causation_id: None,
            payload: EventPayload::Workflow(WorkflowPayload::InstanceCreated {
                workflow_id: self.workflow_id,
                instance_id,
                initial_state: "submitted".to_string(),
            }),
        });
        
        // Start workflow
        self.add_event(GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: self.workflow_id,
            correlation_id,
            causation_id: Some(self.events.last().unwrap().event_id),
            payload: EventPayload::Workflow(WorkflowPayload::StateTransitioned {
                instance_id,
                from_state: "submitted".to_string(),
                to_state: "payment_pending".to_string(),
            }),
        });
        
        // Execute policies on the created events
        for event in self.events.iter().skip(self.events.len().saturating_sub(2)) {
            match self.policy_engine.execute_policies(event, &mut policy_context) {
                Ok(actions) => {
                    if !actions.is_empty() {
                        println!("Policy actions generated: {}", actions.len());
                    }
                }
                Err(e) => {
                    println!("Policy error: {}", e);
                }
            }
        }
        
        println!("Order submitted, starting payment processing...");
        
        // Trigger payment processing
        self.trigger_transition(instance_id, "process_payment", correlation_id)?;
        
        Ok(())
    }
    
    /// Trigger a workflow transition
    fn trigger_transition(
        &mut self, 
        instance_id: Uuid, 
        trigger: &str,
        correlation_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.add_event(GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: self.workflow_id,
            correlation_id,
            causation_id: self.events.last().map(|e| e.event_id),
            payload: EventPayload::Workflow(WorkflowPayload::StateTransitioned {
                instance_id,
                from_state: "current".to_string(), // In a real system, we'd track current state
                to_state: trigger.to_string(),
            }),
        });
        
        println!("Triggered transition: {}", trigger);
        Ok(())
    }
    
    /// Build aggregate projection from events
    fn build_projection(&self) -> cim_graph::core::GraphAggregateProjection {
        use cim_graph::core::build_projection;
        let event_tuples: Vec<(GraphEvent, u64)> = self.events.iter()
            .enumerate()
            .map(|(i, e)| (e.clone(), i as u64 + 1))
            .collect();
        build_projection(event_tuples)
    }
    
    /// Save all events to file
    fn save_events(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let journal = EventJournal::new(self.events.clone());
        journal.save_to_file(filename)?;
        println!("\nSaved {} events to {}", self.events.len(), filename);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Order Processing System Demo ===");
    
    // Create the system
    let mut system = OrderProcessingSystem::new();
    
    // Create a sample order
    let order = Order {
        id: Uuid::new_v4(),
        customer_id: "CUST-001".to_string(),
        items: vec![
            OrderItem {
                product_id: "PROD-001".to_string(),
                quantity: 2,
                price: 29.99,
            },
            OrderItem {
                product_id: "PROD-002".to_string(),
                quantity: 1,
                price: 49.99,
            },
        ],
        status: OrderStatus::Submitted,
    };
    
    let total = order.items.iter().map(|i| i.price * i.quantity as f64).sum::<f64>();
    println!("\nProcessing order:");
    println!("  Customer: {}", order.customer_id);
    println!("  Items: {}", order.items.len());
    println!("  Total: ${:.2}", total);
    
    // Process the order
    system.process_order(order)?;
    
    // Simulate payment confirmation
    let instance_id = Uuid::new_v4(); // In real system, track this
    println!("\nSimulating payment confirmation...");
    system.trigger_transition(instance_id, "payment_confirmed", Uuid::new_v4())?;
    
    // Build projection and show stats
    let projection = system.build_projection();
    println!("\nSystem Statistics:");
    println!("  Total events: {}", system.events.len());
    println!("  Components: {}", projection.components.len());
    println!("  Relationships: {}", projection.relationships.len());
    
    // Save events for persistence
    system.save_events("order_events.json")?;
    
    // Show event causation chain
    println!("\nEvent Causation Chain (last 5 events):");
    for event in system.events.iter().rev().take(5) {
        println!("  Event {}: caused by {:?}", 
                 event.event_id, 
                 event.causation_id);
    }
    
    println!("\n=== Demo Complete ===");
    println!("This demonstrated:");
    println!("- Building a complete order processing system");
    println!("- Using workflows for business processes");
    println!("- Domain concepts for business entities");
    println!("- Event sourcing with causation tracking");
    println!("- Saving events for persistence");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_order_processing() {
        main().unwrap();
    }
}