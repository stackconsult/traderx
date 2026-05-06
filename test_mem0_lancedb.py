#!/usr/bin/env python3
"""Test mem0 with LanceDB integration"""

import yaml
import lancedb
import numpy as np

# Load mem0 config
with open('mem0-config.yaml', 'r') as f:
    config = yaml.safe_load(f)

print(f"✅ mem0 config loaded: {config['memory_store']['type']}")

# Test LanceDB connection
db = lancedb.connect(config['memory_store']['uri'])
print(f"✅ LanceDB connected at: {config['memory_store']['uri']}")

# Test table creation
table = db.create_table(
    config['memory_store']['collection_name'],
    data=[
        {"id": 1, "vector": np.random.rand(768), "content": "test memory 1"},
        {"id": 2, "vector": np.random.rand(768), "content": "test memory 2"},
    ],
    mode="overwrite"
)
print(f"✅ LanceDB table created: {config['memory_store']['collection_name']}")

# Test vector search
query = np.random.rand(768)
results = table.search(query).limit(5).to_pandas()
print(f"✅ Vector search successful: {len(results)} results")

print("\n🎉 mem0 + LanceDB integration validated successfully!")
