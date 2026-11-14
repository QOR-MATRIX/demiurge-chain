#!/usr/bin/env python3
"""Test consensus RPC methods."""

import json
import subprocess

# Test consensus_getStatus
rpc_data = {"jsonrpc": "2.0", "method": "consensus_getStatus", "params": [], "id": 1}
result = subprocess.run([
    "curl", "-s", "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json.dumps(rpc_data),
    "http://localhost:9944"
], capture_output=True, text=True)
print("consensus_getStatus (local):")
print(result.stdout)

# Test via HTTPS
result = subprocess.run([
    "curl", "-s", "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json.dumps(rpc_data),
    "https://rpc.qorsync.dev"
], capture_output=True, text=True)
print("\nconsensus_getStatus (HTTPS):")
print(result.stdout)

# Test chain_getHealth for comparison
rpc_data = {"jsonrpc": "2.0", "method": "chain_getHealth", "params": [], "id": 2}
result = subprocess.run([
    "curl", "-s", "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json.dumps(rpc_data),
    "https://rpc.qorsync.dev"
], capture_output=True, text=True)
print("\nchain_getHealth (HTTPS):")
print(result.stdout)
