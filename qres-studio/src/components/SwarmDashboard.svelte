<script>
    // @ts-nocheck
    import { swarmStatus, quantumState } from "../stores.js";
    import { onMount, onDestroy } from "svelte";

    let broadcastHistory = [];
    let receiveHistory = [];
    let updateInterval = null;

    // Simulate real-time updates (replace with actual WebSocket/polling)
    onMount(() => {
        updateInterval = setInterval(() => {
            // Mock peer discovery
            swarmStatus.update((status) => ({
                ...status,
                peers: Math.floor(Math.random() * 5),
            }));
        }, 5000);
    });

    onDestroy(() => {
        if (updateInterval) clearInterval(updateInterval);
    });

    $: statusColor = $swarmStatus.peers > 0 ? "#4ade80" : "#f87171";
    $: statusText = $swarmStatus.peers > 0 ? "Connected" : "Disconnected";
</script>

<div class="swarm-dashboard">
    <h3>🌐 P2P Swarm Dashboard</h3>

    <div class="status-grid">
        <div class="status-card">
            <div
                class="status-icon"
                style="background-color: {statusColor}"
            ></div>
            <div class="status-info">
                <span class="status-label">Network Status</span>
                <span class="status-value">{statusText}</span>
            </div>
        </div>

        <div class="status-card">
            <div class="status-icon" style="background-color: #60a5fa"></div>
            <div class="status-info">
                <span class="status-label">Connected Peers</span>
                <span class="status-value">{$swarmStatus.peers}</span>
            </div>
        </div>

        <div class="status-card">
            <div class="status-icon" style="background-color: #a78bfa"></div>
            <div class="status-info">
                <span class="status-label">Quantum Fidelity</span>
                <span class="status-value"
                    >{($quantumState.fidelity * 100).toFixed(2)}%</span
                >
            </div>
        </div>

        <div class="status-card">
            <div class="status-icon" style="background-color: #f472b6"></div>
            <div class="status-info">
                <span class="status-label">World Version</span>
                <span class="status-value"
                    >{$quantumState.version || "None"}</span
                >
            </div>
        </div>
    </div>

    <div class="history-section">
        <h4>📤 Recent Broadcasts</h4>
        <div class="history-list">
            {#if $swarmStatus.lastBroadcast}
                <div class="history-item">
                    <span class="timestamp"
                        >{new Date(
                            $swarmStatus.lastBroadcast,
                        ).toLocaleTimeString()}</span
                    >
                    <span class="message"
                        >World state broadcast to {$swarmStatus.peers} peers</span
                    >
                </div>
            {:else}
                <div class="empty-state">No broadcasts yet</div>
            {/if}
        </div>
    </div>

    <div class="history-section">
        <h4>📥 Recent Receives</h4>
        <div class="history-list">
            {#if $swarmStatus.lastReceived}
                <div class="history-item">
                    <span class="timestamp"
                        >{new Date(
                            $swarmStatus.lastReceived,
                        ).toLocaleTimeString()}</span
                    >
                    <span class="message"
                        >Quantum tensor received and merged</span
                    >
                </div>
            {:else}
                <div class="empty-state">No receives yet</div>
            {/if}
        </div>
    </div>
</div>

<style>
    .swarm-dashboard {
        background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
        border: 1px solid #0f3460;
        border-radius: 12px;
        padding: 24px;
        margin: 16px 0;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3);
    }

    h3 {
        color: #e94560;
        margin: 0 0 20px 0;
        font-size: 1.4em;
        font-weight: 600;
    }

    h4 {
        color: #a8dadc;
        margin: 20px 0 12px 0;
        font-size: 1.1em;
        font-weight: 500;
    }

    .status-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 16px;
        margin-bottom: 24px;
    }

    .status-card {
        background: rgba(15, 52, 96, 0.5);
        border: 1px solid #1a5490;
        border-radius: 8px;
        padding: 16px;
        display: flex;
        align-items: center;
        gap: 12px;
        transition: all 0.3s ease;
    }

    .status-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 4px 8px rgba(233, 69, 96, 0.2);
        border-color: #e94560;
    }

    .status-icon {
        width: 12px;
        height: 12px;
        border-radius: 50%;
        box-shadow: 0 0 10px currentColor;
        animation: pulse 2s ease-in-out infinite;
    }

    @keyframes pulse {
        0%,
        100% {
            opacity: 1;
        }
        50% {
            opacity: 0.5;
        }
    }

    .status-info {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .status-label {
        color: #a8dadc;
        font-size: 0.85em;
        opacity: 0.8;
    }

    .status-value {
        color: #f1faee;
        font-size: 1.1em;
        font-weight: 600;
    }

    .history-section {
        margin-top: 20px;
    }

    .history-list {
        background: rgba(15, 52, 96, 0.3);
        border: 1px solid #1a5490;
        border-radius: 8px;
        padding: 12px;
        max-height: 150px;
        overflow-y: auto;
    }

    .history-item {
        display: flex;
        gap: 12px;
        padding: 8px;
        border-bottom: 1px solid rgba(26, 84, 144, 0.3);
    }

    .history-item:last-child {
        border-bottom: none;
    }

    .timestamp {
        color: #60a5fa;
        font-size: 0.85em;
        font-family: "Courier New", monospace;
        min-width: 80px;
    }

    .message {
        color: #a8dadc;
        font-size: 0.9em;
    }

    .empty-state {
        color: #6b7280;
        font-style: italic;
        text-align: center;
        padding: 20px;
    }

    @media (max-width: 768px) {
        .status-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
