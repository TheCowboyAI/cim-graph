# .rules to .claude Reorganization Plan

## New Directory Structure

```
.claude/
├── CLAUDE.md                    # Primary instructions (already exists)
├── INDEX.md                     # Navigation guide to all resources
├── instructions/                # Core operational instructions
│   ├── date-handling.md         # (already exists)
│   ├── cim-core.md             # From cim.mdc + cim-architecture.mdc
│   ├── conversation-model.md    # From cim-conversation-model.mdc
│   └── main-directives.md      # Priority rules from main.mdc
├── patterns/                    # Architectural patterns
│   ├── ddd.md                  # From ddd.mdc
│   ├── ddd-ecs.md              # From ddd-ecs.mdc
│   ├── event-sourcing.md       # From event-sourcing-cim.mdc
│   ├── conceptual-spaces.md    # From conceptual-spaces.mdc
│   └── graphs.md               # From graphs.mdc
├── standards/                   # Technical standards
│   ├── rust.md                 # From rust.mdc
│   ├── rust-nix.md             # From rust-nix.mdc
│   ├── nixos.md                # From nixos.mdc
│   ├── tdd.md                  # From tdd.mdc
│   ├── qa.md                   # From qa.mdc
│   └── mermaid-styling.md      # From mermaid-styling.md
├── architecture/                # Architecture documentation
│   └── module-dependencies.md   # From module-dependencies-and-responsibilities.md
├── security/                    # Security settings
│   └── settings.json           # From settings.local.json
└── deprecated/                  # Original .rules files for reference
    └── [original files]
```

## Priority Order for Content

1. **Critical Instructions** (goes to CLAUDE.md or instructions/)
   - Date handling rules
   - Core CIM concepts
   - Primary development workflow

2. **Architecture Patterns** (goes to patterns/)
   - DDD principles
   - Event sourcing
   - Conceptual spaces theory

3. **Technical Standards** (goes to standards/)
   - Language-specific rules
   - Testing methodologies
   - Documentation standards

4. **Reference Material** (stays in original location or architecture/)
   - Module dependencies
   - Styling guides

## Migration Strategy

1. Extract and consolidate overlapping content
2. Remove outdated or conflicting rules
3. Prioritize CIM-specific instructions
4. Create clear navigation in INDEX.md
5. Archive original files in deprecated/