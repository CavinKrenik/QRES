warning: unexpected `cfg` condition value: `python`
 --> qres_core\src\lib.rs:6:7
  |
6 | #[cfg(feature = "python")]
  |       ^^^^^^^^^^^^^^^^^^ help: remove the condition
  |
  = note: no expected values for `feature`
  = help: consider adding `python` as a feature in `Cargo.toml`
  = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
  = note: `#[warn(unexpected_cfgs)]` on by default

warning: unexpected `cfg` condition value: `python`
 --> qres_core\src\lib.rs:8:7
  |
8 | #[cfg(feature = "python")]
  |       ^^^^^^^^^^^^^^^^^^ help: remove the condition
  |
  = note: no expected values for `feature`
  = help: consider adding `python` as a feature in `Cargo.toml`
  = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `gpu`
  --> qres_core\src\lib.rs:19:7
   |
19 | #[cfg(feature = "gpu")]
   |       ^^^^^^^^^^^^^^^ help: remove the condition
   |
   = note: no expected values for `feature`
   = help: consider adding `gpu` as a feature in `Cargo.toml`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `python`
   --> qres_core\src\lib.rs:574:7
    |
574 | #[cfg(feature = "python")]
    |       ^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `python` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `python`
   --> qres_core\src\lib.rs:587:7
    |
587 | #[cfg(feature = "python")]
    |       ^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `python` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `python`
   --> qres_core\src\lib.rs:600:7
    |
600 | #[cfg(feature = "python")]
    |       ^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `python` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `python`
   --> qres_core\src\lib.rs:611:7
    |
611 | #[cfg(feature = "python")]
    |       ^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `python` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: unexpected `cfg` condition value: `python`
   --> qres_core\src\lib.rs:630:7
    |
630 | #[cfg(feature = "python")]
    |       ^^^^^^^^^^^^^^^^^^ help: remove the condition
    |
    = note: no expected values for `feature`
    = help: consider adding `python` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration

warning: constant `CHUNK_SIZE` is never used
  --> qres_core\src\lib.rs:32:7
   |
32 | const CHUNK_SIZE: usize = 1024 * 1024;
   |       ^^^^^^^^^^
   |
   = note: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

warning: constant `QRES_MAGIC` is never used
  --> qres_core\src\lib.rs:33:7
   |
33 | const QRES_MAGIC: &[u8] = b"QRES";
   |       ^^^^^^^^^^

warning: constant `PREDICTOR_ID_DEFAULT` is never used
  --> qres_core\src\lib.rs:37:7
   |
37 | const PREDICTOR_ID_DEFAULT: u8 = 0;
   |       ^^^^^^^^^^^^^^^^^^^^

warning: constant `PREDICTOR_ID_NEURAL` is never used
  --> qres_core\src\lib.rs:38:7
   |
38 | const PREDICTOR_ID_NEURAL: u8 = 1;
   |       ^^^^^^^^^^^^^^^^^^^

warning: `qres_core` (lib) generated 12 warnings
    Checking qres_daemon v0.10.0 (C:\Dev\QRES\qres_rust\qres_daemon)
warning: unused import: `extract::State`
 --> qres_daemon\src\api.rs:2:5
  |
2 |     extract::State,
  |     ^^^^^^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

warning: unused variable: `state`
   --> qres_daemon\src\api.rs:152:9
    |
152 |     let state = Arc::new(RwLock::new(ApiState::new()));
    |         ^^^^^ help: if this is intentional, prefix it with an underscore: `_state`
    |
    = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: `qres_daemon` (bin "qres_daemon") generated 2 warnings (run `cargo fix --bin "qres_daemon" -p qres_daemon` to apply 2 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.50s
