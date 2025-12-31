<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";

    let prompt = "";
    let result = "";
    let loading = false;
    let status = "Ready";

    async function runQuery() {
        if (!prompt) return;
        loading = true;
        status = "Querying Ollama...";
        result = "";

        try {
            // Call the Rust backend command
            const response = await invoke("query_lm", { prompt });
            result = response;
            status = "Complete";
        } catch (error) {
            console.error(error);
            status = "Error: " + error;
            result = "Is Ollama running? (ollama serve)";
        } finally {
            loading = false;
        }
    }

    /** @param {KeyboardEvent} e */
    function handleKeydown(e) {
        if (e.key === "Enter" && e.ctrlKey) {
            runQuery();
        }
    }
</script>

<div class="lm-container">
    <h3>🧠 AI Insights (Ollama)</h3>
    <div class="input-group">
        <textarea
            bind:value={prompt}
            on:keydown={handleKeydown}
            placeholder="Ask the Hive Mind (e.g., 'Generate 100 log lines for training')..."
            disabled={loading}
        ></textarea>
        <div class="actions">
            <button on:click={runQuery} disabled={loading || !prompt}>
                {loading ? "Thinking..." : "Query"}
            </button>
            <span class="status {status.startsWith('Error') ? 'error' : ''}"
                >{status}</span
            >
        </div>
    </div>

    {#if result}
        <div class="result-area">
            <h4>Response:</h4>
            <pre>{result}</pre>
        </div>
    {/if}
</div>

<style>
    .lm-container {
        background: rgba(255, 255, 255, 0.05);
        border-radius: 8px;
        padding: 1rem;
        margin-top: 1rem;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    h3 {
        margin-top: 0;
        color: #a0c0ff;
        font-size: 1.1rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    textarea {
        width: 100%;
        min-height: 80px;
        background: rgba(0, 0, 0, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
        color: #e0e0e0;
        padding: 0.5rem;
        font-family: inherit;
        resize: vertical;
    }

    textarea:focus {
        outline: none;
        border-color: #646cff;
    }

    .actions {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    button {
        background: #646cff;
        color: white;
        border: none;
        padding: 0.5rem 1rem;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 600;
        transition: background 0.2s;
    }

    button:hover:not(:disabled) {
        background: #747bff;
    }

    button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .status {
        font-size: 0.9rem;
        color: #888;
    }

    .status.error {
        color: #ff6b6b;
    }

    .result-area {
        margin-top: 1rem;
        background: rgba(0, 0, 0, 0.2);
        padding: 1rem;
        border-radius: 4px;
        border-left: 3px solid #646cff;
    }

    .result-area h4 {
        margin-top: 0;
        margin-bottom: 0.5rem;
        font-size: 0.9rem;
        color: #aaa;
    }

    pre {
        white-space: pre-wrap;
        word-wrap: break-word;
        margin: 0;
        font-family: "Fira Code", monospace;
        font-size: 0.9rem;
        color: #d0d0d0;
    }
</style>
