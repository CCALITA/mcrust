use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use mc_core::pos::ChunkPos;

use crate::caves::CaveCarver;
use crate::chunk::Chunk;
use crate::end::EndTerrainGen;
use crate::nether::NetherTerrainGen;
use crate::noise_terrain::NoiseTerrainGen;
use crate::ores::OreGenerator;
use crate::trees;

/// Thread-safe configuration for chunk generation.
///
/// Holds only the seed so it can be shared across worker threads via `Arc`.
/// Each worker creates its own generators from this config, avoiding the need
/// to make the noise generators `Send` or `Sync`.
#[derive(Debug, Clone)]
pub struct ChunkGenConfig {
    pub seed: u64,
}

/// Request to generate a chunk at a given position and dimension.
#[derive(Debug, Clone)]
pub struct ChunkLoadRequest {
    pub pos: ChunkPos,
    pub dimension: u8,
    pub priority: f32,
}

/// Sentinel value sent through the request channel to signal a worker to shut down.
enum WorkerMessage {
    Request(ChunkLoadRequest),
    Shutdown,
}

/// Generates a chunk using fresh, per-call generators.
///
/// This function is the standalone generation pipeline that worker threads call.
/// It creates new `NoiseTerrainGen`, `CaveCarver`, `OreGenerator`, and dimension
/// generators each time, so none of those types need to be `Send` or `Sync`.
pub fn generate_chunk_standalone(config: &ChunkGenConfig, pos: ChunkPos, dimension: u8) -> Chunk {
    match dimension {
        0 => {
            // Overworld
            let terrain_gen = NoiseTerrainGen::new(config.seed);
            let cave_carver = CaveCarver::new(config.seed);
            let ore_gen = OreGenerator::new(config.seed);

            let mut chunk = terrain_gen.generate(pos.x, pos.z);
            cave_carver.carve(&mut chunk, pos.x, pos.z);
            ore_gen.generate_ores(&mut chunk, pos.x, pos.z);
            trees::place_trees(&mut chunk, pos.x, pos.z, config.seed);
            trees::place_vegetation(&mut chunk, pos.x, pos.z, config.seed);
            chunk
        }
        1 => {
            // Nether
            let nether_gen = NetherTerrainGen::new(config.seed);
            nether_gen.generate(pos.x, pos.z)
        }
        2 => {
            // End
            let end_gen = EndTerrainGen::new(config.seed);
            end_gen.generate(pos.x, pos.z)
        }
        _ => {
            log::warn!(
                "Unknown dimension {} for chunk ({}, {}), generating empty chunk",
                dimension,
                pos.x,
                pos.z
            );
            Chunk::new()
        }
    }
}

/// Asynchronous chunk loader that distributes generation work across worker threads.
///
/// Uses `std::sync::mpsc` channels for communication:
/// - Each worker has its own request sender (the loader keeps a clone)
/// - All workers share a single response sender that feeds into one receiver
///
/// Workers create fresh noise generators per chunk (or cache them per-thread)
/// since the existing generators are not `Send`/`Sync`.
pub struct AsyncChunkLoader {
    /// Senders for dispatching requests to individual worker threads.
    request_senders: Vec<Sender<WorkerMessage>>,
    /// Receiver for completed chunks from all workers.
    response_receiver: Receiver<(ChunkPos, Chunk)>,
    /// Worker thread handles for joining on shutdown.
    workers: Vec<JoinHandle<()>>,
    /// Round-robin index for distributing work across workers.
    next_worker: usize,
    /// Count of pending (in-flight) chunk requests.
    pending: Arc<AtomicUsize>,
    /// Flag indicating whether shutdown has been initiated.
    shutdown_flag: Arc<AtomicBool>,
}

impl AsyncChunkLoader {
    /// Creates a new `AsyncChunkLoader` with the specified number of worker threads.
    ///
    /// Each worker thread receives chunk generation requests and sends back
    /// completed `(ChunkPos, Chunk)` pairs through a shared response channel.
    pub fn new(num_threads: usize, seed: u64) -> io::Result<Self> {
        let config = Arc::new(ChunkGenConfig { seed });
        let (response_tx, response_rx) = mpsc::channel();
        let pending = Arc::new(AtomicUsize::new(0));
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let mut request_senders = Vec::with_capacity(num_threads);
        let mut workers = Vec::with_capacity(num_threads);

        for worker_id in 0..num_threads {
            let (req_tx, req_rx) = mpsc::channel::<WorkerMessage>();
            let resp_tx = response_tx.clone();
            let cfg = Arc::clone(&config);
            let pending_count = Arc::clone(&pending);

            let handle = thread::Builder::new()
                .name(format!("chunk-worker-{worker_id}"))
                .spawn(move || {
                    Self::worker_loop(req_rx, resp_tx, &cfg, &pending_count);
                })?;

            request_senders.push(req_tx);
            workers.push(handle);
        }

        Ok(Self {
            request_senders,
            response_receiver: response_rx,
            workers,
            next_worker: 0,
            pending,
            shutdown_flag,
        })
    }

