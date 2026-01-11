import { writable, derived } from 'svelte/store';
import type { TelemetryPacket } from './SensorSimulator';

// --- State Definitions ---

export interface BandwidthPoint {
    timestamp: number;
    rawBytes: number;
    compressedBytes: number;
}

export interface NodeStatus {
    id: string;
    name: string;
    type: string;
    status: 'OFFLINE' | 'LEARNING' | 'INFERRING';
    lastSeen: number;
}

// --- Stores ---

// Toggle for the simulation
export const streamingActive = writable<boolean>(false);

// Live incoming sensor data (keep last 1)
export const currentPacket = writable<TelemetryPacket | null>(null);

// History for the line chart (keep last N points)
export const bandwidthHistory = writable<BandwidthPoint[]>([]);

// List of active nodes in the swarm
export const nodeList = writable<NodeStatus[]>([
    { id: 'n1', name: 'ESP32-01', type: 'Sensor', status: 'OFFLINE', lastSeen: 0 },
    { id: 'n2', name: 'Pi-4-Cluster', type: 'Aggregator', status: 'OFFLINE', lastSeen: 0 },
    { id: 'n3', name: 'Jetson-Nano', type: 'Edge AI', status: 'OFFLINE', lastSeen: 0 }
]);

// Derived stats
export const compressionStats = derived(bandwidthHistory, ($history) => {
    if ($history.length === 0) return { ratio: 0, savings: 0, totalRaw: 0 };

    // Calculate stats over the visible history window
    let totalRaw = 0;
    let totalComp = 0;

    $history.forEach(p => {
        totalRaw += p.rawBytes;
        totalComp += p.compressedBytes;
    });

    const savings = totalRaw > 0 ? ((totalRaw - totalComp) / totalRaw) * 100 : 0;
    const ratio = totalComp > 0 ? (totalRaw / totalComp) : 0;

    return {
        ratio: ratio.toFixed(1),
        savings: savings.toFixed(1),
        totalRaw
    };
});
