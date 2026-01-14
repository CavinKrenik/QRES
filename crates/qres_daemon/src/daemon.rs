use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use sysinfo::{Pid, System};
use tracing::info;

pub struct DaemonManager;

impl DaemonManager {
    fn get_qres_dir() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".qres");
        fs::create_dir_all(&path).expect("Could not create .qres directory");
        path
    }

    pub fn get_pid_file() -> PathBuf {
        Self::get_qres_dir().join("swarm.pid")
    }

    pub fn get_state_file() -> PathBuf {
        Self::get_qres_dir().join("swarm.state")
    }

    pub fn start(wan: bool, interval: u64) -> Result<(), String> {
        let pid_file = Self::get_pid_file();

        // Check if running
        if let Ok(content) = fs::read_to_string(&pid_file) {
            if let Ok(pid_val) = content.trim().parse::<usize>() {
                let s = System::new_all();
                let pid = Pid::from(pid_val);
                if s.process(pid).is_some() {
                    return Err(format!("Daemon already running with PID {}", pid_val));
                }
            }
        }

        // Spawn Child
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;

        // Use Command builder pattern cleanly
        let mut cmd = Command::new(exe);
        cmd.arg("swarm")
            .arg("run-node")
            .arg("--gossip-interval")
            .arg(interval.to_string());

        if wan {
            cmd.arg("--wan");
        }

        let child = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to spawn daemon: {}", e))?;

        // Write PID
        fs::write(&pid_file, child.id().to_string())
            .map_err(|e| format!("Failed to write PID file: {}", e))?;

        info!(pid = child.id(), "Swarm Daemon started");
        if wan {
            info!("WAN Mode Enabled (Global DHT)");
        }
        Ok(())
    }

    pub fn stop() -> Result<(), String> {
        let pid_file = Self::get_pid_file();
        let content = fs::read_to_string(&pid_file)
            .map_err(|_| "No active daemon found (missing PID file).")?;
        let pid_str = content.trim();
        let pid_val: usize = pid_str.parse().map_err(|_| "Invalid PID file content")?;

        let s = System::new_all();
        let pid = Pid::from(pid_val);

        if let Some(process) = s.process(pid) {
            process.kill();
            info!(pid = pid_val, "Swarm Daemon stopped");
            let _ = fs::remove_file(pid_file);
            Ok(())
        } else {
            // Cleanup stale PID
            let _ = fs::remove_file(pid_file);
            Err(format!(
                "Process {} not found. Removed stale PID file.",
                pid_val
            ))
        }
    }

    pub fn status() {
        let pid_file = Self::get_pid_file();
        let state_file = Self::get_state_file();

        // Check Process
        let mut running = false;
        if let Ok(content) = fs::read_to_string(&pid_file) {
            if let Ok(pid_val) = content.trim().parse::<usize>() {
                let s = System::new_all();
                let pid = Pid::from(pid_val);
                if s.process(pid).is_some() {
                    info!(status = "RUNNING", pid = pid_val, "Daemon Status");
                    running = true;
                } else {
                    info!(
                        status = "CRASHED/STOPPED",
                        reason = "Stale PID",
                        "Daemon Status"
                    );
                }
            } else {
                info!(status = "UNKNOWN", reason = "Corrupt PID", "Daemon Status");
            }
        } else {
            info!(status = "STOPPED", "Daemon Status");
        }

        if running {
            // Read State
            if let Ok(json) = fs::read_to_string(&state_file) {
                // Parse JSON string to object to avoid double serialization if possible, or just log raw
                // Since tracing handles JSON, we can pass it as a field or just rely on the user reading the file.
                // But for "status", let's log it.
                if let Ok(metrics) = serde_json::from_str::<serde_json::Value>(&json) {
                    info!(metrics = %metrics, "Swarm Metrics");
                } else {
                    info!(raw_metrics = json, "Swarm Metrics (Raw)");
                }
            } else {
                info!("Waiting for metrics report...");
            }
        }
    }
}
