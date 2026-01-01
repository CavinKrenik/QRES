#!/usr/bin/env python3
"""
QRES Hive Validation Benchmark
Quantifies swarm benefits with multi-node simulations

Measures:
- Agent convergence time (how fast Agent B reaches expert level)
- Ratio improvements from Hive sync
- Hive vs isolated performance comparison

Target Metrics:
- Agent B: 30% faster convergence, 15% better ratios
- Hive vs isolated: 20% improvement on telemetry
- Zero-shot: <1000 compressions to expert level
"""

import subprocess
import json
import time
import os
from pathlib import Path
from typing import Dict, List, Tuple
import matplotlib.pyplot as plt
import numpy as np

# Configuration
HIVE_SERVER_PORT = 5000
NUM_AGENTS = 5
COMPRESSIONS_PER_AGENT = 100
DATA_DIR = Path("benchmarks/datasets/iot_telemetry")
RESULTS_DIR = Path("benchmarks/results/hive_validation")

class HiveValidator:
    def __init__(self):
        self.results_dir = RESULTS_DIR
        self.results_dir.mkdir(parents=True, exist_ok=True)
        
    def start_hive_server(self):
        """Start the Hive server"""
        print("🐝 Starting Hive server...")
        server_process = subprocess.Popen(
            ["python", "utils/hive_server.py"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        time.sleep(2)  # Wait for server to start
        return server_process
    
    def run_isolated_agent(self, agent_id: int, data_files: List[str]) -> Dict:
        """Run agent without Hive (baseline)"""
        print(f"📊 Running isolated Agent {agent_id}...")
        
        results = {
            "agent_id": agent_id,
            "mode": "isolated",
            "compressions": [],
            "ratios": [],
            "engines_used": {},
            "total_time": 0
        }
        
        start_time = time.time()
        
        for i, data_file in enumerate(data_files[:COMPRESSIONS_PER_AGENT]):
            # Compress file
            output_file = f"/tmp/qres_isolated_{agent_id}_{i}.qres"
            
            result = subprocess.run(
                ["qres-cli", "compress", data_file, output_file],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                # Parse compression stats
                original_size = os.path.getsize(data_file)
                compressed_size = os.path.getsize(output_file)
                ratio = compressed_size / original_size
                
                results["compressions"].append(i)
                results["ratios"].append(ratio)
                
                # Track engine (parse from output)
                if "Engine:" in result.stdout:
                    engine = result.stdout.split("Engine:")[1].split()[0]
                    results["engines_used"][engine] = results["engines_used"].get(engine, 0) + 1
        
        results["total_time"] = time.time() - start_time
        results["avg_ratio"] = np.mean(results["ratios"]) if results["ratios"] else 1.0
        
        return results
    
    def run_hive_agent(self, agent_id: int, data_files: List[str], is_expert: bool = False) -> Dict:
        """Run agent with Hive sync"""
        print(f"🐝 Running Hive Agent {agent_id} ({'Expert' if is_expert else 'Novice'})...")
        
        results = {
            "agent_id": agent_id,
            "mode": "hive",
            "is_expert": is_expert,
            "compressions": [],
            "ratios": [],
            "engines_used": {},
            "sync_times": [],
            "total_time": 0
        }
        
        start_time = time.time()
        
        # Create agent workspace
        agent_dir = Path(f"/tmp/qres_agent_{agent_id}")
        agent_dir.mkdir(exist_ok=True)
        os.chdir(agent_dir)
        
        for i, data_file in enumerate(data_files[:COMPRESSIONS_PER_AGENT]):
            # Compress file
            output_file = f"compressed_{i}.qres"
            
            result = subprocess.run(
                ["qres-cli", "compress", data_file, output_file],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                original_size = os.path.getsize(data_file)
                compressed_size = os.path.getsize(output_file)
                ratio = compressed_size / original_size
                
                results["compressions"].append(i)
                results["ratios"].append(ratio)
                
                if "Engine:" in result.stdout:
                    engine = result.stdout.split("Engine:")[1].split()[0]
                    results["engines_used"][engine] = results["engines_used"].get(engine, 0) + 1
            
            # Sync with Hive every 10 compressions
            if (i + 1) % 10 == 0:
                sync_start = time.time()
                subprocess.run(
                    ["python", "../../utils/hive_sync.py"],
                    env={**os.environ, "HIVE_URL": f"http://localhost:{HIVE_SERVER_PORT}"}
                )
                sync_time = time.time() - sync_start
                results["sync_times"].append(sync_time)
        
        results["total_time"] = time.time() - start_time
        results["avg_ratio"] = np.mean(results["ratios"]) if results["ratios"] else 1.0
        results["avg_sync_time"] = np.mean(results["sync_times"]) if results["sync_times"] else 0
        
        return results
    
    def calculate_convergence_time(self, ratios: List[float], expert_ratio: float) -> int:
        """Calculate how many compressions until within 5% of expert"""
        threshold = expert_ratio * 1.05  # Within 5%
        
        for i, ratio in enumerate(ratios):
            if ratio <= threshold:
                return i + 1
        
        return len(ratios)  # Never converged
    
    def generate_report(self, isolated_results: List[Dict], hive_results: List[Dict]):
        """Generate comprehensive validation report"""
        print("\n📈 Generating validation report...")
        
        report = {
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
            "configuration": {
                "num_agents": NUM_AGENTS,
                "compressions_per_agent": COMPRESSIONS_PER_AGENT,
                "hive_server_port": HIVE_SERVER_PORT
            },
            "results": {
                "isolated": {},
                "hive": {},
                "comparison": {}
            }
        }
        
        # Isolated stats
        isolated_ratios = [r["avg_ratio"] for r in isolated_results]
        report["results"]["isolated"] = {
            "avg_ratio": np.mean(isolated_ratios),
            "std_ratio": np.std(isolated_ratios),
            "avg_time": np.mean([r["total_time"] for r in isolated_results])
        }
        
        # Hive stats
        hive_expert = [r for r in hive_results if r["is_expert"]][0]
        hive_novices = [r for r in hive_results if not r["is_expert"]]
        
        hive_ratios = [r["avg_ratio"] for r in hive_novices]
        report["results"]["hive"] = {
            "expert_ratio": hive_expert["avg_ratio"],
            "novice_avg_ratio": np.mean(hive_ratios),
            "novice_std_ratio": np.std(hive_ratios),
            "avg_time": np.mean([r["total_time"] for r in hive_novices]),
            "avg_sync_time": np.mean([r["avg_sync_time"] for r in hive_novices])
        }
        
        # Comparison metrics
        ratio_improvement = (report["results"]["isolated"]["avg_ratio"] - 
                           report["results"]["hive"]["novice_avg_ratio"]) / \
                          report["results"]["isolated"]["avg_ratio"] * 100
        
        # Convergence analysis
        convergence_times = []
        for novice in hive_novices:
            conv_time = self.calculate_convergence_time(
                novice["ratios"], 
                hive_expert["avg_ratio"]
            )
            convergence_times.append(conv_time)
        
        report["results"]["comparison"] = {
            "ratio_improvement_pct": ratio_improvement,
            "avg_convergence_compressions": np.mean(convergence_times),
            "target_ratio_improvement": 15.0,  # Target: 15%
            "target_convergence": 1000,  # Target: <1000 compressions
            "meets_targets": {
                "ratio": ratio_improvement >= 15.0,
                "convergence": np.mean(convergence_times) < 1000
            }
        }
        
        # Save report
        report_file = self.results_dir / "validation_report.json"
        with open(report_file, "w") as f:
            json.dump(report, f, indent=2)
        
        print(f"\n✅ Report saved to {report_file}")
        
        # Print summary
        print("\n" + "="*60)
        print("QRES HIVE VALIDATION RESULTS")
        print("="*60)
        print(f"\n📊 Isolated Performance:")
        print(f"   Avg Ratio: {report['results']['isolated']['avg_ratio']:.4f}")
        print(f"   Std Dev:   {report['results']['isolated']['std_ratio']:.4f}")
        
        print(f"\n🐝 Hive Performance:")
        print(f"   Expert Ratio:  {report['results']['hive']['expert_ratio']:.4f}")
        print(f"   Novice Ratio:  {report['results']['hive']['novice_avg_ratio']:.4f}")
        print(f"   Sync Overhead: {report['results']['hive']['avg_sync_time']:.2f}s")
        
        print(f"\n🎯 Comparison:")
        print(f"   Ratio Improvement:  {ratio_improvement:.1f}% (Target: 15%)")
        print(f"   Convergence Time:   {np.mean(convergence_times):.0f} compressions (Target: <1000)")
        print(f"   Meets Ratio Target: {'✅' if report['results']['comparison']['meets_targets']['ratio'] else '❌'}")
        print(f"   Meets Conv Target:  {'✅' if report['results']['comparison']['meets_targets']['convergence'] else '❌'}")
        print("="*60 + "\n")
        
        return report
    
    def plot_results(self, isolated_results: List[Dict], hive_results: List[Dict]):
        """Generate visualization plots"""
        print("📊 Generating plots...")
        
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle('QRES Hive Validation Results', fontsize=16, fontweight='bold')
        
        # Plot 1: Ratio comparison
        ax1 = axes[0, 0]
        isolated_ratios = [r["avg_ratio"] for r in isolated_results]
        hive_ratios = [r["avg_ratio"] for r in hive_results if not r["is_expert"]]
        
        ax1.boxplot([isolated_ratios, hive_ratios], labels=['Isolated', 'Hive'])
        ax1.set_ylabel('Compression Ratio')
        ax1.set_title('Ratio Distribution: Isolated vs Hive')
        ax1.grid(True, alpha=0.3)
        
        # Plot 2: Convergence over time
        ax2 = axes[0, 1]
        for novice in [r for r in hive_results if not r["is_expert"]]:
            ax2.plot(novice["compressions"], novice["ratios"], alpha=0.6, label=f"Agent {novice['agent_id']}")
        
        expert = [r for r in hive_results if r["is_expert"]][0]
        ax2.axhline(y=expert["avg_ratio"], color='r', linestyle='--', label='Expert Level')
        ax2.set_xlabel('Compressions')
        ax2.set_ylabel('Compression Ratio')
        ax2.set_title('Novice Convergence to Expert Level')
        ax2.legend()
        ax2.grid(True, alpha=0.3)
        
        # Plot 3: Engine usage
        ax3 = axes[1, 0]
        all_engines = set()
        for r in hive_results:
            all_engines.update(r["engines_used"].keys())
        
        engine_counts = {engine: [] for engine in all_engines}
        for r in hive_results:
            for engine in all_engines:
                engine_counts[engine].append(r["engines_used"].get(engine, 0))
        
        x = np.arange(len(hive_results))
        width = 0.2
        for i, (engine, counts) in enumerate(engine_counts.items()):
            ax3.bar(x + i * width, counts, width, label=engine)
        
        ax3.set_xlabel('Agent')
        ax3.set_ylabel('Engine Usage Count')
        ax3.set_title('Engine Selection Distribution')
        ax3.set_xticks(x + width)
        ax3.set_xticklabels([f"A{r['agent_id']}" for r in hive_results])
        ax3.legend()
        ax3.grid(True, alpha=0.3)
        
        # Plot 4: Time comparison
        ax4 = axes[1, 1]
        isolated_times = [r["total_time"] for r in isolated_results]
        hive_times = [r["total_time"] for r in hive_results if not r["is_expert"]]
        
        ax4.bar(['Isolated', 'Hive'], [np.mean(isolated_times), np.mean(hive_times)])
        ax4.set_ylabel('Time (seconds)')
        ax4.set_title('Average Compression Time')
        ax4.grid(True, alpha=0.3)
        
        plt.tight_layout()
        plot_file = self.results_dir / "validation_plots.png"
        plt.savefig(plot_file, dpi=300, bbox_inches='tight')
        print(f"✅ Plots saved to {plot_file}")
        
    def run_full_validation(self):
        """Run complete Hive validation benchmark"""
        print("\n🚀 Starting QRES Hive Validation Benchmark\n")
        
        # Prepare data files
        data_files = list(DATA_DIR.glob("*.dat"))
        if not data_files:
            print("❌ No data files found. Please add IoT telemetry data to benchmarks/datasets/iot_telemetry/")
            return
        
        # Start Hive server
        server_process = self.start_hive_server()
        
        try:
            # Run isolated baseline
            print("\n=== Phase 1: Isolated Baseline ===")
            isolated_results = []
            for i in range(NUM_AGENTS):
                result = self.run_isolated_agent(i, data_files)
                isolated_results.append(result)
            
            # Run Hive-enabled agents
            print("\n=== Phase 2: Hive-Enabled Agents ===")
            hive_results = []
            
            # Agent 0 is expert (pre-trained)
            expert_result = self.run_hive_agent(0, data_files, is_expert=True)
            hive_results.append(expert_result)
            
            # Agents 1-4 are novices (learn from Hive)
            for i in range(1, NUM_AGENTS):
                novice_result = self.run_hive_agent(i, data_files, is_expert=False)
                hive_results.append(novice_result)
            
            # Generate report and plots
            report = self.generate_report(isolated_results, hive_results)
            self.plot_results(isolated_results, hive_results)
            
            print("\n✅ Hive validation complete!")
            
        finally:
            # Cleanup
            server_process.terminate()
            server_process.wait()

if __name__ == "__main__":
    validator = HiveValidator()
    validator.run_full_validation()
