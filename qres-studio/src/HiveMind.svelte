<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    interface Stats {
        bytes_saved: number;
        total_compressions: number;
        engines_used: Record<string, number>;
        avg_ratio?: number;
    }

    let swarmEnabled = false;
    let stats: Stats = {
        bytes_saved: 0,
        total_compressions: 0,
        engines_used: {},
        avg_ratio: 0,
    };
    let wisdom = 0;

    async function toggleSwarm() {
        try {
            await invoke("toggle_swarm", { enabled: !swarmEnabled });
            swarmEnabled = !swarmEnabled;
        } catch (e) {
            console.error("Failed to toggle swarm:", e);
        }
    }

    async function loadData() {
        try {
            stats = await invoke("get_stats");
            // Calculate wisdom from stats
            const total = Object.values(stats.engines_used || {}).reduce(
                (a, b) => a + b,
                0,
            );
            wisdom = total > 0 ? (stats.avg_ratio || 0.5) * 100 : 0;
        } catch (e) {
            console.error("Failed to load data:", e);
        }
    }

    onMount(() => {
        loadData();
        setInterval(loadData, 5000);
    });
</script>

<div class="hive-container">
    <div class="stats-grid">
        <div class="stat-card">
            <div class="stat-value">
                {((stats.bytes_saved || 0) / 1024 / 1024).toFixed(1)}MB
            </div>
            <div class="stat-label">Bytes Saved Today</div>
        </div>

        <div class="stat-card">
            <div class="stat-value">{wisdom.toFixed(0)}%</div>
            <div class="stat-label">Hive Wisdom</div>
        </div>

        <div class="stat-card">
            <div class="stat-value">{stats.total_compressions || 0}</div>
            <div class="stat-label">Total Compressions</div>
        </div>
    </div>

    <div class="swarm-control">
        <h3>Swarm Network</h3>
        <label class="toggle">
            <input
                type="checkbox"
                checked={swarmEnabled}
                on:change={toggleSwarm}
            />
            <span class="slider"></span>
        </label>
        <p class="swarm-status">
            {swarmEnabled ? "🟢 Connected to Hive" : "⚪ Offline"}
        </p>
    </div>

    <div class="engine-breakdown">
        <h3>Engine Usage</h3>
        <div class="engines">
            {#each Object.entries(stats.engines_used || {}) as [engine, count]}
                <div class="engine-bar">
                    <span class="engine-name">{engine}</span>
                    <div class="bar">
                        <div
                            class="fill"
                            style="width: {(count / stats.total_compressions) *
                                100 || 0}%"
                        ></div>
                    </div>
                    <span class="engine-count">{count}</span>
                </div>
            {/each}
        </div>
    </div>
</div>

<style>
    .hive-container {
        padding: 2rem;
        max-width: 1000px;
        margin: 0 auto;
    }

    .stats-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1.5rem;
        margin-bottom: 2rem;
    }

    .stat-card {
        background: rgba(15, 23, 42, 0.6);
        border: 1px solid rgba(99, 102, 241, 0.2);
        border-radius: 12px;
        padding: 1.5rem;
        text-align: center;
    }

    .stat-value {
        font-size: 2.5rem;
        font-weight: 700;
        background: linear-gradient(135deg, #818cf8 0%, #c084fc 100%);
        background-clip: text;
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        margin-bottom: 0.5rem;
    }

    .stat-label {
        color: #94a3b8;
        font-size: 0.9rem;
    }

    .swarm-control {
        background: rgba(15, 23, 42, 0.6);
        border: 1px solid rgba(99, 102, 241, 0.2);
        border-radius: 12px;
        padding: 1.5rem;
        margin-bottom: 2rem;
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .swarm-control h3 {
        margin: 0;
        flex: 1;
    }

    .toggle {
        position: relative;
        display: inline-block;
        width: 60px;
        height: 34px;
    }

    .toggle input {
        opacity: 0;
        width: 0;
        height: 0;
    }

    .slider {
        position: absolute;
        cursor: pointer;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-color: #334155;
        transition: 0.4s;
        border-radius: 34px;
    }

    .slider:before {
        position: absolute;
        content: "";
        height: 26px;
        width: 26px;
        left: 4px;
        bottom: 4px;
        background-color: white;
        transition: 0.4s;
        border-radius: 50%;
    }

    input:checked + .slider {
        background-color: #818cf8;
    }

    input:checked + .slider:before {
        transform: translateX(26px);
    }

    .swarm-status {
        margin: 0;
        color: #94a3b8;
        font-size: 0.9rem;
    }

    .engine-breakdown {
        background: rgba(15, 23, 42, 0.6);
        border: 1px solid rgba(99, 102, 241, 0.2);
        border-radius: 12px;
    }

    .engines {
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .engine-bar {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .engine-name {
        width: 80px;
        text-transform: uppercase;
        font-size: 0.85rem;
        color: #94a3b8;
    }

    .bar {
        flex: 1;
        height: 24px;
        background: rgba(51, 65, 85, 0.5);
        border-radius: 12px;
        overflow: hidden;
    }

    .fill {
        height: 100%;
        background: linear-gradient(90deg, #818cf8 0%, #c084fc 100%);
        transition: width 0.5s;
    }

    .engine-count {
        width: 40px;
        text-align: right;
        color: #94a3b8;
        font-size: 0.9rem;
    }
</style>
