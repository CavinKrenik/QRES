import { invoke } from '@tauri-apps/api/core';
// @ts-ignore - The package is local and might not have perfect typings initially
import * as qresWasm from 'qres-wasm';

export interface CompressionResult {
    data: Uint8Array;
    ratio: number;
    engine: 'NATIVE' | 'WASM';
    duration_ms: number;
}

export class CompressionEngine {
    private wasmInitialized = false;

    async initWasm() {
        if (!this.wasmInitialized) {
            console.log("⚙️ Initializing WASM Core...");
            await qresWasm.default(); // Initialize WASM memory
            this.wasmInitialized = true;
        }
    }

    async compress(fileBytes: Uint8Array, useWasm: boolean = false): Promise<CompressionResult> {
        const start = performance.now();

        if (useWasm) {
            // 🌐 BROWSER MODE (Client-side)
            console.log("🚀 Mode: WASM (Browser Core)");
            await this.initWasm();

            // Call the Rust core directly in browser memory
            const compressed = qresWasm.compress_web(fileBytes);

            const end = performance.now();
            return {
                data: compressed,
                ratio: compressed.length / fileBytes.length,
                engine: 'WASM',
                duration_ms: end - start
            };

        } else {
            // 🖥️ NATIVE MODE (Tauri Daemon)
            console.log("⚡ Mode: NATIVE (Rust Daemon)");

            // Invoke the Tauri command
            // Ensure your src-tauri/src/lib.rs exposes this command!
            const result: any = await invoke('compress_buffer', {
                buffer: Array.from(fileBytes)
            });

            const end = performance.now();
            return {
                data: Uint8Array.from(result.data),
                ratio: result.ratio,
                engine: 'NATIVE',
                duration_ms: end - start
            };
        }
    }
}

export const engine = new CompressionEngine();
