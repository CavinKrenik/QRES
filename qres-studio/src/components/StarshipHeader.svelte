<script lang="ts">
    import { streamingActive } from "../lib/iotStore";

    // Toggle the global store
    function toggleConnection() {
        streamingActive.update((n) => !n);
    }
</script>

<header class="starship-header">
    <div class="logo-area">
        <div class="title-group">
            <h1>QRES STUDIO</h1>
            <span class="version">v15.4.0 // EDGE MONITOR</span>
        </div>
    </div>

    <div class="header-controls">
        <div class="status-indicator">
            <span class="label">SYSTEM STATUS:</span>
            <span class="val" class:online={$streamingActive}>
                {$streamingActive ? "ONLINE" : "STANDBY"}
            </span>
        </div>
    </div>
</header>

<style>
    .starship-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.8rem 1.5rem;
        background: rgba(10, 10, 20, 0.95);
        border-bottom: 1px solid rgba(0, 255, 204, 0.1);
        backdrop-filter: blur(10px);
        flex-shrink: 0;
    }

    .logo-area {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .title-group h1 {
        margin: 0;
        font-family: "JetBrains Mono", monospace;
        font-size: 1.2rem;
        letter-spacing: 2px;
        background: linear-gradient(135deg, #00ffcc 0%, #0080ff 100%);
        background-clip: text;
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .version {
        font-family: "JetBrains Mono", monospace;
        font-size: 0.7rem;
        color: #666;
    }

    .header-controls {
        display: flex;
        align-items: center;
        gap: 2rem;
    }

    .status-indicator {
        font-family: "JetBrains Mono", monospace;
        font-size: 0.8rem;
        color: #888;
    }

    .status-indicator .val {
        color: #555;
        margin-left: 0.5rem;
    }

    .status-indicator .val.online {
        color: #00ffcc;
        text-shadow: 0 0 8px rgba(0, 255, 204, 0.6);
    }
</style>
