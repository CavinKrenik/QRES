<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let swarmEnabled = false;
    let swarmStatus = "Offline";
    let stats = {
        bytes_saved: 0,
        total_compressions: 0,
        avg_ratio: 0,
        engines_used: {} as Record<string, number>,
    };

    async function loadData() {
        try {
            stats = await invoke("get_stats");
        } catch (e) {
            console.error("Failed to load data:", e);
        }
    }

    async function handleSwarmToggle() {
        try {
            const result = await invoke("toggle_swarm", {
                enabled: swarmEnabled,
            });
            swarmStatus = swarmEnabled ? "Connected" : "Offline";
            console.log(result);
        } catch (e) {
            console.error("Swarm toggle failed:", e);
            swarmEnabled = false;
            swarmStatus = "Offline";
        }
    }

    onMount(() => {
        loadData();
        const interval = setInterval(loadData, 5000);
        return () => clearInterval(interval);
    });

    $: hiveWisdom =
        stats.avg_ratio > 0 ? ((1 - stats.avg_ratio) * 100).toFixed(1) : "0.0";
    $: engineEntries = Object.entries(stats.engines_used);
    $: totalEngineUses = engineEntries.reduce(
        (sum, [, count]) => sum + count,
        0,
    );
</script>

<div class="hive-container">
    <div class="hive-header">
        <h2>🐝 Hive Mind</h2>
        <div class="swarm-toggle">
            <label>
                <input
                    type="checkbox"
                    bind:checked={swarmEnabled}
                    on:change={handleSwarmToggle}
                />
                <span>Swarm Network</span>
            </label>
            <span class="status" class:connected={swarmEnabled}>
                {swarmEnabled ? "🟢" : "⚪"}
                {swarmStatus}
            </span>
        </div>
    </div>

    <div class="stats-grid">
        <div class="stat-card">
            <div class="stat-label">Bytes Saved Today</div>
            <div class="stat-value">
                {(stats.bytes_saved / 1024 / 1024).toFixed(1)}MB
            </div>
        </div>

        <div class="stat-card highlight">
            <div class="stat-label">Hive Wisdom</div>
            <div class="stat-value">{hiveWisdom}%</div>
            <div class="stat-subtitle">Compression Efficiency</div>
        </div>

        <div class="stat-card">
            <div class="stat-label">Total Compressions</div>
            <div class="stat-value">{stats.total_compressions}</div>
        </div>
    </div>

    <div class="engine-usage">
        <h3>Engine Usage</h3>
        {#if engineEntries.length > 0}
            <div class="engine-bars">
                {#each engineEntries as [engine, count]}
                    {@const percentage =
                        totalEngineUses > 0
                            ? (count / totalEngineUses) * 100
                            : 0}
                    <div class="engine-bar">
                        <div class="engine-info">
                            <span class="engine-name"
                                >{engine.toUpperCase()}</span
                            >
                            <span class="engine-count"
                                >{count} uses ({percentage.toFixed(1)}%)</span
                            >
                        </div>
                        <div class="bar-container">
                            <div
                                class="bar-fill"
                                style="width: {percentage}%"
                            ></div>
                        </div>
                    </div>
                {/each}
            </div>
        {:else}
            <p class="no-data">
                No compression data yet. Start compressing files!
            </p>
        {/if}
    </div>
</div>

<style>
    .hive-container {
        padding: 2rem;
        max-width: 1200px;
        margin: 0 auto;
    }

    .hive-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 2rem;
    }

    h2 {
        margin: 0;
        font-size: 1.8rem;
        color: #a0c0ff;
    }

    .swarm-toggle {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .swarm-toggle label {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        cursor: pointer;
    }

    .swarm-toggle input[type="checkbox"] {
        width: 20px;
        height: 20px;
        cursor: pointer;
    }

    .status {
        padding: 0.5rem 1rem;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 20px;
        font-size: 0.9rem;
        color: #94a3b8;
    }

    .status.connected {
        color: #10b981;
        background: rgba(16, 185, 129, 0.1);
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 1.5rem;
        margin-bottom: 2rem;
    }

    .stat-card {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 1.5rem;
        transition:
            transform 0.2s,
            box-shadow 0.2s;
    }

    .stat-card:hover {
        transform: translateY(-2px);
        box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
    }

    .stat-card.highlight {
        background: linear-gradient(
            135deg,
            rgba(99, 102, 241, 0.1) 0%,
            rgba(192, 132, 252, 0.1) 100%
        );
        border-color: rgba(99, 102, 241, 0.3);
    }

    .stat-label {
        font-size: 0.85rem;
        color: #94a3b8;
        margin-bottom: 0.5rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .stat-value {
        font-size: 2.5rem;
        font-weight: 700;
        color: #e0e7ff;
    }

    .stat-subtitle {
        font-size: 0.75rem;
        color: #64748b;
        margin-top: 0.25rem;
    }

    .engine-usage {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 1.5rem;
    }

    h3 {
        margin-top: 0;
        margin-bottom: 1.5rem;
        font-size: 1.2rem;
        color: #a0c0ff;
    }

    .engine-bars {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .engine-bar {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .engine-info {
        display: flex;
        justify-content: space-between;
        font-size: 0.9rem;
    }

    .engine-name {
        font-weight: 600;
        color: #e0e7ff;
    }

    .engine-count {
        color: #94a3b8;
    }

    .bar-container {
        height: 8px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
        overflow: hidden;
    }

    .bar-fill {
        height: 100%;
        background: linear-gradient(90deg, #818cf8 0%, #c084fc 100%);
        border-radius: 4px;
        transition: width 0.3s ease;
    }

    .no-data {
        text-align: center;
        color: #64748b;
        padding: 2rem;
        font-style: italic;
    }
</style>
