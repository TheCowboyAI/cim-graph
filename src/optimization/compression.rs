//! Event stream compression for efficient storage and transmission

use crate::events::GraphEvent;
use crate::error::{GraphError, Result};
use std::io::{Read, Write, BufRead, BufReader};

/// Compressed event stream using zstd compression
#[derive(Debug, Clone)]
pub struct CompressedEventStream {
    /// Compressed data
    data: Vec<u8>,
    /// Original event count
    event_count: usize,
    /// Compression ratio
    compression_ratio: f64,
}

impl CompressedEventStream {
    /// Decompress events
    pub fn decompress(&self) -> Result<Vec<GraphEvent>> {
        let decompressed = zstd::decode_all(&self.data[..])
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let events: Vec<GraphEvent> = bincode::deserialize(&decompressed)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        Ok(events)
    }
    
    /// Get compression ratio (compressed size / original size)
    pub fn compression_ratio(&self) -> f64 {
        self.compression_ratio
    }
    
    /// Get compressed size in bytes
    pub fn compressed_size(&self) -> usize {
        self.data.len()
    }
    
    /// Get event count
    pub fn event_count(&self) -> usize {
        self.event_count
    }
}

/// Event compressor with configurable compression level
#[derive(Debug)]
pub struct EventCompressor {
    /// Compression level (1-22, higher = better compression but slower)
    compression_level: i32,
}

impl Default for EventCompressor {
    fn default() -> Self {
        Self {
            compression_level: 3, // Good balance of speed and compression
        }
    }
}

impl EventCompressor {
    /// Create a new compressor with specified level
    pub fn new(compression_level: i32) -> Self {
        Self {
            compression_level: compression_level.clamp(1, 22),
        }
    }
    
    /// Compress a batch of events
    pub fn compress(&self, events: &[GraphEvent]) -> Result<CompressedEventStream> {
        // Serialize events to binary
        let serialized = bincode::serialize(events)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let original_size = serialized.len();
        
        // Compress with zstd
        let compressed = zstd::encode_all(&serialized[..], self.compression_level)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let compressed_size = compressed.len();
        let compression_ratio = compressed_size as f64 / original_size as f64;
        
        Ok(CompressedEventStream {
            data: compressed,
            event_count: events.len(),
            compression_ratio,
        })
    }
    
    /// Compress and write to a writer
    pub fn compress_to<W: Write>(&self, events: &[GraphEvent], writer: W) -> Result<()> {
        let serialized = bincode::serialize(events)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let mut encoder = zstd::Encoder::new(writer, self.compression_level)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        encoder.write_all(&serialized)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        encoder.finish()
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        Ok(())
    }
    
    /// Decompress from a reader
    pub fn decompress_from<R: BufRead>(reader: R) -> Result<Vec<GraphEvent>> {
        let mut decoder = zstd::Decoder::new(reader)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let events: Vec<GraphEvent> = bincode::deserialize(&decompressed)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        Ok(events)
    }
}

/// Streaming event compressor for large event streams
pub struct StreamingCompressor<W: Write> {
    encoder: zstd::Encoder<'static, W>,
    event_count: usize,
}

impl<W: Write> std::fmt::Debug for StreamingCompressor<W> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingCompressor")
            .field("event_count", &self.event_count)
            .finish()
    }
}

impl<W: Write> StreamingCompressor<W> {
    /// Create a new streaming compressor
    pub fn new(writer: W, compression_level: i32) -> Result<Self> {
        let encoder = zstd::Encoder::new(writer, compression_level)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        Ok(Self {
            encoder,
            event_count: 0,
        })
    }
    
    /// Add an event to the stream
    pub fn add_event(&mut self, event: &GraphEvent) -> Result<()> {
        let serialized = bincode::serialize(event)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        // Write length prefix for framing
        let len = serialized.len() as u32;
        self.encoder.write_all(&len.to_le_bytes())
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        // Write serialized event
        self.encoder.write_all(&serialized)
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        self.event_count += 1;
        Ok(())
    }
    
    /// Finish compression and return event count
    pub fn finish(self) -> Result<usize> {
        self.encoder.finish()
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        Ok(self.event_count)
    }
}

/// Streaming event decompressor
pub struct StreamingDecompressor<R> {
    reader: R,
}

impl<R> std::fmt::Debug for StreamingDecompressor<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingDecompressor")
            .finish()
    }
}

impl<R: Read> StreamingDecompressor<R> {
    /// Create a new streaming decompressor  
    pub fn new(reader: R) -> Result<Self> {
        Ok(Self { reader })
    }
    
