<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import { streamingActive, compressionStats } from "../lib/iotStore";

    const dispatch = createEventDispatcher();
    let isRegimeChanged = false;

    function toggleStream() {
        $streamingActive = !$streamingActive;
        dispatch("toggle", $streamingActive);
    }

    function triggerRegime() {
        isRegimeChanged = !isRegimeChanged;
        dispatch("regimeChange", isRegimeChanged);
    }
</script>

<div class="control-panel">
    <button
        class="connect-btn"
        class:active={$streamingActive}
        on:click={toggleStream}
    >
        <div class="status-indicator" class:pulse={$streamingActive}></div>
        {$streamingActive ? "DISCONNECT SWARM" : "CONNECT TO SWARM"}
    </button>

    {#if $streamingActive}
        <div class="stats-row">
            <div class="stat">
                <span class="label">SAVINGS</span>
                <span class="value text-green"
                    >{$compressionStats.savings}%</span
                >
            </div>
            <div class="stat">
                <span class="label">RATIO</span>
                <span class="value text-cyan">{$compressionStats.ratio}:1</span>
            </div>
        </div>

        <button
            class="regime-btn"
            class:danger={isRegimeChanged}
            on:click={triggerRegime}
        >
            ⚠️ {isRegimeChanged ? "NORMALIZE SIGNAL" : "TRIGGER ANOMALY"}
        </button>
    {/if}
</div>

<style>
    .control-panel {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        padding: 1.5rem;
        background: rgba(10, 10, 42, 0.6);
        border: 1px solid #00ffcc33;
        border-radius: 8px;
    }

    .connect-btn {
        position: relative;
        width: 100%;
        padding: 1rem;
        background: #0a0a2a;
        border: 2px solid #00ffcc;
        color: #00ffcc;
        font-family: "JetBrains Mono", monospace;
        font-weight: bold;
        font-size: 1.1rem;
        cursor: pointer;
        transition: all 0.3s ease;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 10px;
    }

    .connect-btn.active {
        background: #00ffcc22;
        box-shadow: 0 0 15px #00ffcc44;
    }

    .status-indicator {
        width: 10px;
        height: 10px;
        background: #333;
        border-radius: 50%;
    }

    .status-indicator.pulse {
        background: #00ffcc;
        box-shadow: 0 0 10px #00ffcc;
        animation: pulse 1.5s infinite;
    }

    .stats-row {
        display: flex;
        justify-content: space-between;
        margin-top: 0.5rem;
    }

    .stat {
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .label {
        font-size: 0.7rem;
        color: #888;
    }

    .value {
        font-size: 1.2rem;
        font-weight: bold;
    }

    .text-green {
        color: #00ffcc;
    }
    .text-cyan {
        color: #00aaff;
    }

    .regime-btn {
        background: #2a0a0a;
        border: 1px solid #ff4444;
        color: #ff4444;
        padding: 0.8rem;
        font-family: "JetBrains Mono", monospace;
        cursor: pointer;
        transition: 0.2s;
    }

    .regime-btn:hover {
        background: #ff4444;
        color: #000;
    }

    .regime-btn.danger {
        background: #ff4444;
        color: #000;
        animation: shake 0.5s;
    }

    @keyframes pulse {
        0% {
            opacity: 1;
        }
        50% {
            opacity: 0.5;
        }
        100% {
            opacity: 1;
        }
    }

    @keyframes shake {
        0% {
            transform: translateX(0);
        }
        25% {
            transform: translateX(5px);
        }
        50% {
            transform: translateX(-5px);
        }
        75% {
            transform: translateX(5px);
        }
        100% {
            transform: translateX(0);
        }
    }
</style>
