"""
TurboQuant - Context Compression for Multi-Model Handoff
Compresses conversation history to maintain 100k+ token context without cloud costs.
"""

import numpy as np
import json
import hashlib
from typing import List, Dict, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from datetime import datetime
import pickle
import zlib
from sentence_transformers import SentenceTransformer
import faiss
import logging

logger = logging.getLogger(__name__)


@dataclass
class EmbeddingVector:
    """Embedding vector with metadata."""
    text: str
    embedding: np.ndarray
    timestamp: datetime
    importance: float  # 0.0 to 1.0
    metadata: Dict[str, Any]
    hash: str = ""
    
    def __post_init__(self):
        """Generate hash for text."""
        if not self.hash:
            self.hash = hashlib.sha256(self.text.encode()).hexdigest()[:16]


@dataclass
class CompressionMetrics:
    """Metrics for compression performance."""
    original_size: int
    compressed_size: int
    compression_ratio: float
    tokens_saved: int
    processing_time_ms: float
    quality_score: float  # Semantic similarity score


class TurboQuant:
    """
    TurboQuant compression system for maintaining large context windows.
    Uses semantic embeddings and importance scoring to compress conversation history.
    """
    
    def __init__(self, model_name: str = "all-MiniLM-L6-v2", max_context_tokens: int = 100000):
        self.model_name = model_name
        self.max_context_tokens = max_context_tokens
        self.avg_tokens_per_word = 1.3  # Approximate token to word ratio
        
        # Initialize embedding model
        try:
            self.model = SentenceTransformer(model_name)
            self.embedding_dim = self.model.get_sentence_embedding_dimension()
        except Exception as e:
            logger.error(f"Failed to load embedding model: {e}")
            raise
        
        # FAISS index for similarity search
        self.index = faiss.IndexFlatIP(self.embedding_dim)
        self.embeddings: List[EmbeddingVector] = []
        
        # Compression settings
        self.min_importance_threshold = 0.3
        self.max_compression_ratio = 0.1  # Compress to 10% of original
        
    def embed_text(self, texts: List[str]) -> np.ndarray:
        """Generate embeddings for a list of texts."""
        return self.model.encode(texts, convert_to_numpy=True)
    
    def calculate_importance(self, text: str, context: Dict[str, Any]) -> float:
        """
        Calculate importance score for a piece of text.
        
        Args:
            text: Text to score
            context: Additional context (role, timestamp, etc.)
            
        Returns:
            Importance score between 0.0 and 1.0
        """
        score = 0.5  # Base score
        
        # Role-based importance
        role = context.get("role", "")
        if role == "system":
            score += 0.3
        elif role == "user":
            score += 0.2
        elif role == "assistant":
            score += 0.1
        
        # Length-based importance (longer messages might be more important)
        word_count = len(text.split())
        if word_count > 100:
            score += 0.1
        elif word_count < 10:
            score -= 0.1
        
        # Keyword-based importance
        important_keywords = [
            "decision", "conclusion", "summary", "plan", "execute",
            "critical", "error", "warning", "important", "note"
        ]
        for keyword in important_keywords:
            if keyword.lower() in text.lower():
                score += 0.05
        
        # Recency importance (newer messages slightly more important)
        if "timestamp" in context:
            age_hours = (datetime.utcnow() - context["timestamp"]).total_seconds() / 3600
            if age_hours < 1:
                score += 0.1
            elif age_hours > 24:
                score -= 0.1
        
        # Clamp between 0 and 1
        return max(0.0, min(1.0, score))
    
    def add_embeddings(self, texts: List[str], contexts: List[Dict[str, Any]]) -> List[str]:
        """
        Add text embeddings to the compression system.
        
        Args:
            texts: List of texts to embed
            contexts: List of context dictionaries for each text
            
        Returns:
            List of embedding hashes
        """
        if len(texts) != len(contexts):
            raise ValueError("Texts and contexts must have same length")
        
        # Generate embeddings
        embeddings = self.embed_text(texts)
        
        # Create embedding vectors
        hashes = []
        for text, embedding, context in zip(texts, embeddings, contexts):
            importance = self.calculate_importance(text, context)
            
            vector = EmbeddingVector(
                text=text,
                embedding=embedding,
                timestamp=context.get("timestamp", datetime.utcnow()),
                importance=importance,
                metadata=context
            )
            
            # Add to index
            self.embeddings.append(vector)
            self.index.add(embedding.reshape(1, -1))
            hashes.append(vector.hash)
        
        return hashes
    
    def compress_context(self, 
                        messages: List[Dict[str, Any]], 
                        target_ratio: Optional[float] = None) -> Tuple[List[Dict[str, Any]], CompressionMetrics]:
        """
        Compress conversation context while preserving important information.
        
        Args:
            messages: List of message dictionaries with 'role', 'content', 'timestamp'
            target_ratio: Target compression ratio (default: max_compression_ratio)
            
        Returns:
            Tuple of (compressed_messages, metrics)
        """
        start_time = datetime.utcnow()
        
        # Calculate target ratio
        if target_ratio is None:
            target_ratio = self.max_compression_ratio
        
        # Extract texts and contexts
        texts = [msg["content"] for msg in messages]
        contexts = [
            {
                "role": msg.get("role"),
                "timestamp": datetime.fromisoformat(msg["timestamp"]) if isinstance(msg.get("timestamp"), str) else msg.get("timestamp", datetime.utcnow()),
                "metadata": {k: v for k, v in msg.items() if k not in ["role", "content", "timestamp"]}
            }
            for msg in messages
        ]
        
        # Add embeddings
        self.add_embeddings(texts, contexts)
        
        # Calculate target size
        original_size = len(messages)
        target_size = max(1, int(original_size * target_ratio))
        
        # Select messages to keep
        selected_indices = self._select_important_messages(target_size)
        
        # Build compressed context
        compressed_messages = [messages[i] for i in selected_indices]
        
        # Calculate metrics
        processing_time = (datetime.utcnow() - start_time).total_seconds() * 1000
        compressed_size = len(compressed_messages)
        compression_ratio = compressed_size / original_size
        
        # Estimate tokens saved
        original_tokens = sum(len(msg["content"].split()) * self.avg_tokens_per_word for msg in messages)
        compressed_tokens = sum(len(msg["content"].split()) * self.avg_tokens_per_word for msg in compressed_messages)
        tokens_saved = original_tokens - compressed_tokens
        
        # Calculate quality score (semantic similarity)
        quality_score = self._calculate_compression_quality(messages, compressed_messages)
        
        metrics = CompressionMetrics(
            original_size=original_size,
            compressed_size=compressed_size,
            compression_ratio=compression_ratio,
            tokens_saved=int(tokens_saved),
            processing_time_ms=processing_time,
            quality_score=quality_score
        )
        
        return compressed_messages, metrics
    
    def _select_important_messages(self, target_size: int) -> List[int]:
        """Select indices of most important messages using various strategies."""
        if len(self.embeddings) <= target_size:
            return list(range(len(self.embeddings)))
        
        # Strategy 1: Keep all high importance messages
        high_importance = [
            i for i, emb in enumerate(self.embeddings)
            if emb.importance >= self.min_importance_threshold
        ]
        
        # If already at target, return
        if len(high_importance) >= target_size:
            return high_importance[:target_size]
        
        # Strategy 2: Add diverse messages using clustering
        remaining_slots = target_size - len(high_importance)
        if remaining_slots > 0:
            diverse_indices = self._select_diverse_messages(
                remaining_slots, exclude_indices=high_importance
            )
            high_importance.extend(diverse_indices)
        
        return high_importance[:target_size]
    
    def _select_diverse_messages(self, count: int, exclude_indices: List[int]) -> List[int]:
        """Select diverse messages using k-means clustering on embeddings."""
        if count <= 0:
            return []
        
        # Get available indices
        available = [i for i in range(len(self.embeddings)) if i not in exclude_indices]
        
        if len(available) <= count:
            return available
        
        # Extract embeddings for available messages
        available_embeddings = np.array([self.embeddings[i].embedding for i in available])
        
        # Simple k-means using FAISS
        k = min(count, len(available))
        kmeans = faiss.Kmeans(d=self.embedding_dim, k=k, niter=20)
        kmeans.train(available_embeddings)
        
        # Find closest to each centroid
        _, assignments = kmeans.index.search(available_embeddings, 1)
        
        # Select one message per cluster
        selected = []
        for cluster_id in range(k):
            cluster_indices = [i for i, a in enumerate(assignments) if a == cluster_id]
            if cluster_indices:
                # Select the most important from the cluster
                cluster_available = [available[i] for i in cluster_indices]
                cluster_importance = [self.embeddings[i].importance for i in cluster_available]
                best_idx = cluster_available[np.argmax(cluster_importance)]
                selected.append(best_idx)
        
        return selected[:count]
    
    def _calculate_compression_quality(self, original: List[Dict[str, Any]], compressed: List[Dict[str, Any]]) -> float:
        """Calculate semantic similarity between original and compressed context."""
        if not compressed:
            return 0.0
        
        # Get embeddings for original and compressed
        original_texts = [msg["content"] for msg in original]
        compressed_texts = [msg["content"] for msg in compressed]
        
        original_emb = self.embed_text(original_texts)
        compressed_emb = self.embed_text(compressed_texts)
        
        # Calculate average similarity to compressed set
        similarities = []
        for o_emb in original_emb:
            # Find most similar in compressed
            sims = np.dot(o_emb, compressed_emb.T)
            similarities.append(np.max(sims))
        
        return float(np.mean(similarities))
    
    def save_state(self, filepath: str):
        """Save compression state to disk."""
        state = {
            "embeddings": [
                {
                    "text": emb.text,
                    "embedding": emb.embedding.tolist(),
                    "timestamp": emb.timestamp.isoformat(),
                    "importance": emb.importance,
                    "metadata": emb.metadata,
                    "hash": emb.hash
                }
                for emb in self.embeddings
            ],
            "model_name": self.model_name,
            "max_context_tokens": self.max_context_tokens
        }
        
        # Compress with zlib
        compressed = zlib.compress(pickle.dumps(state))
        
        with open(filepath, "wb") as f:
            f.write(compressed)
    
    def load_state(self, filepath: str):
        """Load compression state from disk."""
        with open(filepath, "rb") as f:
            compressed = f.read()
        
        state = pickle.loads(zlib.decompress(compressed))
        
        # Restore embeddings
        self.embeddings = []
        for emb_data in state["embeddings"]:
            vector = EmbeddingVector(
                text=emb_data["text"],
                embedding=np.array(emb_data["embedding"]),
                timestamp=datetime.fromisoformat(emb_data["timestamp"]),
                importance=emb_data["importance"],
                metadata=emb_data["metadata"],
                hash=emb_data["hash"]
            )
            self.embeddings.append(vector)
        
        # Rebuild FAISS index
        self.index = faiss.IndexFlatIP(self.embedding_dim)
        if self.embeddings:
            embeddings_array = np.array([emb.embedding for emb in self.embeddings])
            self.index.add(embeddings_array)
    
    def clear(self):
        """Clear all embeddings and reset state."""
        self.embeddings = []
        self.index = faiss.IndexFlatIP(self.embedding_dim)
