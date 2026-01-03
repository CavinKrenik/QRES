<script>
    // @ts-nocheck
    import { quantumState, compressionStats, swarmStatus } from "../stores.js";

    let mode = "standard";
    let threshold = 0.5;
    let noiseLevel = 0.1;
    let inputFile = null;
    let isProcessing = false;
    let statusMessage = "";

    // Simulate API call (replace with actual fetch to Python backend)
    async function compressData() {
        if (!inputFile) {
            statusMessage = "⚠️ Please select a file first";
            return;
        }

        isProcessing = true;
        statusMessage = "🔄 Processing...";

        try {
            // Mock compression (replace with actual API call)
            await new Promise((resolve) => setTimeout(resolve, 1000));

            const mockResult = {
                mode: mode,
                ratio: mode === "quantum" ? 0.0039 : 0.62,
                originalSize: 1024,
                compressedSize: mode === "quantum" ? 4 : 635,
                timestamp: Date.now(),
            };

            compressionStats.set(mockResult);
            statusMessage = `✅ Compressed: ${(mockResult.ratio * 100).toFixed(2)}% ratio`;

            // Update quantum state if in quantum mode
            if (mode === "quantum") {
                quantumState.update((state) => ({
                    ...state,
                    fidelity: 0.999,
                    version: `v${Date.now()}`,
                    timestamp: Date.now(),
                }));
            }
        } catch (error) {
            statusMessage = `❌ Error: ${error.message}`;
        } finally {
            isProcessing = false;
        }
    }

    async function saveState() {
        isProcessing = true;
        statusMessage = "💾 Saving world state...";

        try {
            await new Promise((resolve) => setTimeout(resolve, 500));
            statusMessage = "✅ State saved successfully";
        } catch (error) {
            statusMessage = `❌ Save failed: ${error.message}`;
        } finally {
            isProcessing = false;
        }
    }

    async function broadcastState() {
        isProcessing = true;
        statusMessage = "📡 Broadcasting to swarm...";

        try {
            await new Promise((resolve) => setTimeout(resolve, 500));
            swarmStatus.update((status) => ({
                ...status,
                lastBroadcast: Date.now(),
            }));
            statusMessage = "✅ Broadcast queued";
        } catch (error) {
            statusMessage = `❌ Broadcast failed: ${error.message}`;
        } finally {
            isProcessing = false;
        }
    }

    function handleFileSelect(event) {
        const file = event.target.files[0];
        if (file) {
            inputFile = file;
            statusMessage = `📄 Selected: ${file.name}`;
        }
    }
</script>

<div class="controls-panel">
    <h3>🎛️ Compression Controls</h3>

    <div class="control-group">
        <label for="mode-select">Mode:</label>
        <select id="mode-select" bind:value={mode} disabled={isProcessing}>
            <option value="standard">Standard</option>
            <option value="quantum">Quantum</option>
        </select>
    </div>

    <div class="control-group">
        <label for="threshold-slider"
            >Relevance Threshold: {threshold.toFixed(2)}</label
        >
        <input
            id="threshold-slider"
            type="range"
            bind:value={threshold}
            min="0"
            max="1"
            step="0.05"
            disabled={isProcessing}
            aria-label="Relevance threshold for compression"
        />
    </div>

    {#if mode === "quantum"}
        <div class="control-group">
            <label for="noise-slider"
                >Noise Level: {noiseLevel.toFixed(2)}</label
            >
            <input
                id="noise-slider"
                type="range"
                bind:value={noiseLevel}
                min="0"
                max="0.5"
                step="0.05"
                disabled={isProcessing}
                aria-label="Quantum noise simulation level"
            />
        </div>
    {/if}

    <div class="control-group">
        <label for="file-input">Input File:</label>
        <input
            id="file-input"
            type="file"
            on:change={handleFileSelect}
            disabled={isProcessing}
            aria-label="Select file to compress"
        />
    </div>

    <div class="button-group">
        <button
            class="primary-btn"
            on:click={compressData}
            disabled={isProcessing || !inputFile}
            aria-label="Start compression"
        >
            {isProcessing ? "⏳ Processing..." : "🚀 Compress"}
        </button>

        {#if mode === "quantum"}
            <button
                class="secondary-btn"
                on:click={saveState}
                disabled={isProcessing}
                aria-label="Save world state"
            >
                💾 Save State
            </button>

            <button
                class="secondary-btn"
                on:click={broadcastState}
                disabled={isProcessing}
                aria-label="Broadcast to swarm"
            >
                📡 Broadcast
            </button>
        {/if}
    </div>

    {#if statusMessage}
        <div class="status-message" role="status" aria-live="polite">
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

    input[type="range"] {
        width: 100%;
        height: 6px;
        background: #0f3460;
        border-radius: 3px;
        outline: none;
        -webkit-appearance: none;
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 18px;
        height: 18px;
        background: #e94560;
        border-radius: 50%;
        cursor: pointer;
        transition: all 0.3s ease;
    }

    input[type="range"]::-webkit-slider-thumb:hover {
        transform: scale(1.2);
        box-shadow: 0 0 10px rgba(233, 69, 96, 0.6);
    }

    input[type="range"]::-moz-range-thumb {
        width: 18px;
        height: 18px;
        background: #e94560;
        border-radius: 50%;
        cursor: pointer;
        border: none;
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

    .secondary-btn {
        background: linear-gradient(135deg, #1a5490 0%, #0f3460 100%);
        color: #a8dadc;
        border: 1px solid #1a5490;
    }

    .secondary-btn:hover:not(:disabled) {
        background: linear-gradient(135deg, #1e6bb8 0%, #1a5490 100%);
        color: white;
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