    /// Read the next event from the stream
    pub fn next_event(&mut self) -> Result<Option<GraphEvent>> {
        panic!("StreamingDecompressor::next_event not implemented - use collect_all instead");
    }
    
    /// Collect all remaining events
    pub fn collect_all(self) -> Result<Vec<GraphEvent>> {
        // Create decoder from the reader  
        let mut decoder = zstd::Decoder::new(BufReader::new(self.reader))
            .map_err(|e| GraphError::SerializationError(e.to_string()))?;
        
        let mut events = Vec::new();
        loop {
            // Read length prefix
            let mut len_bytes = [0u8; 4];
            match decoder.read_exact(&mut len_bytes) {
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(GraphError::SerializationError(e.to_string())),
            }
            
            let len = u32::from_le_bytes(len_bytes) as usize;
            
            // Read event data
            let mut event_data = vec![0u8; len];
            decoder.read_exact(&mut event_data)
                .map_err(|e| GraphError::SerializationError(e.to_string()))?;
            
            let event: GraphEvent = bincode::deserialize(&event_data)
                .map_err(|e| GraphError::SerializationError(e.to_string()))?;
            
            events.push(event);
        }
        
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventPayload, IpldPayload};
    use uuid::Uuid;
    
    fn create_test_events(count: usize) -> Vec<GraphEvent> {
        let aggregate_id = Uuid::new_v4();
        (0..count).map(|i| GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id,
            correlation_id: Uuid::new_v4(),
            causation_id: None,
            payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                cid: format!("Qm{}", i),
                link_name: "test_link".to_string(),
                target_cid: format!("QmTarget{}", i),
            }),
        }).collect()
    }
    
    #[test]
    fn test_compression_roundtrip() {
        let events = create_test_events(100);
        let compressor = EventCompressor::default();
        
        let compressed = compressor.compress(&events).unwrap();
        assert!(compressed.compression_ratio() < 1.0); // Should compress
        
        let decompressed = compressed.decompress().unwrap();
        assert_eq!(decompressed.len(), events.len());
        
        for (orig, decomp) in events.iter().zip(decompressed.iter()) {
            assert_eq!(orig.event_id, decomp.event_id);
        }
    }
    
    #[test]
    fn test_streaming_compression() {
        use std::io::Cursor;

        let events = create_test_events(50);
        let mut buffer = Vec::new();

        // Compress events
        {
            let mut compressor = StreamingCompressor::new(&mut buffer, 3).unwrap();
            for event in &events {
                compressor.add_event(event).unwrap();
            }
            let count = compressor.finish().unwrap();
            assert_eq!(count, 50);
        }

        // Decompress events
        let cursor = Cursor::new(buffer);
        let buf_reader = BufReader::new(cursor);
        let decompressor = StreamingDecompressor::new(buf_reader).unwrap();
        let decompressed = decompressor.collect_all().unwrap();

        assert_eq!(decompressed.len(), events.len());
    }

    // ========== EventCompressor Tests ==========

    #[test]
    fn test_event_compressor_default() {
        let compressor = EventCompressor::default();
        let events = create_test_events(10);

        let compressed = compressor.compress(&events).unwrap();

        assert_eq!(compressed.event_count(), 10);
        assert!(compressed.compressed_size() > 0);
    }

    #[test]
    fn test_event_compressor_custom_level() {
        let compressor_low = EventCompressor::new(1);
        let compressor_high = EventCompressor::new(19);

        let events = create_test_events(100);

        let compressed_low = compressor_low.compress(&events).unwrap();
        let compressed_high = compressor_high.compress(&events).unwrap();

        // Both should decompress correctly
        let decompressed_low = compressed_low.decompress().unwrap();
        let decompressed_high = compressed_high.decompress().unwrap();

        assert_eq!(decompressed_low.len(), 100);
        assert_eq!(decompressed_high.len(), 100);

        // Higher compression level should typically produce smaller output
        // (though this may not always be true for small data)
    }

    #[test]
    fn test_event_compressor_level_clamping() {
        // Test that invalid levels are clamped
        let compressor_too_low = EventCompressor::new(-5);
        let compressor_too_high = EventCompressor::new(100);

        let events = create_test_events(5);

        // Should not panic, levels are clamped to valid range
        let compressed1 = compressor_too_low.compress(&events).unwrap();
        let compressed2 = compressor_too_high.compress(&events).unwrap();

        assert_eq!(compressed1.decompress().unwrap().len(), 5);
        assert_eq!(compressed2.decompress().unwrap().len(), 5);
    }

    #[test]
    fn test_compress_empty_events() {
        let compressor = EventCompressor::default();
        let events: Vec<GraphEvent> = vec![];

        let compressed = compressor.compress(&events).unwrap();

        assert_eq!(compressed.event_count(), 0);
        assert!(compressed.compressed_size() > 0); // Still has some overhead

        let decompressed = compressed.decompress().unwrap();
        assert!(decompressed.is_empty());
    }

    #[test]
    fn test_compress_single_event() {
        let compressor = EventCompressor::default();
        let events = create_test_events(1);

        let compressed = compressor.compress(&events).unwrap();

        assert_eq!(compressed.event_count(), 1);

        let decompressed = compressed.decompress().unwrap();
        assert_eq!(decompressed.len(), 1);
        assert_eq!(decompressed[0].event_id, events[0].event_id);
    }

    #[test]
    fn test_compress_large_batch() {
        let compressor = EventCompressor::default();
        let events = create_test_events(1000);

        let compressed = compressor.compress(&events).unwrap();

        assert_eq!(compressed.event_count(), 1000);

        let decompressed = compressed.decompress().unwrap();
        assert_eq!(decompressed.len(), 1000);

        // Verify first and last events
        assert_eq!(decompressed[0].event_id, events[0].event_id);
        assert_eq!(decompressed[999].event_id, events[999].event_id);
    }

    // ========== CompressedEventStream Tests ==========

    #[test]
    fn test_compressed_event_stream_accessors() {
        let compressor = EventCompressor::default();
        let events = create_test_events(25);

        let compressed = compressor.compress(&events).unwrap();

        assert_eq!(compressed.event_count(), 25);
        assert!(compressed.compressed_size() > 0);
        assert!(compressed.compression_ratio() > 0.0);
        assert!(compressed.compression_ratio() <= 1.5); // Usually < 1, but allow for small data
    }

    #[test]
    fn test_compression_ratio_calculation() {
        let compressor = EventCompressor::default();

        // Repetitive data compresses well
        let events = create_test_events(500);
        let compressed = compressor.compress(&events).unwrap();

        // Compression ratio should be less than 1 for compressible data
        assert!(compressed.compression_ratio() < 1.0,
            "Expected compression ratio < 1.0, got {}", compressed.compression_ratio());
    }

    // ========== Writer/Reader Tests ==========

    #[test]
    fn test_compress_to_writer() {
        use std::io::Cursor;

        let compressor = EventCompressor::default();
        let events = create_test_events(30);
        let mut buffer = Vec::new();

        compressor.compress_to(&events, &mut buffer).unwrap();

        assert!(!buffer.is_empty());

        // Decompress from the buffer
        let cursor = Cursor::new(buffer);
        let decompressed = EventCompressor::decompress_from(BufReader::new(cursor)).unwrap();

        assert_eq!(decompressed.len(), 30);
    }

    #[test]
    fn test_decompress_from_reader() {
        use std::io::Cursor;

        let compressor = EventCompressor::new(5);
        let events = create_test_events(20);

        // Compress
        let mut buffer = Vec::new();
        compressor.compress_to(&events, &mut buffer).unwrap();

        // Decompress
        let cursor = Cursor::new(buffer);
        let reader = BufReader::new(cursor);
        let decompressed = EventCompressor::decompress_from(reader).unwrap();

        assert_eq!(decompressed.len(), 20);

        for (orig, decomp) in events.iter().zip(decompressed.iter()) {
            assert_eq!(orig.event_id, decomp.event_id);
            assert_eq!(orig.aggregate_id, decomp.aggregate_id);
        }
    }

    // ========== StreamingCompressor Tests ==========

    #[test]
    fn test_streaming_compressor_creation() {
        use std::io::Cursor;

        let buffer = Cursor::new(Vec::new());
        let compressor = StreamingCompressor::new(buffer, 3).unwrap();

        // Debug format should work
        let debug_str = format!("{:?}", compressor);
        assert!(debug_str.contains("StreamingCompressor"));
        assert!(debug_str.contains("event_count"));
    }

    #[test]
    fn test_streaming_compressor_single_event() {
        use std::io::Cursor;

        let events = create_test_events(1);
        let mut buffer = Vec::new();

        {
            let mut compressor = StreamingCompressor::new(&mut buffer, 3).unwrap();
            compressor.add_event(&events[0]).unwrap();
            let count = compressor.finish().unwrap();
            assert_eq!(count, 1);
        }

        let cursor = Cursor::new(buffer);
        let decompressor = StreamingDecompressor::new(BufReader::new(cursor)).unwrap();
        let decompressed = decompressor.collect_all().unwrap();

        assert_eq!(decompressed.len(), 1);
    }

    #[test]
    fn test_streaming_compressor_incremental() {
        use std::io::Cursor;

        let events = create_test_events(10);
        let mut buffer = Vec::new();

        {
            let mut compressor = StreamingCompressor::new(&mut buffer, 3).unwrap();

            // Add events one at a time
            for (i, event) in events.iter().enumerate() {
                compressor.add_event(event).unwrap();
                // Event count should track correctly
                // (We can't easily check internal state, but finish will return count)
                let _ = i; // Just to use i
            }

            let final_count = compressor.finish().unwrap();
            assert_eq!(final_count, 10);
        }

        // Verify decompression
        let cursor = Cursor::new(buffer);
        let decompressor = StreamingDecompressor::new(BufReader::new(cursor)).unwrap();
        let decompressed = decompressor.collect_all().unwrap();

        assert_eq!(decompressed.len(), 10);
    }

    #[test]
    fn test_streaming_compressor_empty() {
        use std::io::Cursor;

        let mut buffer = Vec::new();

        {
            let compressor = StreamingCompressor::new(&mut buffer, 3).unwrap();
            let count = compressor.finish().unwrap();
            assert_eq!(count, 0);
        }

        // Empty stream should still decompress to empty
        let cursor = Cursor::new(buffer);
        let decompressor = StreamingDecompressor::new(BufReader::new(cursor)).unwrap();
        let decompressed = decompressor.collect_all().unwrap();

        assert!(decompressed.is_empty());
    }

    // ========== StreamingDecompressor Tests ==========

    #[test]
    fn test_streaming_decompressor_creation() {
        use std::io::Cursor;

        let cursor = Cursor::new(Vec::new());
        let decompressor = StreamingDecompressor::new(cursor).unwrap();

        let debug_str = format!("{:?}", decompressor);
        assert!(debug_str.contains("StreamingDecompressor"));
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn test_streaming_decompressor_next_event_panics() {
        use std::io::Cursor;

        let cursor = Cursor::new(Vec::new());
        let mut decompressor = StreamingDecompressor::new(cursor).unwrap();

        // This should panic as it's not implemented
        let _ = decompressor.next_event();
    }

    #[test]
    fn test_streaming_roundtrip_various_sizes() {
        use std::io::Cursor;

        for size in [1, 5, 10, 50, 100, 200] {
            let events = create_test_events(size);
            let mut buffer = Vec::new();

            {
                let mut compressor = StreamingCompressor::new(&mut buffer, 3).unwrap();
                for event in &events {
                    compressor.add_event(event).unwrap();
                }
                compressor.finish().unwrap();
            }

            let cursor = Cursor::new(buffer);
            let decompressor = StreamingDecompressor::new(BufReader::new(cursor)).unwrap();
            let decompressed = decompressor.collect_all().unwrap();

            assert_eq!(decompressed.len(), size,
                "Size mismatch for {} events", size);

            // Verify data integrity
            for (orig, decomp) in events.iter().zip(decompressed.iter()) {
                assert_eq!(orig.event_id, decomp.event_id);
            }
        }
    }

    // ========== Payload Variety Tests ==========

    #[test]
    fn test_compress_various_payloads() {
        let compressor = EventCompressor::default();
        let aggregate_id = Uuid::new_v4();

        // Note: Using payloads without serde_json::Value fields because bincode
        // doesn't support deserialize_any required by serde_json::Value
        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                    cid: "QmSource".to_string(),
                    link_name: "child".to_string(),
                    target_cid: "QmTarget".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: Some(Uuid::new_v4()),
                payload: EventPayload::Ipld(IpldPayload::CidPinned {
                    cid: "QmTest".to_string(),
                    recursive: true,
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidUnpinned {
                    cid: "QmUnpin".to_string(),
                }),
            },
        ];

        let compressed = compressor.compress(&events).unwrap();
        let decompressed = compressed.decompress().unwrap();

        assert_eq!(decompressed.len(), 3);

        // Verify each payload type
        match &decompressed[0].payload {
            EventPayload::Ipld(IpldPayload::CidLinkAdded { cid, link_name, target_cid }) => {
                assert_eq!(cid, "QmSource");
                assert_eq!(link_name, "child");
                assert_eq!(target_cid, "QmTarget");
            }
            _ => panic!("Wrong payload type for first event"),
        }

        match &decompressed[1].payload {
            EventPayload::Ipld(IpldPayload::CidPinned { recursive, .. }) => {
                assert!(*recursive);
            }
            _ => panic!("Wrong payload type for second event"),
        }

        match &decompressed[2].payload {
            EventPayload::Ipld(IpldPayload::CidUnpinned { cid }) => {
                assert_eq!(cid, "QmUnpin");
            }
            _ => panic!("Wrong payload type for third event"),
        }
    }

    #[test]
    fn test_compress_workflow_payloads() {
        use crate::events::WorkflowPayload;

        let compressor = EventCompressor::default();
        let aggregate_id = Uuid::new_v4();
        let workflow_id = Uuid::new_v4();

        let events = vec![
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::WorkflowDefined {
                    workflow_id,
                    name: "TestWorkflow".to_string(),
                    version: "1.0.0".to_string(),
                }),
            },
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id,
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Workflow(WorkflowPayload::StateAdded {
                    workflow_id,
                    state_id: "start".to_string(),
                    state_type: "initial".to_string(),
                }),
            },
        ];

        let compressed = compressor.compress(&events).unwrap();
        let decompressed = compressed.decompress().unwrap();

        assert_eq!(decompressed.len(), 2);
    }

    // ========== Error Handling Tests ==========

    #[test]
    fn test_decompress_invalid_data() {
        let invalid_data = vec![0u8, 1, 2, 3, 4, 5]; // Not valid compressed data

        let stream = CompressedEventStream {
            data: invalid_data,
            event_count: 0,
            compression_ratio: 1.0,
        };

        let result = stream.decompress();
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_from_invalid_reader() {
        use std::io::Cursor;

        let invalid_data = vec![0u8, 1, 2, 3];
        let cursor = Cursor::new(invalid_data);
        let reader = BufReader::new(cursor);

        let result = EventCompressor::decompress_from(reader);
        assert!(result.is_err());
    }

    // ========== Edge Cases ==========

    #[test]
    fn test_compress_events_with_all_fields() {
        let compressor = EventCompressor::default();

        let event = GraphEvent {
            event_id: Uuid::new_v4(),
            aggregate_id: Uuid::new_v4(),
            correlation_id: Uuid::new_v4(),
            causation_id: Some(Uuid::new_v4()), // Has causation_id
            payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                cid: "QmSource".to_string(),
                link_name: "child".to_string(),
                target_cid: "QmTarget".to_string(),
            }),
        };

        let compressed = compressor.compress(&[event.clone()]).unwrap();
        let decompressed = compressed.decompress().unwrap();

        assert_eq!(decompressed[0].event_id, event.event_id);
        assert_eq!(decompressed[0].causation_id, event.causation_id);
    }

    #[test]
    fn test_compress_events_with_large_data() {
        let compressor = EventCompressor::default();

        // Create many events with repetitive data (large payload with bincode-compatible types)
        // Note: We can't use serde_json::Value with bincode, so we use repetitive string data
        let large_cid = "Qm".to_string() + &"x".repeat(1000);
        let large_link_name = "link_".to_string() + &"a".repeat(500);

        let events: Vec<GraphEvent> = (0..100).map(|i| {
            GraphEvent {
                event_id: Uuid::new_v4(),
                aggregate_id: Uuid::new_v4(),
                correlation_id: Uuid::new_v4(),
                causation_id: None,
                payload: EventPayload::Ipld(IpldPayload::CidLinkAdded {
                    cid: format!("{}{}", large_cid, i),
                    link_name: large_link_name.clone(),
                    target_cid: format!("QmTarget{}{}", "y".repeat(100), i),
                }),
            }
        }).collect();

        let compressed = compressor.compress(&events).unwrap();

        // Large repetitive data should compress well
        assert!(compressed.compression_ratio() < 1.0,
            "Expected compression ratio < 1.0, got {}", compressed.compression_ratio());

        let decompressed = compressed.decompress().unwrap();

        assert_eq!(decompressed.len(), 100);

        // Verify first and last event data integrity
        match &decompressed[0].payload {
            EventPayload::Ipld(IpldPayload::CidLinkAdded { cid, link_name, .. }) => {
                assert!(cid.starts_with("Qm"));
                assert!(link_name.starts_with("link_"));
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_multiple_compression_levels() {
        let events = create_test_events(50);

        // Test various compression levels
        for level in [1, 3, 5, 10, 15, 19, 22] {
            let compressor = EventCompressor::new(level);
            let compressed = compressor.compress(&events).unwrap();
            let decompressed = compressed.decompress().unwrap();

            assert_eq!(decompressed.len(), 50,
                "Level {} failed to decompress correctly", level);
        }
    }
}