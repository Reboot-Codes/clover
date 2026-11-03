#!/usr/bin/env python3
import subprocess
import time
import signal
import sys
import os
from datetime import datetime

script_dir = os.path.dirname(os.path.abspath(__file__))
repo_root = os.path.dirname(script_dir)

bin_path = os.path.join(repo_root, "target/release/clover-hub")
if not os.path.exists(bin_path):
    bin_path = os.path.join(repo_root, "target/debug/clover-hub")
if not os.path.exists(bin_path):
    bin_path = "./target/release/clover-hub"

log_file = "overnight_memory.csv"

print(f"[{datetime.now()}] Launching {bin_path} with /opt/clover...")
proc = subprocess.Popen([bin_path, "run", "server", "-d", "/opt/clover"])
pid = proc.pid
print(f"PID: {pid}. Logging memory every 60s to {log_file}.")
print("Press Ctrl+C in the morning to stop and view summary.\n")

with open(log_file, "w") as f:
    f.write("timestamp,elapsed_minutes,vm_rss_mb,vm_data_mb,vm_size_mb\n")

start_time = time.time()


def get_mem():
    try:
        with open(f"/proc/{pid}/status") as sf:
            rss, data, size = 0, 0, 0
            for line in sf:
                if line.startswith("VmRSS:"):
                    rss = int(line.split()[1]) / 1024.0
                elif line.startswith("VmData:"):
                    data = int(line.split()[1]) / 1024.0
                elif line.startswith("VmSize:"):
                    size = int(line.split()[1]) / 1024.0
            return rss, data, size
    except Exception:
        return None, None, None


# State tracking for summary calculation
initial_rss = None
initial_data = None
initial_size = None
latest_rss = None
latest_data = None
latest_size = None
peak_rss = 0.0
min_rss = float("inf")
sample_count = 0


def print_summary():
    elapsed_sec = max(time.time() - start_time, 0.1)
    elapsed_hr = elapsed_sec / 3600.0

    print("\n" + "=" * 65)
    print(f"OVERNIGHT TEST SUMMARY ({elapsed_hr:.2f} hours / {elapsed_sec/60.0:.1f} minutes):")
    print(f"  Samples Recorded: {sample_count}")
    if initial_rss is not None and latest_rss is not None:
        rss_drift = latest_rss - initial_rss
        data_drift = (latest_data - initial_data) if (latest_data and initial_data) else 0.0
        hr_div = max(elapsed_hr, 0.001)

        print(f"  Initial RSS:      {initial_rss:.2f} MB")
        print(f"  Final RSS:        {latest_rss:.2f} MB")
        print(f"  Peak RSS:         {peak_rss:.2f} MB")
        print(f"  Min RSS:          {min_rss:.2f} MB")
        print(f"  RSS Drift:        {rss_drift:+.2f} MB ({rss_drift / hr_div:+.2f} MB/hr)")
        if initial_data and latest_data:
            print(f"  Initial Data:     {initial_data:.2f} MB")
            print(f"  Final Data:       {latest_data:.2f} MB")
            print(f"  Data Drift:       {data_drift:+.2f} MB ({data_drift / hr_div:+.2f} MB/hr)")
    else:
        print("  Warning: No valid memory samples were collected.")
    print("=" * 65)
    print(f"Full timeseries saved to {log_file}\n")


def shutdown(sig=None, frame=None):
    global latest_rss, latest_data, latest_size, peak_rss, min_rss

    # Sample memory one last time while process is still running
    if proc.poll() is None:
        final_rss, final_data, final_size = get_mem()
        if final_rss is not None:
            latest_rss, latest_data, latest_size = final_rss, final_data, final_size
            peak_rss = max(peak_rss, latest_rss)
            min_rss = min(min_rss, latest_rss)

    print("\nShutting down clover-hub...")
    if proc.poll() is None:
        proc.send_signal(signal.SIGINT)
        try:
            proc.wait(timeout=10)
        except Exception:
            proc.kill()

    print_summary()
    sys.exit(0)


signal.signal(signal.SIGINT, shutdown)
signal.signal(signal.SIGTERM, shutdown)

# 15s initialization warmup before baseline
print("Warming up for 15s before taking baseline...")
time.sleep(15)

if proc.poll() is not None:
    print(f"Process exited prematurely during warmup with code {proc.returncode}!")
    sys.exit(1)

initial_rss, initial_data, initial_size = get_mem()
if initial_rss is None:
    print("Failed to read initial memory status!")
    shutdown()

latest_rss, latest_data, latest_size = initial_rss, initial_data, initial_size
peak_rss = initial_rss
min_rss = initial_rss
sample_count = 1

now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
elapsed_min = (time.time() - start_time) / 60.0
with open(log_file, "a") as f:
    f.write(f"{now_str},{elapsed_min:.1f},{latest_rss:.2f},{latest_data:.2f},{latest_size:.2f}\n")

print(f"Baseline established: VmRSS={initial_rss:.2f} MB, VmData={initial_data:.2f} MB\n")

try:
    while True:
        time.sleep(60)
        if proc.poll() is not None:
            print(f"\nProcess exited with code {proc.returncode}!")
            break

        rss, data, size = get_mem()
        if rss is None:
            break

        sample_count += 1
        latest_rss, latest_data, latest_size = rss, data, size
        peak_rss = max(peak_rss, rss)
        min_rss = min(min_rss, rss)

        elapsed_min = (time.time() - start_time) / 60.0
        now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        with open(log_file, "a") as f:
            f.write(f"{now_str},{elapsed_min:.1f},{rss:.2f},{data:.2f},{size:.2f}\n")

        print(f"[{now_str}] +{elapsed_min/60.0:4.2f}h | VmRSS={rss:7.2f} MB (diff {rss-initial_rss:+6.2f} MB) | VmData={data:7.2f} MB")
except Exception as e:
    print(f"Loop interrupted: {e}")

shutdown()

