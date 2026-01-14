# Contributing to QRES

## Engineering Standards & Governance

### 1. Code Style
* **Rust (The Core):**
    * **No Unwrap:** `unwrap()` is strictly forbidden in `crates/`. Use `expect("context")` or handle `Result` properly.
    * **Determinism:** Do not use `std::collections::HashMap` for logic that affects model consensus. Use `BTreeMap` or deterministic alternatives.
    * **Formatting:** Run `cargo fmt` before pushing. 100 char line limit.
* **Python (The Research):**
    * **Formatting:** Use `Black` (88 chars).
    * **Type Hints:** Mandatory for shared libraries in `bindings/`.
* **Web (The Dashboard):**
    * **No Tailwind:** Use scoped CSS in Svelte components. Maintain the "Cyberpunk" aesthetic manually to keep builds lightweight.

### 2. Testing Protocol
* **Graceful Degradation:** Tests requiring large datasets (e.g., Jena Climate) must **skip** (not fail) if data is missing.
    ```python
    @pytest.mark.skipif(not os.path.exists("data/jena.csv"), reason="Dataset missing")
    ```
* **CI/CD:** All PRs must pass the `cross_arch_battle` workflow (x86, ARM, WASM).

### 3. Commit Messages
Follow Conventional Commits:
* `feat(core):` New deterministic math features
* `fix(daemon):` P2P sync bug fixes
* `perf(wasm):` Optimization for browser runtime
* `doc(spec):` Updates to protocol specification
