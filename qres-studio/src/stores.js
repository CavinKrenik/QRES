import { writable, derived } from 'svelte/store';

// Quantum State Store - for persistent world states
export const quantumState = writable({
    fidelity: 1.0,
    graph: { nodes: [], edges: [] },
    version: null,
    timestamp: null
});

// Swarm Status Store - for P2P network status
export const swarmStatus = writable({
    status: 'Idle',
    peers: 0,
    lastBroadcast: null,
    lastReceived: null
});

// Compression Stats Store - for real-time metrics
export const compressionStats = writable({
    mode: 'standard',
    ratio: 0,
    originalSize: 0,
    compressedSize: 0,
    timestamp: null as number | null
});

// Neural Optimization Store - for AQC/RL metrics
export const neuralStats = writable({
    sparsity: 0,
    confidence: [],
    activeEngine: null
});

// Derived store for overall system health
export const systemHealth = derived(
    [quantumState, swarmStatus, compressionStats],
    ([$quantum, $swarm, $compression]) => ({
        quantumFidelity: $quantum.fidelity,
        networkConnected: $swarm.peers > 0,
        compressionActive: $compression.ratio > 0,
        overallStatus: $quantum.fidelity > 0.98 && $swarm.peers >= 0 ? 'Healthy' : 'Warning'
    })
);

// API endpoint configuration
export const apiConfig = writable({
    baseUrl: 'http://localhost:8000',
    timeout: 30000
});
