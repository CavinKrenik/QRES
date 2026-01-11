<script lang="ts">
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import { writable } from "svelte/store";
    import { toast } from "@zerodevx/svelte-toast";

    // Persistent swarm state
    const swarmEnabled = writable(false);

    let swarmStatus = "Offline";
    let stats = {
        bytes_saved: 0,
        total_compressions: 0,
        avg_ratio: 0,
        engines_used: {} as Record<string, number>,
    };

    async function loadData() {
        // @ts-ignore
        if (!window.__TAURI__) {
            toast.push("Running in browser mode - hive features disabled");
            return;
        }
        try {
            stats = await invoke("get_stats");

            // Load swarm status
            const enabled = (await invoke("get_swarm_status")) as boolean;
            swarmEnabled.set(enabled);
            swarmStatus = enabled ? "Connected" : "Offline";
        } catch (e) {
            console.error("Failed to load data:", e);
            toast.push(`Failed to load data: ${e}`);
        }
    }

    async function handleSwarmToggle() {
        // @ts-ignore
        if (!window.__TAURI__) {
            toast.push("Swarm toggle not available in browser mode");
            return;
        }
        const enabled = $swarmEnabled;
        try {
            const result = (await invoke("toggle_swarm", {
                enabled,
            })) as string;
            swarmStatus = enabled ? "Connected" : "Offline";
            console.log(result);

            if (enabled) {
                toast.push(
                    "Swarm Network Enabled - sharing learnings with the collective!",
                );
            } else {
                toast.push(
                    "Swarm Network Disabled - operating in isolated mode",
                );
            }
        } catch (e) {
            console.error("Swarm toggle failed:", e);
            swarmEnabled.set(false);
            swarmStatus = "Offline";
            alert("Failed to toggle swarm: " + e);
        }
    }

    function showNotification(title: string, message: string) {
        // Simple notification - could be enhanced with Tauri notifications
        console.log(`${title}: ${message}`);
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

    // React to swarm toggle changes
    $: if ($swarmEnabled !== undefined) {
        handleSwarmToggle();
    }
</script>

<div class="hive-container">
    <div class="hive-header">
        <h2>Hive Mind</h2>
        <div class="swarm-toggle">
            <label class="toggle-label">
                <input
                    type="checkbox"
                    bind:checked={$swarmEnabled}
                    class="toggle-input"
                />
                <span class="toggle-slider"></span>
                <span class="toggle-text">Swarm Network</span>
            </label>
            <span class="status" class:connected={$swarmEnabled}>
                {$swarmEnabled ? "🟢" : "⚪"}
                {swarmStatus}
            </span>
        </div>
    </div>

    <div class="collective-banner" class:active={$swarmEnabled}>
        {#if $swarmEnabled}
            <div class="banner-content">
                <span class="banner-icon">🌐</span>
                <div class="banner-text">
                    <strong>Collective Learning Active</strong>
                    <small>Sharing knowledge with the swarm</small>
                </div>
            </div>
        {:else}
            <div class="banner-content inactive">
                <span class="banner-icon">💤</span>
                <div class="banner-text">
                    <strong>Isolated Mode</strong>
                    <small>Enable swarm to share learnings</small>
                </div>
            </div>
        {/if}
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
        padding: 1rem;
        height: 100%;
        overflow-y: auto;
    }

    .hive-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1.5rem;
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

    .toggle-label {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        cursor: pointer;
        user-select: none;
    }

    .toggle-input {
        position: absolute;
        opacity: 0;
        width: 0;
        height: 0;
    }

    .toggle-slider {
        position: relative;
        width: 50px;
        height: 26px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 26px;
        transition: background 0.3s;
    }

    .toggle-slider::before {
        content: "";
        position: absolute;
        width: 20px;
        height: 20px;
        left: 3px;
        top: 3px;
        background: white;
        border-radius: 50%;
        transition: transform 0.3s;
    }

    .toggle-input:checked + .toggle-slider {
        background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    }

    .toggle-input:checked + .toggle-slider::before {
        transform: translateX(24px);
    }

    .toggle-text {
        font-size: 0.95rem;
        color: #e0e7ff;
    }

    .status {
        padding: 0.5rem 1rem;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 20px;
        font-size: 0.9rem;
        color: #94a3b8;
        transition: all 0.3s;
    }

    .status.connected {
        color: #10b981;
        background: rgba(16, 185, 129, 0.1);
        box-shadow: 0 0 20px rgba(16, 185, 129, 0.2);
    }

    .collective-banner {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 1rem 1.5rem;
        margin-bottom: 2rem;
        transition: all 0.3s;
    }

    .collective-banner.active {
        background: linear-gradient(
            135deg,
            rgba(16, 185, 129, 0.1) 0%,
            rgba(5, 150, 105, 0.1) 100%
        );
        border-color: rgba(16, 185, 129, 0.3);
    }

    .banner-content {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .banner-content.inactive {
        opacity: 0.6;
    }

    .banner-icon {
        font-size: 2rem;
    }

    .banner-text strong {
        display: block;
        font-size: 1rem;
        color: #e0e7ff;
        margin-bottom: 0.25rem;
    }

    .banner-text small {
        font-size: 0.85rem;
        color: #94a3b8;
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
