"""
fetch_weather_replay.py - Self-Calibrating "Director's Cut"

SOLUTION: The Jena station is 155m above sea level, so "normal" is ~996 mbar, not 1013.
This script dynamically calculates the local median and sets the pivot accordingly.
"""

import pandas as pd
import json
import os
import zipfile
import requests
import io

# Config
URL = "https://storage.googleapis.com/tensorflow/tf-keras-datasets/jena_climate_2009_2016.csv.zip"
OUTPUT_PATH = "qres-studio/src/lib/weather_data.json"
BLOCK_SIZE = 5000 

def fetch_and_process():
    print(f"📡 Fetching Jena Climate Dataset from {URL}...")
    try:
        r = requests.get(URL)
        r.raise_for_status()
    except Exception as e:
        print(f"❌ Download failed: {e}")
        return

    print("📦 Extracting and processing CSV...")
    try:
        with zipfile.ZipFile(io.BytesIO(r.content)) as z:
            with z.open("jena_climate_2009_2016.csv") as f:
                df = pd.read_csv(f, usecols=["Date Time", "p (mbar)", "T (degC)"])
    except Exception as e:
        print(f"❌ Extraction failed: {e}")
        return
    
    # 1. Determine local "Normal" pressure (elevation-adjusted)
    median_pressure = df['p (mbar)'].median()
    print(f"📍 Local Median Pressure: {median_pressure:.2f} mbar (Jena is 155m elevation)")
    
    # Set "Calm Pivot" slightly above median so ~60% of days = "Calm"
    CALM_PIVOT = median_pressure + 2.0 
    print(f"🎯 Setting Calm Threshold at: {CALM_PIVOT:.2f} mbar (Pressure > This = 0 Vibration)")

    # 2. Find Calm vs Storm Centers
    df['p_smooth'] = df['p (mbar)'].rolling(window=144).mean()
    
    calm_idx = int(df['p_smooth'].idxmax())
    storm_idx = int(df['p_smooth'].idxmin())
    
    print(f"   CALM peak: index {calm_idx} ({df.loc[calm_idx, 'p_smooth']:.2f} mbar)")
    print(f"   STORM peak: index {storm_idx} ({df.loc[storm_idx, 'p_smooth']:.2f} mbar)")
    
    # 3. Extract Blocks (centered on peaks)
    calm_start = max(0, calm_idx - BLOCK_SIZE // 2)
    storm_start = max(0, storm_idx - BLOCK_SIZE // 2)
    
    calm_block = df.iloc[calm_start : calm_start + BLOCK_SIZE].copy()
    storm_block = df.iloc[storm_start : storm_start + BLOCK_SIZE].copy()
    
    director_cut = pd.concat([calm_block, storm_block])
    
    export_data = []
    print("🔄 Mapping Physics to Sensors (using local calibration)...")
    
    for _, row in director_cut.iterrows():
        pressure = float(row["p (mbar)"])
        
        # DYNAMIC MAPPING based on local calibration
        vibration_proxy = max(0.0, (CALM_PIVOT - pressure) * 0.5)
        
        export_data.append({
            "temp": float(row["T (degC)"]),
            "vibration": float(round(vibration_proxy, 3)),
            "pressure_raw": float(round(pressure, 2))
        })
    
    # Verify
    calm_count = len(calm_block)
    calm_frames = sum(1 for d in export_data[:calm_count] if d['vibration'] < 1.0)
    storm_frames = sum(1 for d in export_data[calm_count:] if d['vibration'] > 1.0)
    
    print(f"   ✓ CALM frames (vib < 1.0): {calm_frames}/{calm_count} ({100*calm_frames/calm_count:.1f}%)")
    print(f"   ✓ STORM frames (vib > 1.0): {storm_frames}/{len(export_data)-calm_count}")
    print(f"   First frame: pressure={export_data[0]['pressure_raw']}, vib={export_data[0]['vibration']}")
        
    print(f"💾 Saving {len(export_data)} frames to {OUTPUT_PATH}...")
    
    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)
    with open(OUTPUT_PATH, "w") as f:
        json.dump(export_data, f)
    print("✅ Done! Narrative calibrated to local elevation.")
    print(f"   Total: {len(export_data)} frames (~{len(export_data) // 10 // 60} min at 10Hz)")
    print(f"   Transition at frame: {calm_count}")

if __name__ == "__main__":
    fetch_and_process()
