import os
from typing import List, Union, Optional, Tuple
import networkx as nx
import numpy as np

try:
    import torch
    import torch.nn.functional as F
    from sentence_transformers import SentenceTransformer
    import open_clip
    from PIL import Image
    MULTIMODAL_AVAILABLE = True
except ImportError:
    MULTIMODAL_AVAILABLE = False
    print("[QRES-MM] Multi-modal dependencies not found. Operating in fallback mode.")

class MultiModalMemory:
    """
    QRES v7.0 Multi-Modal Graph Memory.
    Uses CLIP and Sentence-Transformers to create a semantic graph of data chunks.
    """
    def __init__(self, model_name: str = "all-MiniLM-L6-v2", clip_model: str = "ViT-B-32"):
        self.graph = nx.Graph()
        self.text_model = None
        self.clip_model = None
        self.clip_preprocess = None
        self.clip_tokenizer = None
        self.device = "cpu"
        
        if MULTIMODAL_AVAILABLE:
            self.device = "cuda" if torch.cuda.is_available() else "cpu"
            print(f"[QRES-MM] Initializing Multi-Modal Memory on {self.device}...")
            
            # Load Text Model
            try:
                self.text_model = SentenceTransformer(model_name, device=self.device)
                print(f"[QRES-MM] Text model '{model_name}' loaded.")
            except Exception as e:
                print(f"[QRES-MM] Failed to load text model: {e}")

            # Load CLIP Model
            try:
                # Use smaller model or just check if we need it
                self.clip_model, _, self.clip_preprocess = open_clip.create_model_and_transforms(clip_model, pretrained='laion2b_s34b_b79k', device=self.device)
                self.clip_tokenizer = open_clip.get_tokenizer(clip_model)
                print(f"[QRES-MM] CLIP model '{clip_model}' loaded.")
            except Exception as e:
                print(f"[QRES-MM] Warning: Failed to load CLIP model. Image features disabled. ({e})")

    def add_text_node(self, node_id: str, text: str):
        """Adds a text node to the graph with its embedding."""
        if not self.text_model:
            return
        
        embedding = self.text_model.encode(text, convert_to_tensor=True)
        self.graph.add_node(node_id, type="text", content=text[:50], embedding=embedding)
        self._link_node(node_id, embedding)

    def add_image_node(self, node_id: str, image_path: str):
        """Adds an image node to the graph with its embedding."""
        if not self.clip_model or not os.path.exists(image_path):
            return
        
        try:
            image = self.clip_preprocess(Image.open(image_path)).unsqueeze(0).to(self.device)
            with torch.no_grad():
                embedding = self.clip_model.encode_image(image)
                embedding /= embedding.norm(dim=-1, keepdim=True)
            
            self.graph.add_node(node_id, type="image", path=image_path, embedding=embedding.squeeze())
            self._link_node(node_id, embedding.squeeze())
        except Exception as e:
            print(f"[QRES-MM] Error adding image node: {e}")

    def _link_node(self, node_id: str, embedding: torch.Tensor, threshold: float = 0.5):
        """Links the new node to existing nodes based on cosine similarity."""
        for other_id, data in self.graph.nodes(data=True):
            if other_id == node_id:
                continue
            
            other_emb = data.get("embedding")
            if other_emb is not None:
                sim = F.cosine_similarity(embedding.unsqueeze(0), other_emb.reshape(1, -1)).item()
                if sim > threshold:
                    self.graph.add_edge(node_id, other_id, weight=sim)

    def find_related(self, query_text: str, top_k: int = 3) -> List[Tuple[str, float]]:
        """Finds nodes related to the query text."""
        if not self.text_model:
            return []
            
        query_emb = self.text_model.encode(query_text, convert_to_tensor=True)
        results = []
        
        for node_id, data in self.graph.nodes(data=True):
            node_emb = data.get("embedding")
            if node_emb is not None:
                # Ensure dimensions match (CLIP embeddings might need projection if comparing cross-model directly without alignment)
                # For v7.0 Foundations, we assume text-to-text or CLIP text-to-image queries. 
                # If searching images with text, we should use CLIP text encoder.
                
                # Check node type
                if data['type'] == 'image' and self.clip_model:
                     # Use CLIP text encoder for query
                     with torch.no_grad():
                        clip_tokens = self.clip_tokenizer([query_text]).to(self.device)
                        clip_query_emb = self.clip_model.encode_text(clip_tokens)
                        clip_query_emb /= clip_query_emb.norm(dim=-1, keepdim=True)
                        sim = F.cosine_similarity(clip_query_emb, node_emb.unsqueeze(0)).item()
                else: 
                    # Standard text similarity
                    sim = F.cosine_similarity(query_emb.unsqueeze(0), node_emb.reshape(1, -1)).item()
                
                results.append((node_id, sim))
        
        results.sort(key=lambda x: x[1], reverse=True)
        return results[:top_k]

    def get_stats(self):
        return {
            "num_nodes": self.graph.number_of_nodes(),
            "num_edges": self.graph.number_of_edges(),
            "density": nx.density(self.graph)
        }

    def export_json(self, path: str):
        """Exports the graph structure (nodes/edges) to a JSON file for visualization."""
        import json
        
        data = {
            "nodes": [],
            "edges": []
        }
        
        for node_id, attrs in self.graph.nodes(data=True):
            # Convert embedding to list if present (too large for visualization, maybe skip or truncate)
            # For XAI, we just need the structure and content metadata
            clean_attrs = attrs.copy()
            if "embedding" in clean_attrs:
                del clean_attrs["embedding"]
            
            clean_attrs["id"] = node_id
            data["nodes"].append(clean_attrs)
            
        for u, v, attrs in self.graph.edges(data=True):
            edge_data = attrs.copy()
            edge_data["source"] = u
            edge_data["target"] = v
            data["edges"].append(edge_data)
            
        try:
            with open(path, "w") as f:
                json.dump(data, f, indent=2)
            print(f"[QRES-MM] Exported graph to {path}")
        except Exception as e:
            print(f"[QRES-MM] Export failed: {e}")

    def detect_bias(self, threshold: float = 0.5) -> bool:
        """
        [Ethical Pruning] Detects statistical bias in edge weights (skewed distribution).
        Uses Gini coefficient to measure inequality in relation strength.
        """
        weights = [d['weight'] for _, _, d in self.graph.edges(data=True)]
        if not weights:
            return False
            
        weights = np.array(weights)
        # Gini coefficient calculation
        sorted_weights = np.sort(weights)
        n = len(weights)
        cumulative_weights = np.cumsum(sorted_weights)
        _sum = cumulative_weights[-1]
        
        # Gini Formula: (2 * sum(i * xi) - (n+1) * sum(xi)) / (n * sum(xi))
        # Or standard area-based:
        index = np.arange(1, n + 1)
        gini = (2 * np.sum(index * sorted_weights) - (n + 1) * _sum) / (n * _sum)
        
        print(f"[Ethical Pruning] Gini Coefficient: {gini:.4f}")
        
        if gini > threshold:
            print("⚠️  Bias detected: High skew in relations. Pruning/Debiasing...")
            median_weight = np.median(weights)
            
            # Simple Debias: Cap excessively strong edges relative to median (Ethical flattening)
            pruned_count = 0
            for u, v, d in self.graph.edges(data=True):
                if d['weight'] > median_weight * 2.0:
                    d['weight'] *= 0.8 # Decay outlier
                    d['decayed'] = True # Mark for XAI
                    pruned_count += 1
                    
            print(f"🔧 Pruned {pruned_count} biased edges.")
            return True
        else:
            print("✅ Graph edge distribution fits ethical guidelines.")
            return False

if __name__ == "__main__":
    print("Testing MultiModalMemory Class...")
    mm = MultiModalMemory()
    mm.add_text_node("doc1", "The quick brown fox jumps over the lazy dog.")
    mm.add_text_node("doc2", "A fast russet canine leaps over a sluggish hound.")
    mm.add_text_node("doc3", "Quantum mechanics describes the physical properties of nature.")
    
    # Simulate heavy bias to trigger ethical pruning
    mm.add_text_node("bias_node", "bias")
    mm.graph.add_edge("doc1", "bias_node", weight=0.99) # Artificial strong link
    
    print("Stats:", mm.get_stats())
    
    # Run Ethical Check
    mm.detect_bias(threshold=0.4)
    
    # Export for XAI
    mm.export_json("assets/knowledge_graph.json")
    
    print("\nQuery: 'dog running'")
    results = mm.find_related("dog running")
    for nid, score in results:
        print(f" - {nid}: {score:.4f}")
