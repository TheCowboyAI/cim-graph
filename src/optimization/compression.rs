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
}