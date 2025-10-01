# CIM Graph Refactoring Implementation Plan

## Overview

This plan details the systematic refactoring of cim-graph to align with CIM's event-driven architecture. Each sprint focuses on a specific aspect of the transformation.

## Sprint 1: Foundation Cleanup (Days 1-3)

### Goals
- Remove all mutable abstractions
- Establish event-driven foundation

### Tasks
1. **Remove Graph trait** ✅
   - Delete trait definition
   - Update all imports
   - Remove from exports

2. **Remove GraphBuilder** ✅
   - Delete builder module
   - Update documentation
   - Remove examples using builder

3. **Remove EventGraph** ✅
   - Delete petgraph implementation
   - Remove event handler mixing

4. **Create core event schemas** ✅
   - GraphEvent structure
   - EventPayload variants
   - Command definitions

### Deliverables
- Clean codebase without mutations
- Event schema module
- Updated module exports

## Sprint 2: Projection System (Days 4-6)

### Goals
- Implement read-only projections
- Create projection engines

### Tasks
1. **GraphProjection trait** ✅
   - Read-only interface
   - Query methods only
   - Version tracking

2. **Aggregate projections** ✅
   - ECS component storage
   - Relationship tracking
   - Left-fold implementation

3. **State machine** ✅
   - Command validation
   - Event generation
   - Transition rules

4. **Projection builders**
   - Generic projection engine ✅
   - Type-specific builders ⏳

### Deliverables
- Working projection system
- State machine implementation
- Query systems

## Sprint 3: IPLD Integration (Days 7-9)

### Goals
- Integrate IPLD as storage heart
- Implement CID chains

### Tasks
1. **CID event payloads** ✅
   - Payload → CID generation
   - Merkle DAG construction
   - Chain tracking

2. **Event chain builder** ✅
   - Previous CID linking
   - Root CID management
   - Chain verification

3. **IPLD projection engine** ✅
   - CID-based projections
   - Content addressing
   - Immutable verification

4. **cim-ipld integration** ⏳
   - Real CID generation
   - IPLD codecs
   - DAG operations

### Deliverables
- CID chain implementation
- IPLD projection engine
- Documentation on storage architecture

## Sprint 4: Graph Type Migration (Days 10-15)

### Goals
- Convert all graph types to projections
- Maintain type-specific semantics

### Tasks
1. **IpldGraph → IpldProjection** ⏳
   - Content-addressed nodes
   - CID-based edges
   - Pin/unpin metadata

2. **ContextGraph → ContextProjection** ⏳
   - Bounded contexts
   - Aggregates/entities
   - DDD relationships

3. **WorkflowGraph → WorkflowProjection** ⏳
   - States as entities
   - Transitions as events
   - Instance tracking

4. **ConceptGraph → ConceptProjection** ⏳
   - Concepts as entities
   - Properties as components
   - Inference events

5. **ComposedGraph → ComposedProjection** ⏳
   - Multi-graph aggregation
   - Cross-graph queries
   - Namespace isolation

### Deliverables
- All graph types as projections
- Type-specific command handlers
- Migration examples

## Sprint 5: System Integration (Days 16-20)

### Goals
- Integrate with NATS JetStream
- Enable collaborative features

### Tasks
1. **NATS event publishing** ⏳
   - Event → JetStream
   - Subject routing
   - Headers (correlation, causation)

2. **Event replay** ⏳
   - Fetch by aggregate
   - Replay from sequence
   - Catchup mechanism

3. **cim-domain subject module integration** ⏳
   - Subject algebra
   - Graph subjects
   - Event routing

4. **Collaborative features** ⏳
   - Shared subscriptions
   - Real-time updates
   - Conflict-free design

### Deliverables
- NATS integration module
- Collaborative examples
- Subject documentation

## Sprint 6: Testing & Documentation (Days 21-25)

### Goals
- Comprehensive testing
- Complete documentation

### Tasks
1. **Remove mutation tests** ⏳
   - Delete invalid tests
   - Update test strategy
   - Coverage analysis

2. **Add projection tests** ⏳
   - Event folding tests
   - Query system tests
   - State machine tests

3. **Integration tests** ⏳
   - End-to-end flows
   - NATS integration
   - Multi-client scenarios

4. **Documentation update** ⏳
   - Architecture guide
   - Migration guide
   - API reference

### Deliverables
- 90%+ test coverage
- Complete documentation
- Migration tooling

## Success Metrics

### Technical Metrics
- [ ] 0 mutation methods remaining
- [ ] 100% events have CIDs
- [ ] All state changes through StateMachine
- [ ] Full NATS integration
- [ ] Projection performance < 10ms

### Quality Metrics
- [ ] Test coverage > 90%
- [ ] Documentation coverage 100%
- [ ] All examples updated
- [ ] Breaking changes documented
- [ ] Migration guide complete

## Risk Management

### High Priority Risks
1. **API Breaking Changes**
   - Mitigation: Compatibility layer
   - Timeline impact: +3 days

2. **Performance Regression**
   - Mitigation: Projection caching
   - Timeline impact: +2 days

### Medium Priority Risks
1. **Integration Complexity**
   - Mitigation: Incremental integration
   - Timeline impact: +2 days

2. **Learning Curve**
   - Mitigation: Extensive examples
   - Timeline impact: +1 day

## Dependencies

### External Dependencies
- `cim-ipld` - For CID generation
- `cim-domain` (subject module) - For subject algebra
- `cim-domain` - For event patterns
- `nats` - For JetStream client

### Internal Dependencies
- Complete event schema before projections
- Projections before type migrations
- State machine before command handlers

## Communication Plan

### Daily Updates
- Progress against sprint goals
- Blockers and risks
- Next day priorities

### Sprint Reviews
- Completed deliverables
- Lessons learned
- Plan adjustments

### Stakeholder Updates
- Weekly progress summary
- Risk status
- Timeline adjustments

## Conclusion

This plan provides a systematic approach to refactoring cim-graph into a proper event-driven system. The phased approach minimizes risk while ensuring each component is properly integrated. Total timeline: 25 working days.
