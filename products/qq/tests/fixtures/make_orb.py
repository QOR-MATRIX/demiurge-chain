"""Write orb.gltf: QQ's test model, a lit sphere with a glowing band.

A glTF 2.0 file with its buffer embedded, written with the standard library only, so the fixture is the same bytes on
every machine and can be regenerated rather than trusted:

    python products/qq/tests/fixtures/make_orb.py

Two meshes: an orange metal-rough sphere (the body), and a thin ring around its equator with an emissive material
(the band), which is what the renderer's bloom has something to bloom from.
"""

import base64
import json
import math
import struct
from pathlib import Path

HERE = Path(__file__).resolve().parent


def sphere(radius, rings, segments):
    positions, normals, indices = [], [], []
    for r in range(rings + 1):
        theta = math.pi * r / rings
        for s in range(segments + 1):
            phi = 2 * math.pi * s / segments
            n = (math.sin(theta) * math.cos(phi), math.cos(theta), math.sin(theta) * math.sin(phi))
            normals.append(n)
            positions.append(tuple(radius * c for c in n))
    for r in range(rings):
        for s in range(segments):
            a = r * (segments + 1) + s
            b = a + segments + 1
            indices += [a, a + 1, b, b, a + 1, b + 1]
    return positions, normals, indices


def ring(radius, tube, segments, sides):
    positions, normals, indices = [], [], []
    for i in range(segments + 1):
        u = 2 * math.pi * i / segments
        cu, su = math.cos(u), math.sin(u)
        for j in range(sides + 1):
            v = 2 * math.pi * j / sides
            cv, sv = math.cos(v), math.sin(v)
            normals.append((cv * cu, sv, cv * su))
            positions.append(((radius + tube * cv) * cu, tube * sv, (radius + tube * cv) * su))
    for i in range(segments):
        for j in range(sides):
            a = i * (sides + 1) + j
            b = a + sides + 1
            indices += [a, b, a + 1, b, b + 1, a + 1]
    return positions, normals, indices


def pack(meshes):
    """One buffer: per mesh, positions, normals, then indices, each 4-byte aligned."""
    blob = bytearray()
    views, accessors, prims = [], [], []
    for positions, normals, indices in meshes:
        attrs = {}
        for name, data in (("POSITION", positions), ("NORMAL", normals)):
            offset = len(blob)
            for v in data:
                blob += struct.pack("<3f", *v)
            views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(blob) - offset, "target": 34962})
            acc = {"bufferView": len(views) - 1, "componentType": 5126, "count": len(data), "type": "VEC3"}
            if name == "POSITION":
                acc["min"] = [min(p[k] for p in data) for k in range(3)]
                acc["max"] = [max(p[k] for p in data) for k in range(3)]
            accessors.append(acc)
            attrs[name] = len(accessors) - 1
        offset = len(blob)
        for i in indices:
            blob += struct.pack("<I", i)
        views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(blob) - offset, "target": 34963})
        accessors.append({"bufferView": len(views) - 1, "componentType": 5125, "count": len(indices), "type": "SCALAR"})
        prims.append({"attributes": attrs, "indices": len(accessors) - 1})
        while len(blob) % 4:
            blob += b"\0"
    return bytes(blob), views, accessors, prims


def main():
    body = sphere(1.0, 32, 48)
    band = ring(1.18, 0.05, 96, 12)
    blob, views, accessors, prims = pack([body, band])
    prims[0]["material"] = 0
    prims[1]["material"] = 1
    gltf = {
        "asset": {"version": "2.0", "generator": "QQ make_orb.py"},
        "scene": 0,
        "scenes": [{"nodes": [0, 1]}],
        "nodes": [{"name": "Body", "mesh": 0}, {"name": "Band", "mesh": 1}],
        "meshes": [{"name": "Body", "primitives": [prims[0]]}, {"name": "Band", "primitives": [prims[1]]}],
        "materials": [
            {
                "name": "Ember",
                "pbrMetallicRoughness": {"baseColorFactor": [1.0, 0.14, 0.01, 1.0], "metallicFactor": 0.15, "roughnessFactor": 0.35},
            },
            {
                "name": "Glow",
                "pbrMetallicRoughness": {"baseColorFactor": [0.02, 0.35, 1.0, 1.0], "metallicFactor": 0.0, "roughnessFactor": 0.5},
                "emissiveFactor": [0.03, 0.45, 1.0],
            },
        ],
        "buffers": [{"byteLength": len(blob), "uri": "data:application/octet-stream;base64," + base64.b64encode(blob).decode()}],
        "bufferViews": views,
        "accessors": accessors,
    }
    (HERE / "orb.gltf").write_text(json.dumps(gltf, indent=1, sort_keys=True) + "\n", encoding="utf-8", newline="\n")


if __name__ == "__main__":
    main()
