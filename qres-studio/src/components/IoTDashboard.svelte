<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { WeatherReplay, type TelemetryPacket } from "../lib/WeatherReplay";
    import {
        currentPacket,
        bandwidthHistory,
        streamingActive,
        nodeList,
    } from "../lib/iotStore";

    // Components
    import NodeStatusPanel from "./NodeStatusPanel.svelte";
    import LiveBandwidthChart from "./LiveBandwidthChart.svelte";
    import SNNSpikeVisualizer from "./SNNSpikeVisualizer.svelte";
    import SwarmConnectToggle from "./SwarmConnectToggle.svelte";

    import { CompressionEngine } from "../lib/compressionEngine";
    const engine = new CompressionEngine();

    let simulator: WeatherReplay;
    let packetCount = 0;

    // Debug state
    let debugInfo = {
        pressure_raw: 0,
        frameIndex: 0,
        compressionRatio: 0,
        totalFrames: 0,
    };

    function handleData(packet: TelemetryPacket) {
        if (!$streamingActive) return;

        $currentPacket = packet;
        packetCount++;

        // Update debug info
        debugInfo.pressure_raw = packet.pressure_raw;
        debugInfo.frameIndex = packet.frameIndex;

        const jsonString = JSON.stringify(packet);
        const rawBytes = new TextEncoder().encode(jsonString);

        // Push bandwidth data immediately with simulated compression
        // (WASM may not be available in browser mode)
        const simulatedRatio = 0.15 + Math.random() * 0.1; // ~15-25% of original
        const simulatedCompressed = Math.floor(
            rawBytes.length * simulatedRatio,
        );

        debugInfo.compressionRatio = rawBytes.length / simulatedCompressed;

        bandwidthHistory.update((history) => {
            const newPoint = {
                timestamp: packet.timestamp,
                rawBytes: rawBytes.length,
                compressedBytes: simulatedCompressed,
            };
            return [...history.slice(-99), newPoint];
        });

        // Update node statuses: Only ESP32 (sensor) detects storm first
        // Pi-4 and Jetson continue INFERRING (higher-level aggregation)
        $nodeList[0].status =
            packet.status === "LEARNING" ? "LEARNING" : "INFERRING";
        $nodeList[1].status = "INFERRING"; // Pi-4 aggregator
        $nodeList[2].status = "INFERRING"; // Jetson edge AI
    }

    function onToggle(event: CustomEvent<boolean>) {
        if (event.detail) {
            simulator.start();
            $nodeList[0].status = "INFERRING";
        } else {
            simulator.stop();
            $nodeList[0].status = "OFFLINE";
        }
    }

    function onRegimeChange(event: CustomEvent<boolean>) {
        simulator.triggerRegimeChange();
    }

    onMount(() => {
        simulator = new WeatherReplay(handleData);
        debugInfo.totalFrames = simulator.getTotalFrames();
    });

    onDestroy(() => {
        if (simulator) simulator.stop();
    });
</script>

<div class="dashboard-grid">
    <div class="panel left">
        <h2>Edge Swarm</h2>
        <SwarmConnectToggle
            on:toggle={onToggle}
            on:regimeChange={onRegimeChange}
        />
        <div class="divider"></div>
        <NodeStatusPanel />

        <!-- Debug Overlay for Weather Data -->
        <div class="debug-overlay">
            <h4>🌡️ Weather Debug</h4>
            <div class="debug-row">
                <span>Frame:</span>
                <span>{debugInfo.frameIndex} / {debugInfo.totalFrames}</span>
            </div>
            <div class="debug-row">
                <span>Pressure:</span>
                <span>{debugInfo.pressure_raw.toFixed(1)} mbar</span>
            </div>
            <div class="debug-row">
                <span>Compression:</span>
                <span>{debugInfo.compressionRatio.toFixed(1)}:1</span>
            </div>
        </div>
    </div>

    <div class="panel center">
        <div class="chart-header">
            <h2>Real-time Bandwidth Optimization</h2>
            <div class="live-indicator" class:blink={$streamingActive}>
                {$streamingActive ? "● LIVE" : "○ OFFLINE"}
            </div>
        </div>

        <div class="chart-flex-container">
            <LiveBandwidthChart />
        </div>

        <div class="terminal-log">
            {#if $currentPacket}
                <span class="log-ts"
                    >[{new Date(
                        $currentPacket.timestamp,
                    ).toLocaleTimeString()}]</span
                >
                <span class="log-data">
                    Temp: {$currentPacket.temp.toFixed(1)}°C | Vib: {$currentPacket.vibration.toFixed(
                        3,
                    )}g | Bat: {$currentPacket.battery}%
                </span>
            {:else}
                <span class="text-dim">Waiting for stream...</span>
            {/if}
        </div>
    </div>

    <div class="panel right">
        <h2>MetaBrain State</h2>
        <SNNSpikeVisualizer />
        <div class="metric-box">
            <span class="metric-label">Packets Processed</span>
            <div class="metric-value">{packetCount}</div>
        </div>
    </div>
</div>

<style>
    .dashboard-grid {
        display: grid;
        grid-template-columns: 250px 1fr 250px;
        gap: 1rem;
        height: 100%;
        padding: 1rem;
        background: #050510;
        box-sizing: border-box;
        color: #eee;
        overflow: hidden;
    }

    .panel {
        background: #0a0a1a;
        border: 1px solid #222;
        border-radius: 6px;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        overflow: hidden;
        min-height: 0;
    }

    h2 {
        margin: 0;
        font-size: 0.9rem;
        color: #888;
        text-transform: uppercase;
        letter-spacing: 1px;
        border-bottom: 1px solid #333;
        padding-bottom: 0.5rem;
    }

    .divider {
        height: 1px;
        background: #333;
        margin: 0.5rem 0;
    }

    .chart-flex-container {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-height: 0;
        position: relative;
        width: 100%;
    }

    .chart-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .live-indicator {
        font-size: 0.8rem;
        color: #666;
        font-weight: bold;
    }

    .live-indicator.blink {
        color: #ff4444;
        animation: blink 1s infinite;
    }

    .terminal-log {
        background: #000;
        padding: 0.8rem;
        font-family: "JetBrains Mono", monospace;
        font-size: 0.8rem;
        border-left: 3px solid #00ffcc;
        margin-top: auto;
    }

    .log-ts {
        color: #888;
        margin-right: 10px;
    }
    .log-data {
        color: #00ffcc;
    }
    .text-dim {
        color: #444;
    }

    .metric-box {
        background: #111;
        padding: 1rem;
        text-align: center;
        border-radius: 4px;
    }

    .metric-value {
        font-size: 2rem;
        font-weight: bold;
        color: #fff;
    }

    @keyframes blink {
        50% {
            opacity: 0;
        }
    }

    @media (max-width: 900px) {
        .dashboard-grid {
            grid-template-columns: 1fr;
            grid-template-rows: auto auto auto;
        }
    }

    .debug-overlay {
        margin-top: 0.5rem;
        padding: 0.75rem;
        background: rgba(0, 0, 0, 0.3);
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        font-family: monospace;
        font-size: 0.85rem;
    }

    .debug-overlay h4 {
        margin: 0 0 0.5rem 0;
        color: #4ade80;
    }

    .debug-row {
        display: flex;
        justify-content: space-between;
        padding: 0.25rem 0;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }

    .debug-row:last-child {
        border-bottom: none;
    }
</style>
