# QRES v19 Protocol Specification

## Overview
QRES v19 is a protocol for decentralized neural consensus. While it produces `.qres` artifacts (saved genes), its primary function is defining the `SwarmNeuron` trait for behavior and the Gossip headers used for gene propagation.

## Core Specifications
1. **The Neuron Protocol:** Defines how nodes compute residuals (`I16F16`) and BFP gradients (`Bfp16Vec`).
2. **The Gene Format:** A bytecode serialization standard for transmitting learned strategies across the gossip network.
3. **Consensus:** A deterministic, reputation-weighted agreement mechanism using Trimmed Mean Aggregation.

---

## 2. Header Structure (24 bytes)

| Offset | Length | Type | Description |
| :--- | :--- | :--- | :--- |
| 0 | 4 | `[u8; 4]` | Magic Bytes (`QRES`) |
| 4 | 2 | `u16` | Major Version (19) |
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
### 3.3 Summary Gene (Type `0x13`) - v19.0
Contains rapid onboarding state.

| Field | Size | Details |
| :--- | :--- | :--- |
| Block ID | 1 byte | `0x13` |
| Round | 8 bytes | `u64` Round Index |
| Hash | 32 bytes | History Hash |
| Consensus | N bytes | BFP-16 Encoded Vector |
| Variance | N bytes | BFP-16 Encoded Vector |

---

## 4. Deterministic Math Specification

### 4.1 State Consensus (Q16.16)
All consensus state must use `i32` fixed point:

```rust
// Current prediction value (16 bits integer, 16 bits fraction)
type Q16 = i32;
// 0.1 + 0.2 = round(6553.6 + 13107.2) = 19661
```

### 4.2 Gradient Updates (BFP-16) - v19.0
All gradients must use Block Floating Point to preserve dynamic range at low learning rates:

```rust
struct Bfp16Vec {
    exponent: i8,      // Shared exponent
    mantissas: Vec<i16> // Signed 16-bit integers
}
// Value[i] = mantissa[i] * 2^(exponent)
```
