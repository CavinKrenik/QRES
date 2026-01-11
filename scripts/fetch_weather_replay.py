import pandas as pd
import json
import os
import zipfile
import requests
import io

# Config
URL = "https://storage.googleapis.com/tensorflow/tf-keras-datasets/jena_climate_2009_2016.csv.zip"
OUTPUT_PATH = "qres-studio/src/lib/weather_data.json"
CHUNK_SIZE = 1000  # ~1 week of data at 10-min intervals

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

    # Select window with "Storm" dynamics (index 10000 has good variance)
    start_idx = 10000
    subset = df.iloc[start_idx : start_idx + CHUNK_SIZE]

    export_data = []
    print("🔄 Mapping Physics to Sensors...")
    for _, row in subset.iterrows():
        pressure = float(row["p (mbar)"])
        # Normal pressure ~1013 mbar. Drops below 1000 = storms.
        # Map (1005 - pressure) * 0.5 to vibration.
        # Storm at 980 mbar -> vibration ~12.5
        # Calm at 1020 mbar -> vibration 0
        vibration_proxy = max(0.0, (1005.0 - pressure) * 0.5)

        export_data.append({
            "temp": float(row["T (degC)"]),
            "vibration": round(vibration_proxy, 4),
            "pressure_raw": pressure
        })

    print(f"💾 Saving {len(export_data)} frames to {OUTPUT_PATH}...")
    os.makedirs(os.path.dirname(OUTPUT_PATH), exist_ok=True)

    with open(OUTPUT_PATH, "w") as f:
        json.dump(export_data, f, indent=2)

    print("✅ Done! Weather data ready for replay.")
    
    # Print sample data for verification
    print("\n📊 Sample Data Preview:")
    print(f"  Frame 0 (start):   temp={export_data[0]['temp']:.1f}°C, pressure={export_data[0]['pressure_raw']:.1f}mbar, vibration={export_data[0]['vibration']:.2f}")
    print(f"  Frame 500 (mid):   temp={export_data[500]['temp']:.1f}°C, pressure={export_data[500]['pressure_raw']:.1f}mbar, vibration={export_data[500]['vibration']:.2f}")
    print(f"  Frame 999 (end):   temp={export_data[999]['temp']:.1f}°C, pressure={export_data[999]['pressure_raw']:.1f}mbar, vibration={export_data[999]['vibration']:.2f}")

if __name__ == "__main__":
    fetch_and_process()
