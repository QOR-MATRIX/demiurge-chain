#!/usr/bin/env python3
"""Test auth service."""

import json
import subprocess

# Create proper JSON files
login_data = {"identifier": "test", "password": "test123"}
with open("/tmp/login.json", "w") as f:
    json.dump(login_data, f)

print("Login JSON:", json.dumps(login_data))

# Test local auth
result = subprocess.run([
    "curl", "-s", "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json.dumps(login_data),
    "http://localhost:8080/api/v1/auth/login"
], capture_output=True, text=True)
print("\nLocal auth response:", result.stdout)

# Test via HTTPS
result = subprocess.run([
    "curl", "-s", "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json.dumps(login_data),
    "https://qorsync.dev/api/v1/auth/login"
], capture_output=True, text=True)
print("\nHTTPS auth response:", result.stdout)

# Test RPC
rpc_data = {"jsonrpc": "2.0", "method": "chain_getHealth", "params": [], "id": 1}
result = subprocess.run([
    "curl", "-s", "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json.dumps(rpc_data),
    "http://localhost:9944"
], capture_output=True, text=True)
print("\nLocal RPC response:", result.stdout)

# Test RPC via HTTPS
result = subprocess.run([
    "curl", "-s", "-X", "POST",
    "-H", "Content-Type: application/json",
    "-d", json.dumps(rpc_data),
    "https://rpc.qorsync.dev"
], capture_output=True, text=True)
print("\nHTTPS RPC response:", result.stdout)