    /// The main loop for a worker thread.
    ///
    /// Blocks on receiving requests, generates chunks, and sends results back.
    /// Exits when it receives a `Shutdown` message or the request channel is closed.
    fn worker_loop(
        req_rx: Receiver<WorkerMessage>,
        resp_tx: Sender<(ChunkPos, Chunk)>,
        config: &ChunkGenConfig,
        pending: &AtomicUsize,
    ) {
        while let Ok(message) = req_rx.recv() {
            match message {
                WorkerMessage::Request(request) => {
                    let chunk = generate_chunk_standalone(config, request.pos, request.dimension);

                    // If the response channel is closed, the loader has been dropped;
                    // decrement pending and exit gracefully.
                    if resp_tx.send((request.pos, chunk)).is_err() {
                        pending.fetch_sub(1, Ordering::SeqCst);
                        break;
                    }
                    pending.fetch_sub(1, Ordering::SeqCst);
                }
                WorkerMessage::Shutdown => break,
            }
        }
    }

    /// Sends a chunk generation request to the next worker (round-robin).
    pub fn request_chunk(&mut self, pos: ChunkPos, dimension: u8, priority: f32) {
        if self.shutdown_flag.load(Ordering::SeqCst) {
            log::warn!("Cannot request chunks after shutdown has been initiated");
            return;
        }

        let request = ChunkLoadRequest {
            pos,
            dimension,
            priority,
        };

        self.pending.fetch_add(1, Ordering::SeqCst);

        let worker_index = self.next_worker % self.request_senders.len();
        self.next_worker = self.next_worker.wrapping_add(1);

        if self.request_senders[worker_index]
            .send(WorkerMessage::Request(request))
            .is_err()
        {
            log::error!("Failed to send chunk request to worker {worker_index}");
            self.pending.fetch_sub(1, Ordering::SeqCst);
        }
    }

    /// Non-blocking poll for all ready (completed) chunks.
    ///
    /// Returns a `Vec` of `(ChunkPos, Chunk)` pairs that have been generated
    /// since the last call. Returns an empty vec if nothing is ready.
    pub fn poll_ready(&self) -> Vec<(ChunkPos, Chunk)> {
        let mut results = Vec::new();
        while let Ok(result) = self.response_receiver.try_recv() {
            results.push(result);
        }
        results
    }

    /// Returns the number of chunk requests that are currently in-flight.
    pub fn pending_count(&self) -> usize {
        self.pending.load(Ordering::SeqCst)
    }

