<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { save, open } from "@tauri-apps/plugin-dialog";

    export let updateGraph = () => {};

    let mode = "standard";
    let threshold = 0.5;
    let selectedFile = null;
    let isProcessing = false;

    async function handleFileSelect() {
        try {
            const selected = await open({
                multiple: false,
                title: "Select file to compress",
            });
            if (selected && typeof selected === "string") {
                selectedFile = {
                    name: selected.split(/[\/\\]/).pop(),
                    path: selected,
                };
                console.log(`Selected: ${selectedFile.name}`);
            }
        } catch (e) {
            console.error("Failed to open file dialog");
        }
    }

    async function handleCompress() {
        if (!selectedFile) {
            console.error("Please select a file first");
            return;
        }

        isProcessing = true;

        try {
            const destPath = await save({
                defaultPath: `${selectedFile.name}.qres`,
                title: "Save compressed file",
                filters: [{ name: "QRES", extensions: ["qres"] }],
            });

            if (!destPath) {
                isProcessing = false;
                return;
            }

            console.log("Compressing...");

            // Use compress_file for real functionality
            await invoke("compress_file", {
                src: selectedFile.path,
                dest: destPath,
            });

            console.log(`Compressed! Saved to ${destPath}`);
            updateGraph(); // Refresh neural graph
        } catch (error) {
            console.error(`Failed: ${error}`);
        } finally {
            isProcessing = false;
        }
    }

    async function handleDecompress() {
        isProcessing = true;

        try {
            const srcPath = await open({
                filters: [{ name: "QRES Files", extensions: ["qres", "qrar"] }],
                title: "Select file to decompress",
            });

            if (!srcPath || typeof srcPath !== "string") {
                isProcessing = false;
                return;
            }

            const destPath = await save({
                title: "Save decompressed file",
            });

            if (!destPath) {
                isProcessing = false;
                return;
            }

            console.log("Decompressing...");

            await invoke("decompress_file", {
                src: srcPath,
                dest: destPath,
            });

            console.log(`Decompressed to ${destPath}`);
        } catch (error) {
            console.error(`Failed: ${error}`);
        } finally {
            isProcessing = false;
        }
    }
</script>

<aside class="sidebar">
    <h2>🎛️ Control Panel</h2>

    <div class="section">
        <div class="mode-toggle">
            <button
                class:active={mode === "standard"}
                on:click={() => (mode = "standard")}
                disabled={isProcessing}
            >
                Standard
            </button>
            <button
                class:active={mode === "quantum"}
                on:click={() => (mode = "quantum")}
                disabled={isProcessing}
            >
                Quantum
            </button>
        </div>
    </div>

    <div class="section">
        <label for="threshold-slider">Threshold: {threshold.toFixed(2)}</label>
        <input
            id="threshold-slider"
            type="range"
            bind:value={threshold}
            min="0"
            max="1"
            step="0.01"
            disabled={isProcessing}
        />
    </div>

    <div class="section">
        <label for="file-picker">Source File</label>
        <button
            id="file-picker"
            class="picker-btn"
            on:click={handleFileSelect}
            disabled={isProcessing}
        >
            {selectedFile ? "📂 Change File" : "📁 Select File"}
        </button>

        {#if selectedFile}
            <div class="file-info" title={selectedFile.path}>
                📄 {selectedFile.name}
            </div>
        {/if}
    </div>

    <div class="actions">
        <button
            class="btn-primary"
            on:click={handleCompress}
            disabled={isProcessing || !selectedFile}
        >
            {isProcessing ? "⏳ Processing..." : "🚀 Compress"}
        </button>

        <button
            class="btn-secondary"
            on:click={handleDecompress}
            disabled={isProcessing}
        >
            📦 Decompress
        </button>
    </div>
</aside>

<style>
    .sidebar {
        grid-area: sidebar;
        padding: 1.5rem;
        background: rgba(10, 10, 42, 0.95);
        border-right: 1px solid rgba(0, 255, 204, 0.2);
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    h2 {
        margin: 0;
        color: #00ffcc;
        font-size: 1.1rem;
    }

    .section {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    label {
        color: #a8dadc;
        font-size: 0.85rem;
        text-transform: uppercase;
    }

    .mode-toggle {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 0.5rem;
    }

    .mode-toggle button {
        padding: 0.6rem;
        background: rgba(0, 128, 255, 0.1);
        border: 1px solid rgba(0, 128, 255, 0.3);
        border-radius: 4px;
        color: #a8dadc;
        cursor: pointer;
        transition: all 0.2s;
    }

    .mode-toggle button.active {
        background: #00ffcc;
        border-color: #00ffcc;
        color: #0a0a2a;
        font-weight: 600;
    }

    input[type="range"] {
        width: 100%;
        height: 4px;
        background: rgba(0, 128, 255, 0.2);
        border-radius: 2px;
        outline: none;
        -webkit-appearance: none;
        appearance: none;
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 16px;
        height: 16px;
        background: #00ffcc;
        border-radius: 50%;
        cursor: pointer;
    }

    .picker-btn {
        padding: 0.6rem;
        background: rgba(0, 128, 255, 0.1);
        border: 1px dashed rgba(0, 128, 255, 0.5);
        border-radius: 4px;
        color: #00ffcc;
        cursor: pointer;
        transition: all 0.2s;
        font-size: 0.85rem;
    }

    .picker-btn:hover:not(:disabled) {
        background: rgba(0, 128, 255, 0.2);
        border-style: solid;
        border-color: #00ffcc;
    }

    .file-info {
        padding: 0.5rem;
        background: rgba(0, 128, 255, 0.1);
        border: 1px solid rgba(0, 128, 255, 0.3);
        border-radius: 4px;
        color: #ffffff;
        font-size: 0.85rem;
    }

    .actions {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        margin-top: auto;
    }

    button {
        padding: 0.9rem;
        border: none;
        border-radius: 6px;
        font-size: 0.95rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-primary {
        background: linear-gradient(135deg, #00ffcc 0%, #0080ff 100%);
        color: #0a0a2a;
    }

    .btn-primary:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 255, 204, 0.4);
    }

    .btn-secondary {
        background: rgba(0, 128, 255, 0.2);
        color: #00ffcc;
        border: 1px solid rgba(0, 128, 255, 0.5);
    }

    .btn-secondary:hover:not(:disabled) {
        background: rgba(0, 128, 255, 0.3);
    }

    button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
