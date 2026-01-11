<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import {
        SensorSimulator,
        type TelemetryPacket,
    } from "../lib/SensorSimulator";
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

    // Use the existing compression engine (assuming it's available)
    import { CompressionEngine } from "../lib/compressionEngine";
    const engine = new CompressionEngine();

    let simulator: SensorSimulator;
    let packetCount = 0;

    function handleData(packet: TelemetryPacket) {
        if (!$streamingActive) return;

        // 1. Update UI with latest packet
        $currentPacket = packet;
        packetCount++;

        // 2. Compress the data
        const jsonString = JSON.stringify(packet);
        const rawBytes = new TextEncoder().encode(jsonString);

        // Simulate async compression
        engine.compress(rawBytes, true).then((result) => {
            const compressedSize = result.data.length;

            // 3. Update Chart History
            bandwidthHistory.update((history) => {
                const newPoint = {
                    timestamp: packet.timestamp,
                    rawBytes: rawBytes.length,
                    compressedBytes: compressedSize,
                };
                // Keep last 100 points
                return [...history.slice(-99), newPoint];
            });

            // 4. Update Node Status if Regime Change
            if (packet.status === "LEARNING") {
                $nodeList[0].status = "LEARNING";
            } else {
                $nodeList[0].status = "INFERRING";
            }
        });
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
        simulator = new SensorSimulator(handleData);
        // Initialize WASM
        // engine.initWasm(); // Assuming this exists or is auto-called
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
    </div>

    <div class="panel center">
        <div class="chart-header">
            <h2>Real-time Bandwidth Optimization</h2>
            <div class="live-indicator" class:blink={$streamingActive}>
                {$streamingActive ? "● LIVE" : "○ OFFLINE"}
            </div>
        </div>
        <LiveBandwidthChart />

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
        color: #eee;
    }

    .panel {
        background: #0a0a1a;
        border: 1px solid #222;
        border-radius: 6px;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
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
</style>