    /// Shuts down all worker threads gracefully.
    ///
    /// Sends a termination signal to each worker and joins all threads.
    pub fn shutdown(self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // Send shutdown signal to each worker.
        for sender in &self.request_senders {
            let _ = sender.send(WorkerMessage::Shutdown);
        }

        // Drop the senders so workers see channel closure if they missed the signal.
        drop(self.request_senders);

        // Join all worker threads.
        for handle in self.workers {
            if let Err(e) = handle.join() {
                log::error!("Chunk worker thread panicked: {e:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_core::block::BlockId;

    #[test]
    fn standalone_generates_overworld_chunk() {
        let config = ChunkGenConfig { seed: 42 };
        let pos = ChunkPos::new(0, 0);
        let chunk = generate_chunk_standalone(&config, pos, 0);

        // Overworld always has bedrock at y=-64
        assert_eq!(
            chunk.get_block(0, -64, 0),
            BlockId::Bedrock,
            "expected bedrock at y=-64 in overworld chunk"
        );

        // High altitude should be air
        assert_eq!(
            chunk.get_block(0, 200, 0),
            BlockId::Air,
            "expected air at y=200 in overworld chunk"
        );
    }

    #[test]
    fn standalone_generates_nether_chunk() {
        let config = ChunkGenConfig { seed: 42 };
        let pos = ChunkPos::new(0, 0);
        let chunk = generate_chunk_standalone(&config, pos, 1);

        // Nether has bedrock at y=0
        assert_eq!(
            chunk.get_block(0, 0, 0),
            BlockId::Bedrock,
            "expected bedrock at y=0 in nether chunk"
        );
    }

    #[test]
    fn standalone_generates_end_chunk() {
        let config = ChunkGenConfig { seed: 42 };
        let pos = ChunkPos::new(0, 0);
        let chunk = generate_chunk_standalone(&config, pos, 2);

        // End has EndStone at y=64 near origin
        assert_eq!(
            chunk.get_block(0, 64, 0),
            BlockId::EndStone,
            "expected EndStone at y=64 in end chunk near origin"
        );
    }

    #[test]
    fn standalone_unknown_dimension_returns_empty() {
        let config = ChunkGenConfig { seed: 42 };
        let pos = ChunkPos::new(0, 0);
        let chunk = generate_chunk_standalone(&config, pos, 255);

        // Unknown dimension should produce an empty (all-air) chunk
        assert_eq!(
            chunk.get_block(0, 64, 0),
            BlockId::Air,
            "expected air in unknown dimension chunk"
        );
    }

    #[test]
    fn request_and_receive_single_chunk() {
        let mut loader = AsyncChunkLoader::new(2, 42).unwrap();
        let pos = ChunkPos::new(0, 0);
        loader.request_chunk(pos, 0, 1.0);

        // Wait for the chunk to be generated (with timeout)
        let mut results = Vec::new();
        let start = std::time::Instant::now();
        while results.is_empty() && start.elapsed() < std::time::Duration::from_secs(30) {
            results = loader.poll_ready();
            if results.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        assert_eq!(results.len(), 1, "expected exactly one chunk result");
        let (result_pos, chunk) = &results[0];
        assert_eq!(*result_pos, pos, "returned chunk has wrong position");

        // Verify the chunk has valid overworld content
        assert_eq!(
            chunk.get_block(0, -64, 0),
            BlockId::Bedrock,
            "expected bedrock at y=-64"
        );

        loader.shutdown();
    }

    #[test]
    fn multiple_requests_complete() {
        let mut loader = AsyncChunkLoader::new(2, 42).unwrap();
        let positions = vec![
            ChunkPos::new(0, 0),
            ChunkPos::new(1, 0),
            ChunkPos::new(0, 1),
            ChunkPos::new(-1, -1),
        ];

        for &pos in &positions {
            loader.request_chunk(pos, 0, 1.0);
        }

        // Collect all results with timeout
        let mut results = Vec::new();
        let start = std::time::Instant::now();
        while results.len() < positions.len()
            && start.elapsed() < std::time::Duration::from_secs(60)
        {
            results.extend(loader.poll_ready());
            if results.len() < positions.len() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        assert_eq!(
            results.len(),
            positions.len(),
            "expected {} chunk results, got {}",
            positions.len(),
            results.len()
        );

        // Verify all requested positions are present in results
        let result_positions: std::collections::HashSet<ChunkPos> =
            results.iter().map(|(pos, _)| *pos).collect();
        for pos in &positions {
            assert!(
                result_positions.contains(pos),
                "missing result for chunk ({}, {})",
                pos.x,
                pos.z
            );
        }

        loader.shutdown();
    }

    #[test]
    fn pending_count_tracks_inflight_requests() {
        let mut loader = AsyncChunkLoader::new(1, 42).unwrap();

        assert_eq!(loader.pending_count(), 0, "initial pending should be 0");

        // Request several chunks
        for i in 0..4 {
            loader.request_chunk(ChunkPos::new(i, 0), 0, 1.0);
        }

        // Pending should be > 0 immediately after requesting (before any complete)
        // Note: some may have already completed, so we just check it was non-zero
        // at some point by verifying total results + pending equals total requested.
        let mut total_received = 0;
        let start = std::time::Instant::now();
        while total_received < 4 && start.elapsed() < std::time::Duration::from_secs(60) {
            let ready = loader.poll_ready();
            total_received += ready.len();
            if total_received < 4 {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        assert_eq!(total_received, 4, "expected all 4 chunks to complete");
        assert_eq!(
            loader.pending_count(),
            0,
            "pending should be 0 after all complete"
        );

        loader.shutdown();
    }

    #[test]
    fn shutdown_is_clean() {
        let mut loader = AsyncChunkLoader::new(4, 42).unwrap();

        // Request a few chunks
        for i in 0..3 {
            loader.request_chunk(ChunkPos::new(i, 0), 0, 1.0);
        }

        // Shutdown should not panic or hang
        loader.shutdown();
    }

    #[test]
    fn single_thread_works() {
        let mut loader = AsyncChunkLoader::new(1, 42).unwrap();
        let pos = ChunkPos::new(5, 5);
        loader.request_chunk(pos, 0, 1.0);

        let mut results = Vec::new();
        let start = std::time::Instant::now();
        while results.is_empty() && start.elapsed() < std::time::Duration::from_secs(30) {
            results = loader.poll_ready();
            if results.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, pos);

        loader.shutdown();
    }

    #[test]
    fn nether_chunk_via_async() {
        let mut loader = AsyncChunkLoader::new(1, 42).unwrap();
        let pos = ChunkPos::new(0, 0);
        loader.request_chunk(pos, 1, 1.0);

        let mut results = Vec::new();
        let start = std::time::Instant::now();
        while results.is_empty() && start.elapsed() < std::time::Duration::from_secs(30) {
            results = loader.poll_ready();
            if results.is_empty() {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }

        assert_eq!(results.len(), 1);
        let (_, chunk) = &results[0];
        // Nether bedrock at y=0
        assert_eq!(chunk.get_block(0, 0, 0), BlockId::Bedrock);

        loader.shutdown();
    }
}
