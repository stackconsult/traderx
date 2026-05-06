#!/usr/bin/env python3
"""Test LanceDB installation and basic functionality"""

import lancedb
import numpy as np

# Create LanceDB connection (embedded, no server needed)
db = lancedb.connect("./test_lancedb")

# Create a table with vector data
table = db.create_table(
    "vectors",
    data=[
        {"id": 1, "vector": np.array([0.1, 0.2, 0.3]), "label": "test1"},
        {"id": 2, "vector": np.array([0.4, 0.5, 0.6]), "label": "test2"},
    ],
    mode="overwrite"
)

# Test vector search
query = np.array([0.1, 0.2, 0.3])
results = table.search(query).limit(5).to_pandas()

print("✅ LanceDB test successful!")
print(f"Query result: {results}")
print(f"Table schema: {table.schema}")
