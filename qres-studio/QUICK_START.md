# QRES Studio - Quick Start Guide

## 🎯 What You're Looking At

QRES Studio is a **futuristic GUI** for the QRES compression system with **AI integration**. The app is currently running in development mode!

## 🖥️ Interface Overview

### Top Bar
- **💾 Saved**: Total megabytes saved through compression
- **📦 Files**: Number of files processed

### Three Main Tabs

#### 1️⃣ Drop Zone (Compression)
**What it does**: Compress or decompress files with real-time visualization

**How to use**:
1. Drag any file onto the circular drop zone
2. Watch the progress ring fill up (color shows active engine)
3. Choose where to save the `.qres` compressed file
4. See real-time stats: progress %, engine type, compression ratio

**Engine Colors**:
- 🟡 **Gold** = ZSTD (fallback for random data)
- 🔵 **Blue** = LINEAR (simple patterns)
- 🟢 **Green** = IPEPS (tensor networks)
- 🟣 **Purple** = LSTM (neural predictor)

#### 2️⃣ Hive Mind (Analytics)
**What it does**: Shows compression statistics and swarm network status

**Features**:
- **Swarm Toggle**: Simulate connecting to the distributed Hive network
- **Hive Wisdom**: Average compression ratio across all operations
- **Engine Usage Chart**: Bar chart showing which engines are used most
- **Stats Cards**: Total compressions, bytes saved, average ratio

#### 3️⃣ AI Gen (Ollama Integration) ⭐ NEW!
**What it does**: Use local AI (Llama3) for data generation and analysis

**How to use**:
1. Make sure Ollama is running (`ollama serve`)
2. Type a prompt in the text area
3. Click "Query" or press **Ctrl+Enter**
4. Wait for the AI response

**Example Prompts**:
```
Generate 100 structured log lines for training data

Analyze this compression ratio pattern and suggest optimizations

Create 50 text snippets with varying entropy levels

Explain why LSTM performs better on time-series data
```

## 🚀 Getting Started

### First Time Setup
```bash
# 1. Install Ollama (optional, for AI features)
# Download from: https://ollama.com

# 2. Pull a model
ollama pull llama3

# 3. Start Ollama server
ollama serve

# 4. In another terminal, run QRES Studio
cd qres-studio
npm run tauri dev
```

### Quick Test
1. **Test Compression**: Drag a text file onto the Drop Zone
2. **Check Analytics**: Switch to Hive Mind tab to see stats
3. **Try AI**: Go to AI Gen tab and ask: "What is QRES?"

## 🎨 Visual Guide

### Drop Zone States
- **Idle**: Dashed circle, "Drop file here" text
- **Dragging**: Circle glows blue, scales up slightly
- **Processing**: Progress ring fills, shows % and active engine
- **Complete**: Resets to idle, stats bar updates

### Hive Mind Dashboard
- **Top Section**: Swarm toggle and Hive Wisdom percentage
- **Middle**: Three stat cards (compressions, saved, ratio)
- **Bottom**: Bar chart of engine usage

### AI Gen Interface
- **Input**: Large textarea for prompts
- **Status**: Shows "Ready", "Querying...", or errors
- **Output**: Formatted response in code block

## 🔧 Troubleshooting

### "Is Ollama running?" Error
**Solution**: 
```bash
# Start Ollama in a terminal
ollama serve
```

### Compression Not Working
**Check**:
- Is the file path accessible?
- Do you have write permissions for the destination?
- Check the terminal for Rust backend errors

### Stats Not Updating
**Note**: Current stats are placeholder data. Real integration with qres_rust coming soon!

## 💡 Pro Tips

### Keyboard Shortcuts
- **Ctrl+Enter** in AI Gen: Submit prompt
- **Tab**: Navigate between tabs (use mouse for now)

### Best Practices
1. **Compress Similar Files Together**: Helps the Hive learn patterns
2. **Use AI for Data Gen**: Generate training data for meta-brain
3. **Monitor Engine Usage**: See which predictor works best for your data

### Performance
- **Drop Zone**: Real-time updates via Tauri events
- **AI Gen**: Response time depends on Ollama model (2-10s typical)
- **Hive Mind**: Charts update instantly

## 🎯 Common Workflows

### Workflow 1: Compress a Dataset
```
1. Open Drop Zone tab
2. Drag first file → observe which engine is used
3. Repeat for similar files
4. Check Hive Mind to see engine distribution
```

### Workflow 2: Generate Training Data
```
1. Open AI Gen tab
2. Prompt: "Generate 100 JSON objects with random sensor data"
3. Copy response
4. Save to file for training meta-brain
```

### Workflow 3: Analyze Performance
```
1. Compress several files in Drop Zone
2. Switch to Hive Mind
3. Open AI Gen
4. Prompt: "Based on these stats, how can I improve compression?"
```

## 📚 Technical Details

### Architecture
- **Frontend**: Svelte (reactive UI)
- **Backend**: Rust via Tauri (secure, fast)
- **Compression**: qres_rust library (neural predictors)
- **AI**: Ollama (local LLM, no cloud)

### Data Flow
```
User Action → Svelte Component → Tauri Command → Rust Backend
                                                      ↓
                                            qres_rust / Ollama
                                                      ↓
                                              Tauri Event ← ←
                                                      ↓
                                            Svelte Update
```

### Security
- All file operations use Tauri's secure dialog
- Ollama runs locally (no data sent to cloud)
- No API keys or external services required

## 🎨 Customization

### Changing Ollama Model
Edit `src-tauri/src/commands.rs`:
```rust
.json(&serde_json::json!({
    "model": "phi",  // Change from "llama3"
    "prompt": prompt,
    "stream": false
}))
```

### Adjusting Theme Colors
Edit `src/App.svelte`:
```css
background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 100%);
```

## 🐛 Known Issues
1. Decompression not yet implemented (shows alert)
2. Stats are placeholder data (not connected to real qres_rust stats)
3. Training integration (`run_training` command) not yet added

## 🚀 Next Steps
- Try compressing different file types
- Experiment with AI prompts
- Monitor which engines work best for your data
- Provide feedback for improvements!

---

**Need Help?** Check the main README or open an issue on GitHub.
**Version**: 0.1.0 | **Last Updated**: Dec 30, 2025
