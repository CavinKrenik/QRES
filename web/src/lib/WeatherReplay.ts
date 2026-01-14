/**
 * WeatherReplay.ts
 * Replays real-world Jena Climate data to validate QRES on non-synthetic streams.
 * Maps Atmospheric Pressure -> Vibration to visualize "Storms" as "Stress".
 */

import weatherData from './weather_data.json';

export interface WeatherFrame {
    temp: number;
    vibration: number;
    pressure_raw: number;
}

export interface TelemetryPacket {
    timestamp: number;
    deviceId: string;
    temp: number;
    vibration: number;
    battery: number;
    status: 'IDLE' | 'LEARNING' | 'INFERRING';
    // Debug fields
    pressure_raw: number;
    frameIndex: number;
}

type DataCallback = (data: TelemetryPacket) => void;

export class WeatherReplay {
    private intervalId: number | null = null;
    private onData: DataCallback;
    private index = 0;
    private deviceId = "JENA-STATION-04";
    private frames: WeatherFrame[] = weatherData as WeatherFrame[];

    constructor(onData: DataCallback) {
        this.onData = onData;
        console.log(`[WeatherReplay] Loaded ${this.frames.length} frames from Jena Climate Dataset`);
    }

    public start(frequencyHz: number = 10) { // 10Hz for smooth UI
        if (this.intervalId) return;

        console.log(`[WeatherReplay] Starting replay at ${frequencyHz}Hz`);
        const intervalMs = 1000 / frequencyHz;

        this.intervalId = setInterval(() => {
            if (!this.frames || this.frames.length === 0) {
                console.error('[WeatherReplay] No data loaded!');
                return;
            }

            const frame = this.frames[this.index];
            const now = Date.now();

            // Simulate battery drain over the replay period
            const batteryLevel = 100 - (this.index / this.frames.length) * 20;

            // --- AUTO-TRIGGER LOGIC ---
            // Threshold 1.0: Calm (0.0) won't trigger, Storm (15+) will
            const isStorming = frame.vibration > 1.0;

            // Narrative phase markers for demo
            if (this.index === 0) console.log("🎬 STARTING: Calm Phase (High Compression)");
            if (this.index === 5000) console.log("🎬 TRANSITION: Storm Phase Arriving!");

            // Debug: Log storm triggers
            if (isStorming && this.index % 50 === 0) {
                console.log(`⚡ STORM: Vibration ${frame.vibration.toFixed(2)} > 1.0 → LEARNING`);
            }

            const packet: TelemetryPacket = {
                timestamp: now,
                deviceId: this.deviceId,
                temp: frame.temp,
                vibration: parseFloat(frame.vibration.toFixed(4)),
                battery: parseFloat(batteryLevel.toFixed(1)),
                status: isStorming ? 'LEARNING' : 'INFERRING',
                // Debug fields
                pressure_raw: frame.pressure_raw,
                frameIndex: this.index
            };

            // Log periodically for debugging
            if (this.index % 100 === 0) {
                console.log(`[WeatherReplay] Frame ${this.index}: ${isStorming ? '🌧️ STORM' : '☀️ CALM'} | temp=${frame.temp.toFixed(1)}°C | pressure=${frame.pressure_raw.toFixed(1)}mbar | vibration=${frame.vibration.toFixed(2)}`);
            }

            this.onData(packet);

            // Loop forever through the dataset
            this.index = (this.index + 1) % this.frames.length;

        }, intervalMs) as unknown as number;
    }

    public stop() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
            console.log('[WeatherReplay] Stopped');
        }
    }

    public triggerRegimeChange() {
        const oldIndex = this.index;
        this.index = (this.index + 200) % this.frames.length;
        console.log(`[WeatherReplay] Manual regime change: jumped from frame ${oldIndex} to ${this.index}`);
        return true;
    }

    public getCurrentIndex(): number {
        return this.index;
    }

    public getTotalFrames(): number {
        return this.frames.length;
    }
}
