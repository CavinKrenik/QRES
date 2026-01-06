<script>
    // @ts-nocheck
    import { invoke } from "@tauri-apps/api/core";
    import { save, open } from "@tauri-apps/plugin-dialog";
    import { toast } from "@zerodevx/svelte-toast";
    import { engine } from "../lib/compressionEngine"; // Import Hybrid Engine

    export let updateGraph = () => {};

    // UI State
    let runtimeMode = "native"; // 'native' | 'wasm'
    let compressionMode = "standard"; // 'standard' | 'quantum'
    let threshold = 0.5;
    let selectedFile = null;
    let isProcessing = false;

    // Detect environment
    let isNative = false;
    if (typeof window !== "undefined") {
        // @ts-ignore
        isNative = !!window.__TAURI__;
        // Default to WASM if not native
        if (!isNative) runtimeMode = "wasm";
    }

    function handleFileSelect(event) {
        const file = event.target.files[0];
        if (file) {
            selectedFile = file;
            toast.push(`Selected: ${file.name}`);
        }
    }

    async function handleCompress() {
        if (!selectedFile) {
            toast.push("Please select a file first");
            return;
        }

        isProcessing = true;
        const useWasm = runtimeMode === "wasm";

        try {
            // 1. Browser/WASM Mode
            if (useWasm) {
                toast.push("🚀 Starting WASM Compression...");
                const buffer = await selectedFile.arrayBuffer();
                const bytes = new Uint8Array(buffer);

                const result = await engine.compress(bytes, true);

                toast.push(
                    `✅ WASM Success! Ratio: ${(result.ratio * 100).toFixed(2)}% (${result.duration_ms.toFixed(0)}ms)`,
                );
                updateGraph();
                return;
            }

            // 2. Native/Daemon Mode
            if (!isNative) {
                toast.push(
                    "❌ Native mode requires Tauri environment. Switching to WASM.",
                );
                runtimeMode = "wasm";
                isProcessing = false;
                return; // Let user try again or auto-retry? Let's stop and let them click again.
            }

            // Native Logic
            const fileInput = document.getElementById("file-input");
            const filePath = fileInput.files[0].path; // Tauri injected path

            if (!filePath) {
                toast.push(
                    "Could not determine file path for native compression.",
                );
                isProcessing = false;
                return;
            }

            const destPath = await save({
                defaultPath: `${selectedFile.name}.qres`,
                title: "Save compressed file",
            });

            if (!destPath) {
                isProcessing = false;
                return;
            }

            const outPath = await invoke("compress", {
                path: filePath,
                mode: compressionMode,
                threshold,
                outPath: destPath,
            });
            toast.push(`⚡ Native Success! Saved to ${outPath}`);
            updateGraph();
        } catch (error) {
            console.error(error);
            toast.push(`Failed: ${error}`);
        } finally {
            isProcessing = false;
        }
    }

    async function handleDecompress() {
        // Decompression logic (Hybrid or Native? For now keep Native-only or add WASM later if needed)
        // The DropZone has hybrid logic, let's keep this simple for now.
        if (!isNative) {
            toast.push(
                "Decompression currently only supported in Native Mode.",
            );
            return;
        }

        isProcessing = true;
        try {
            const srcPath = await open({
                filters: [{ name: "QRES Files", extensions: ["qres", "qrar"] }],
                title: "Select file to decompress",
            });
            if (!srcPath) return;

            const destPath = await save({
                defaultPath: "extracted",
                title: "Save decompressed file",
            });
            if (!destPath) return;

            const outPath = await invoke("decompress", {
                path: srcPath,
                outFolder: destPath,
            });
            toast.push(`Decompressed to ${outPath}`);
        } catch (error) {
            toast.push(`Failed: ${error}`);
        } finally {
            isProcessing = false;
        }
    }
</script>

<aside class="sidebar">
    <h2>🎛️ Control Panel</h2>

    <!-- Runtime Toggle -->
    <div class="section">
        <span class="label">Engine Runtime</span>
        <div class="mode-toggle">
            <button
                class:active={runtimeMode === "native"}
                on:click={() => (runtimeMode = "native")}
                disabled={isProcessing || !isNative}
                title={!isNative
                    ? "Not available in browser"
                    : "Use Rust Daemon"}
            >
                ⚡ Native
            </button>
            <button
                class:active={runtimeMode === "wasm"}
                on:click={() => (runtimeMode = "wasm")}
                disabled={isProcessing}
            >
                🌐 WASM
            </button>
        </div>
    </div>

    <!-- Compression Mode (Only relevant for Native mostly, but we keep it) -->
    <div class="section">
        <span class="label">Algorithm</span>
        <div class="mode-toggle">
            <button
                class:active={compressionMode === "standard"}
                on:click={() => (compressionMode = "standard")}
                disabled={isProcessing}
            >
                Standard
            </button>
            <button
                class:active={compressionMode === "quantum"}
                on:click={() => (compressionMode = "quantum")}
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
        <label for="file-input">Select File</label>
        <input
            id="file-input"
            type="file"
            on:change={handleFileSelect}
            disabled={isProcessing}
        />

        {#if selectedFile}
            <div class="file-info">
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

    label,
    .label {
        color: #a8dadc;
        font-size: 0.85rem;
        text-transform: uppercase;
        display: block; /* Ensure it behaves like block label */
        margin-bottom: 0.5rem;
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

    input[type="file"] {
        padding: 0.5rem;
        background: rgba(0, 128, 255, 0.1);
        border: 1px solid rgba(0, 128, 255, 0.3);
        border-radius: 4px;
        color: #ffffff;
        cursor: pointer;
        font-size: 0.85rem;
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
