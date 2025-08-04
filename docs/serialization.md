# Serialization Guide

CIM Graph provides comprehensive serialization support for persisting graphs, sharing them between systems, and integrating with different tools and frameworks.

## Table of Contents

1. [Supported Formats](#supported-formats)
2. [JSON Serialization](#json-serialization)
3. [Nix Expression Export](#nix-expression-export)
4. [Binary Serialization](#binary-serialization)
5. [Custom Serialization](#custom-serialization)
6. [Streaming Serialization](#streaming-serialization)
7. [Schema Evolution](#schema-evolution)
8. [Best Practices](#best-practices)

## Supported Formats

CIM Graph supports multiple serialization formats:

| Format | Use Case | Human Readable | Size | Speed |
|--------|----------|----------------|------|-------|
| JSON | Web APIs, Config | Yes | Large | Medium |
| Nix | Nix integration | Yes | Large | Slow |
| Binary | Storage, Transfer | No | Small | Fast |
| MessagePack | Efficient JSON | No | Medium | Fast |
| Protocol Buffers | Cross-language | No | Small | Fast |

## JSON Serialization

### Basic Usage

```rust
use cim_graph::serde_support::{to_json, from_json};

// Serialize to JSON
let graph = create_graph()?;
let json = to_json(&graph)?;
std::fs::write("graph.json", json)?;

// Deserialize from JSON
let json_str = std::fs::read_to_string("graph.json")?;
let loaded_graph: IpldGraph = from_json(&json_str)?;
```

### JSON Schema

The standard JSON format for graphs:

```json
{
  "version": "1.0",
  "graph_type": "IpldGraph",
  "metadata": {
    "created_at": "2024-01-15T10:00:00Z",
    "author": "system",
    "description": "Content addressing graph"
  },
  "nodes": [
    {
      "id": "node-1",
      "type": "ipld",
      "data": {
        "cid": "QmXoypizjW3WknFi...",
        "codec": "dag-cbor",
        "size": 1024
      },
      "metadata": {}
    }
  ],
  "edges": [
    {
      "id": "edge-1",
      "source": "node-1",
      "target": "node-2",
      "type": "contains",
      "weight": 1.0,
      "data": {
        "path": "/data/users"
      },
      "metadata": {}
    }
  ]
}
```

### Pretty Printing

```rust
use cim_graph::serde_support::{to_json_pretty, JsonConfig};

// Pretty print with custom config
let config = JsonConfig::new()
    .with_indent(2)
    .with_sort_keys(true)
    .with_escape_unicode(false);

let pretty_json = to_json_pretty(&graph, config)?;
```

### Selective Serialization

```rust
use cim_graph::serde_support::{SerializeOptions, NodeFilter};

// Serialize only specific nodes
let options = SerializeOptions::new()
    .with_node_filter(|node| node.metadata.get("public") == Some(&true))
    .with_edge_filter(|edge| edge.weight > 0.5)
    .exclude_metadata(false);

let filtered_json = to_json_with_options(&graph, options)?;
```

## Nix Expression Export

Export graphs as Nix expressions for integration with Nix package manager:

### Basic Export

```rust
use cim_graph::serde_support::to_nix;

let nix_expr = to_nix(&graph)?;
std::fs::write("graph.nix", nix_expr)?;
```

### Nix Format Example

```nix
{
  version = "1.0";
  graphType = "ContextGraph";
  metadata = {
    createdAt = "2024-01-15T10:00:00Z";
    boundedContext = "sales";
  };
  nodes = [
    {
      id = "customer-123";
      type = "Customer";
      data = {
        name = "Alice Smith";
        email = "alice@example.com";
        tier = "premium";
      };
    }
    {
      id = "order-456";
      type = "Order";
      data = {
        total = 150.00;
        status = "pending";
      };
    }
  ];
  edges = [
    {
      id = "edge-1";
      source = "customer-123";
      target = "order-456";
      type = "placed";
      data = {
        timestamp = "2024-01-15T09:30:00Z";
      };
    }
  ];
}
```

### Nix Function Generation

Generate Nix functions that operate on graphs:

```rust
use cim_graph::serde_support::{NixBuilder, NixFunction};

let nix = NixBuilder::new()
    .add_graph("salesGraph", &graph)
    .add_function(NixFunction::new("findCustomerOrders")
        .with_param("customerId")
        .with_body(r#"
          builtins.filter 
            (edge: edge.source == customerId && edge.type == "placed")
            salesGraph.edges
        "#))
    .build()?;

std::fs::write("graph-utils.nix", nix)?;
```

## Binary Serialization

For efficient storage and network transfer:

### Using Bincode

```rust
use cim_graph::serde_support::{to_binary, from_binary};

// Serialize to binary
let bytes = to_binary(&graph)?;
std::fs::write("graph.bin", bytes)?;

// Deserialize from binary
let bytes = std::fs::read("graph.bin")?;
let graph: WorkflowGraph = from_binary(&bytes)?;

// Compressed binary
let compressed = to_binary_compressed(&graph)?;
```

### Using MessagePack

```rust
use cim_graph::serde_support::{to_msgpack, from_msgpack};

// More efficient than JSON, still somewhat readable
let msgpack_data = to_msgpack(&graph)?;
let restored = from_msgpack::<ConceptGraph>(&msgpack_data)?;
```

### Using Protocol Buffers

First, define your schema in `graph.proto`:

```protobuf
syntax = "proto3";

message Graph {
  string version = 1;
  string graph_type = 2;
  repeated Node nodes = 3;
  repeated Edge edges = 4;
}

message Node {
  string id = 1;
  string type = 2;
  google.protobuf.Any data = 3;
}

message Edge {
  string id = 1;
  string source = 2;
  string target = 3;
  double weight = 4;
}
```

Then use in Rust:

```rust
use cim_graph::serde_support::{to_protobuf, from_protobuf};

let proto_bytes = to_protobuf(&graph)?;
let restored = from_protobuf::<IpldGraph>(&proto_bytes)?;
```

## Custom Serialization

### Implementing Custom Formats

```rust
use cim_graph::{Graph, Node, Edge};
use cim_graph::serde_support::{Serializer, Deserializer};

pub struct GraphMLSerializer;

impl<G: Graph> Serializer<G> for GraphMLSerializer {
    type Output = String;
    type Error = GraphMLError;
    
    fn serialize(&self, graph: &G) -> Result<Self::Output, Self::Error> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns">"#);
        xml.push_str("<graph>");
        
        // Serialize nodes
        for node_id in graph.node_indices() {
            let node = graph.get_node(node_id)?;
            xml.push_str(&format!(
                r#"<node id="{}"><data>{:?}</data></node>"#,
                node_id, node
            ));
        }
        
        // Serialize edges
        for edge_id in graph.edge_indices() {
            let (source, target) = graph.edge_endpoints(edge_id)?;
            let edge = graph.get_edge(edge_id)?;
            xml.push_str(&format!(
                r#"<edge source="{}" target="{}"><data>{:?}</data></edge>"#,
                source, target, edge
            ));
        }
        
        xml.push_str("</graph></graphml>");
        Ok(xml)
    }
}
```

### Format Adapters

```rust
use cim_graph::serde_support::{FormatAdapter, Format};

// Adapt graph to different formats
let adapter = FormatAdapter::new(&graph);

// Export to various formats
let dot = adapter.to_format(Format::Graphviz)?;
let cypher = adapter.to_format(Format::Cypher)?;
let gexf = adapter.to_format(Format::GEXF)?;
```

## Streaming Serialization

For very large graphs that don't fit in memory:

### Streaming JSON Writer

```rust
use cim_graph::serde_support::{StreamingJsonWriter, StreamingOptions};
use std::io::BufWriter;

let file = std::fs::File::create("large_graph.json")?;
let writer = BufWriter::new(file);

let mut streaming = StreamingJsonWriter::new(writer);
streaming.begin_graph("LargeGraph", metadata)?;

// Stream nodes
for node in node_iterator {
    streaming.write_node(&node)?;
}

// Stream edges  
for edge in edge_iterator {
    streaming.write_edge(&edge)?;
}

streaming.end_graph()?;
```

### Streaming JSON Reader

```rust
use cim_graph::serde_support::{StreamingJsonReader, NodeHandler, EdgeHandler};

let file = std::fs::File::open("large_graph.json")?;
let mut reader = StreamingJsonReader::new(file);

// Process nodes as they're read
reader.on_node(|node| {
    println!("Processing node: {:?}", node.id);
    process_node(node);
    Ok(())
})?;

// Process edges as they're read
reader.on_edge(|edge| {
    println!("Processing edge: {} -> {}", edge.source, edge.target);
    process_edge(edge);
    Ok(())
})?;

// Start streaming
reader.read()?;
```

### Chunked Serialization

```rust
use cim_graph::serde_support::{ChunkedSerializer, ChunkSize};

// Serialize in chunks of 1000 nodes
let chunks = ChunkedSerializer::new(&graph)
    .with_chunk_size(ChunkSize::Nodes(1000))
    .serialize()?;

for (i, chunk) in chunks.enumerate() {
    let filename = format!("graph_chunk_{:04}.json", i);
    std::fs::write(filename, chunk)?;
}

// Deserialize chunks
let graph = ChunkedDeserializer::new()
    .add_chunk_file("graph_chunk_0000.json")?
    .add_chunk_file("graph_chunk_0001.json")?
    .deserialize::<LargeGraph>()?;
```

## Schema Evolution

Handle changes to graph structure over time:

### Versioned Serialization

```rust
use cim_graph::serde_support::{Version, VersionedSerializer};

#[derive(Serialize, Deserialize)]
#[serde(tag = "version")]
enum VersionedGraph {
    #[serde(rename = "1.0")]
    V1(GraphV1),
    #[serde(rename = "2.0")]
    V2(GraphV2),
}

// Serialize with version
let versioned = VersionedSerializer::new()
    .with_version(Version::new(2, 0))
    .serialize(&graph)?;

// Deserialize with automatic migration
let graph = VersionedDeserializer::new()
    .with_migration(Version::new(1, 0), Version::new(2, 0), migrate_v1_to_v2)
    .deserialize(&data)?;
```

### Migration Functions

```rust
fn migrate_v1_to_v2(v1: GraphV1) -> Result<GraphV2> {
    let mut v2 = GraphV2::new();
    
    // Migrate nodes
    for node in v1.nodes {
        let v2_node = NodeV2 {
            id: node.id,
            type_name: node.node_type, // renamed field
            data: migrate_node_data(node.data)?,
            // new field with default
            created_at: Utc::now(),
        };
        v2.add_node(v2_node)?;
    }
    
    // Migrate edges
    for edge in v1.edges {
        v2.add_edge(migrate_edge(edge)?)?;
    }
    
    Ok(v2)
}
```

## Best Practices

### 1. Choose the Right Format

```rust
// For web APIs and human inspection
let json = to_json_pretty(&graph)?;

// For storage and performance
let binary = to_binary_compressed(&graph)?;

// For cross-language compatibility
let protobuf = to_protobuf(&graph)?;
```

### 2. Validate on Deserialization

```rust
use cim_graph::serde_support::{ValidationOptions, GraphValidator};

let validator = GraphValidator::new()
    .require_connected(true)
    .require_dag(true)
    .max_nodes(10_000);

let graph = from_json_validated(&json, validator)?;
```

### 3. Handle Large Graphs

```rust
// Use streaming for graphs > 100MB
if graph.node_count() > 1_000_000 {
    use_streaming_serialization(&graph)?;
} else {
    use_standard_serialization(&graph)?;
}
```

### 4. Include Metadata

```rust
use cim_graph::serde_support::Metadata;

let metadata = Metadata::new()
    .with_created_at(Utc::now())
    .with_author("system")
    .with_description("Daily snapshot")
    .with_custom("environment", "production");

let json = to_json_with_metadata(&graph, metadata)?;
```

### 5. Compression

```rust
use cim_graph::serde_support::Compression;

// Choose compression based on use case
let compressed = match use_case {
    UseCase::Storage => to_binary_compressed(&graph, Compression::Zstd)?,
    UseCase::Network => to_binary_compressed(&graph, Compression::Lz4)?,
    UseCase::Archive => to_binary_compressed(&graph, Compression::Xz)?,
};
```

### 6. Error Handling

```rust
use cim_graph::serde_support::{SerializationError, DeserializationError};

match from_json::<IpldGraph>(&json) {
    Ok(graph) => process_graph(graph),
    Err(DeserializationError::InvalidVersion(v)) => {
        println!("Unsupported version: {}", v);
        try_migration()?
    }
    Err(DeserializationError::MissingField(field)) => {
        println!("Missing required field: {}", field);
        use_defaults()?
    }
    Err(e) => return Err(e.into()),
}
```

### 7. Security Considerations

```rust
use cim_graph::serde_support::{SecurityOptions, Sanitizer};

// Sanitize untrusted input
let options = SecurityOptions::new()
    .max_string_length(1024)
    .max_collection_size(10_000)
    .forbid_custom_types(true);

let graph = from_json_secure(&untrusted_json, options)?;
```