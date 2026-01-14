/**
 * SensorSimulator.ts
 * Generates fake IoT telemetry data to test QRES compression.
 * Simulates a 10Hz stream from a vibration sensor.
 */

export interface TelemetryPacket {
    timestamp: number;
    deviceId: string;
    temp: number;
    vibration: number;
    battery: number;
    status: 'IDLE' | 'LEARNING' | 'INFERRING';
}

type DataCallback = (data: TelemetryPacket) => void;

export class SensorSimulator {
    private intervalId: number | null = null;
    private onData: DataCallback;
    private isRegimeChanged: boolean = false;
    private deviceId: string = "ESP32-EDGE-01";

    // Normal operating parameters
    private baseTemp = 45.0;
    private baseVibration = 2.5;

    // Anomaly operating parameters
    private anomalyTemp = 65.0;
    private anomalyVibration = 45.0;

    constructor(onData: DataCallback) {
        this.onData = onData;
    }

    public start(frequencyHz: number = 10) {
        if (this.intervalId) return;

        const intervalMs = 1000 / frequencyHz;

        this.intervalId = setInterval(() => {
            const now = Date.now();

            // Add some random noise
            const noise = (Math.random() - 0.5) * 0.5;

            let temp, vibration;

            if (this.isRegimeChanged) {
                // High vibration, high temp state
                temp = this.anomalyTemp + (Math.random() * 2);
                vibration = this.anomalyVibration + (Math.random() * 10);
            } else {
                // Normal state
                temp = this.baseTemp + noise;
                vibration = this.baseVibration + (Math.random() * 0.2);
            }

            const packet: TelemetryPacket = {
                timestamp: now,
                deviceId: this.deviceId,
                temp: parseFloat(temp.toFixed(2)),
                vibration: parseFloat(vibration.toFixed(4)),
                battery: parseFloat((98.0 - (Date.now() % 100000) / 5000).toFixed(1)), // Slow drain
                status: this.isRegimeChanged ? 'LEARNING' : 'INFERRING'
            };

            this.onData(packet);

        }, intervalMs) as unknown as number;
    }

    public stop() {
        if (this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = null;
        }
    }

    public triggerRegimeChange() {
        this.isRegimeChanged = !this.isRegimeChanged;
        return this.isRegimeChanged;
    }
}
