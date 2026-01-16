# QRES v18 Protocol Specification

## Overview
QRES v18 is a protocol for decentralized neural consensus. While it produces `.qres` artifacts (saved genes), its primary function is defining the `SwarmNeuron` trait for behavior and the Gossip headers used for gene propagation.

## Core Specifications
1. **The Neuron Protocol:** Defines how nodes compute residuals (`I16F16`) and detect 'Surprise' (Entropy).
2. **The Gene Format:** A bytecode serialization standard for transmitting learned strategies across the gossip network.
3. **Consensus:** A deterministic, reputation-weighted agreement mechanism.

---

## 2. Header Structure (24 bytes)

| Offset | Length | Type | Description |
| :--- | :--- | :--- | :--- |
| 0 | 4 | `[u8; 4]` | Magic Bytes (`QRES`) |
| 4 | 2 | `u16` | Major Version (12) |
| 6 | 2 | `u16` | Minor Version (0) |
| 8 | 4 | `u32` | Flags (Bitmask) |
| 12 | 8 | `u64` | Total Uncompressed Size |
| 20 | 4 | `u32` | Header Checksum (CRC32) |

### Flags
*   `0x01`: **Solid Archive** (Single stream, no random access)
*   `0x02`: **Encrypted** (AES-256-GCM)
*   `0x04`: **Checksummed** (Each block has CRC32)

---

## 3. Block Structure

A QRES file is a stream of Blocks. Blocks are either "Epiphanies" (Model Updates) or "Residuals" (Data).

### 3.1 Epiphany Block (Type `0x0E`)
Contains new weights for the predictor model.

| Field | Size | Details |
| :--- | :--- | :--- |
| Block ID | 1 byte | `0x0E` |
| Length | 2 bytes | Size of weight payload |
| Predictor ID | 1 byte | ID of predictor to update (e.g. 1=Linear) |
| Weights | N bytes | Q16.16 fixed-point weights |

### 3.2 Residual Block (Type `0x0D`)
Contains compressed residuals (prediction errors).

| Field | Size | Details |
| :--- | :--- | :--- |
| Block ID | 1 byte | `0x0D` |
| Compressed Len | 4 bytes | Size of bit-packed payload |
| Original Len | 4 bytes | Size of raw data |
| Payload | N bytes | Bit-packed residuals (Delta + RLE + Huffman) |

---

## 4. Deterministic Math Specification (Q16.16)

All predictors must implement the `Predictor` trait using `i32`:

```rust
// Current prediction value (16 bits integer, 16 bits fraction)
type Q16 = i32;

fn predict(history: &[u8]) -> u8 {
    let prediction: Q16 = calculate_weighted_sum(history);
    (prediction >> 16) as u8 // Truncate to integer byte
}
```

This ensures that `0.1 + 0.2` is represented as `6553 + 13107 = 19660` on all systems, avoiding IEEE 754 variance.
