<script>
    import { engine } from "../lib/compressionEngine";
    import { createEventDispatcher } from "svelte";

    // UI State
    let mode = "native"; // 'native' | 'wasm'
    let inputFile = null;
    let isProcessing = false;
    let statusMessage = "";
    let useWasm = false; // Internal convenience

    const dispatch = createEventDispatcher();

    // Handle file selection
    function handleFileSelect(event) {
        const file = event.target.files[0];
        if (file) {
            inputFile = file;
            statusMessage = `📄 Selected: ${file.name}`;
        }
    }

    // Real Compression Call
    async function compressData() {
        if (!inputFile) return;

        isProcessing = true;
        // Sync toggles
        useWasm = mode === "wasm";
        statusMessage = useWasm
            ? "🚀 Compressing in Browser..."
            : "⚡ Compressing in Daemon...";

        try {
            // Read bytes
            const buffer = await inputFile.arrayBuffer();
            const bytes = new Uint8Array(buffer);

            // Call Hybrid Engine
            const result = await engine.compress(bytes, useWasm);

            statusMessage = `✅ Done! Ratio: ${(result.ratio * 100).toFixed(2)}% (${result.duration_ms.toFixed(0)}ms)`;

            // Notify parent
            dispatch("complete", result);
        } catch (error) {
            statusMessage = `❌ Error: ${error}`;
            console.error(error);
        } finally {
            isProcessing = false;
        }
    }
</script>

<div class="controls-panel">
    <h3>🎛️ Hybrid Controls</h3>

    <div class="control-group">
        <label>Runtime Engine:</label>
        <select bind:value={mode} disabled={isProcessing}>
            <option value="native">⚡ Native (Rust Daemon)</option>
            <option value="wasm">🌐 WebAssembly (Browser)</option>
        </select>
    </div>

    <div class="control-group">
        <label>Input File:</label>
        <input
            type="file"
            on:change={handleFileSelect}
            disabled={isProcessing}
        />
    </div>

    <div class="button-group">
        <button
            class="primary-btn"
            on:click={compressData}
            disabled={isProcessing || !inputFile}
        >
            {isProcessing ? "⏳ Processing..." : "🚀 Compress"}
        </button>
    </div>

    {#if statusMessage}
        <div class="status-message">
            {statusMessage}
        </div>
    {/if}
</div>

<style>
    .controls-panel {
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

    .control-group {
        margin-bottom: 20px;
    }

    label {
        display: block;
        color: #a8dadc;
        margin-bottom: 8px;
        font-size: 0.95em;
        font-weight: 500;
    }

    select,
    input[type="file"] {
        width: 100%;
        padding: 10px;
        background: #0f3460;
        border: 1px solid #1a5490;
        border-radius: 6px;
        color: #f1faee;
        font-size: 1em;
        transition: all 0.3s ease;
    }

    select:hover,
    input[type="file"]:hover {
        border-color: #e94560;
    }

    select:focus,
    input[type="file"]:focus {
        outline: none;
        border-color: #e94560;
        box-shadow: 0 0 0 3px rgba(233, 69, 96, 0.2);
    }

    .button-group {
        display: flex;
        gap: 12px;
        flex-wrap: wrap;
        margin-top: 20px;
    }

    button {
        padding: 12px 24px;
        border: none;
        border-radius: 8px;
        font-size: 1em;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
        flex: 1;
        min-width: 120px;
    }

    .primary-btn {
        background: linear-gradient(135deg, #e94560 0%, #d63447 100%);
        color: white;
        box-shadow: 0 4px 6px rgba(233, 69, 96, 0.3);
    }

    .primary-btn:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 6px 12px rgba(233, 69, 96, 0.4);
    }

    button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .status-message {
        margin-top: 16px;
        padding: 12px;
        background: rgba(168, 218, 220, 0.1);
        border-left: 4px solid #e94560;
        border-radius: 4px;
        color: #a8dadc;
        font-size: 0.95em;
    }

    @media (max-width: 768px) {
        .button-group {
            flex-direction: column;
        }

        button {
            width: 100%;
        }
    }
</style>
