from huggingface_hub import hf_hub_download
import shutil
import os

def fetch():
    print("Fetching bert-base-uncased tokenizer...")
    # Download
    local_path = hf_hub_download(repo_id="bert-base-uncased", filename="tokenizer.json")
    
    # Destination
    dest_dir = os.path.join("qres_rust", "assets")
    if not os.path.exists(dest_dir):
        os.makedirs(dest_dir)
        
    dest_path = os.path.join(dest_dir, "tokenizer.json")
    
    # Copy
    shutil.copy(local_path, dest_path)
    print(f"Saved to {dest_path} ({os.path.getsize(dest_path)} bytes)")

if __name__ == "__main__":
    fetch()
