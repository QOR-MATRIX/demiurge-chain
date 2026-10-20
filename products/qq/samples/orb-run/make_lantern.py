"""Writes assets/lantern.gltf: a lantern on a pole, for Orb Run's arena (DIRECTION P3.12).

  Pole    an iron pole 2.4 units tall, skinned to two joints: Root at its foot and Head at 2 units, so the top sways.
  Cage    a gilt base and cap at the head, riding on Head.
  Core    an ember core inside the cage, riding on Head, with a morph target that swells it.

  idle    3 s, looping: the head sways 8 degrees either way while the core breathes in and out.
  flare   0.8 s, once: the core swells to its full size and back, and the head jolts.

One file, its buffer inside it as base64. Standard library only: python make_lantern.py
"""
import base64
import json
import math
import pathlib
import struct

HERE = pathlib.Path(__file__).parent
blob = bytearray()
views = []
accessors = []
FLOAT, USHORT, UBYTE = 5126, 5123, 5121
ARRAY, ELEMENTS = 34962, 34963


def add(data, fmt, count, kind, target=None, minmax=None):
    while len(blob) % 4:
        blob.append(0)
    views.append({"buffer": 0, "byteOffset": len(blob), "byteLength": len(data), **({"target": target} if target else {})})
    blob.extend(data)
    accessor = {"bufferView": len(views) - 1, "componentType": fmt, "count": count, "type": kind}
    if minmax:
        accessor["min"], accessor["max"] = minmax
    accessors.append(accessor)
    return len(accessors) - 1


def floats(values):
    return struct.pack("<%df" % len(values), *values)


def bounds(points):
    return [min(p[i] for p in points) for i in range(3)], [max(p[i] for p in points) for i in range(3)]


def box(cx, cy, cz, hx, hy, hz):
    """A box's eight corners and its twelve triangles, wound outward."""
    points = [(cx + x * hx, cy + y * hy, cz + z * hz) for x in (-1, 1) for y in (-1, 1) for z in (-1, 1)]
    faces = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
    indices = []
    for (a, b, c, d) in faces:
        indices += [a, b, c, a, c, d]
    return points, indices


def mesh(points, indices, extra=None):
    attributes = {"POSITION": add(floats([c for p in points for c in p]), FLOAT, len(points), "VEC3", ARRAY,
                                  bounds(points))}
    index = add(struct.pack("<%dH" % len(indices), *indices), USHORT, len(indices), "SCALAR", ELEMENTS)
    if extra:
        attributes.update(extra)
    return attributes, index


# The pole: eight rings up to 2.4, the top two above the head joint (2.0) skinned to it.
half = 0.05
rings = [0.0, 0.4, 0.8, 1.2, 1.6, 2.0, 2.2, 2.4]
corners = [(-half, -half), (half, -half), (half, half), (-half, half)]
pole_points = [(x, y, z) for y in rings for (x, z) in corners]
pole_indices = []
for ring in range(len(rings) - 1):
    for side in range(4):
        a = ring * 4 + side
        b = ring * 4 + (side + 1) % 4
        pole_indices += [a, b, b + 4, a, b + 4, a + 4]
joints = []
for (x, y, z) in pole_points:
    joints += [1 if y >= 2.0 else 0, 0, 0, 0]
pole_attrs, pole_index = mesh(pole_points, pole_indices, {
    "JOINTS_0": add(struct.pack("<%dB" % len(joints), *joints), UBYTE, len(pole_points), "VEC4", ARRAY),
    "WEIGHTS_0": add(floats([1, 0, 0, 0] * len(pole_points)), FLOAT, len(pole_points), "VEC4", ARRAY),
})
ibm = add(floats([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1] + [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, -2, 0, 1]),
          FLOAT, 2, "MAT4")

# The cage, a base and a cap with the core between them, and the core, about the head (their nodes ride on Head, so
# their points are about its origin).
base_points, base_indices = box(0, 0.02, 0, 0.22, 0.03, 0.22)
cap_points, cap_indices = box(0, 0.5, 0, 0.24, 0.04, 0.24)
cage_points = base_points + cap_points
cage_indices = base_indices + [i + len(base_points) for i in cap_indices]
cage_attrs, cage_index = mesh(cage_points, cage_indices)
core_points, core_indices = box(0, 0.25, 0, 0.13, 0.17, 0.13)
core_delta = [c * 0.6 for p in box(0, 0, 0, 0.13, 0.17, 0.13)[0] for c in p]
core_attrs, core_index = mesh(core_points, core_indices)
core_target = add(floats(core_delta), FLOAT, 8, "VEC3", None,
                  bounds([tuple(core_delta[i:i + 3]) for i in range(0, 24, 3)]))


def turn(degrees):
    a = math.radians(degrees) / 2
    return [0, 0, math.sin(a), math.cos(a)]


idle_times = add(floats([0, 0.75, 1.5, 2.25, 3.0]), FLOAT, 5, "SCALAR", None, ([0.0], [3.0]))
idle_turns = add(floats(turn(0) + turn(8) + turn(0) + turn(-8) + turn(0)), FLOAT, 5, "VEC4")
idle_weights = add(floats([0.0, 0.5, 1.0, 0.5, 0.0]), FLOAT, 5, "SCALAR")
flare_times = add(floats([0, 0.2, 0.8]), FLOAT, 3, "SCALAR", None, ([0.0], [0.8]))
flare_turns = add(floats(turn(0) + turn(-5) + turn(0)), FLOAT, 3, "VEC4")
flare_weights = add(floats([0.0, 1.6, 0.0]), FLOAT, 3, "SCALAR")

gltf = {
    "asset": {"version": "2.0", "generator": "QQ make_lantern.py"},
    "scene": 0,
    "scenes": [{"nodes": [0]}],
    "nodes": [
        {"name": "Lantern", "children": [1, 3]},
        {"name": "Root", "children": [2]},
        {"name": "Head", "translation": [0, 2.0, 0], "children": [4, 5]},
        {"name": "Pole", "mesh": 0, "skin": 0},
        {"name": "Cage", "mesh": 1},
        {"name": "Core", "mesh": 2},
    ],
    "skins": [{"name": "Sway", "joints": [1, 2], "inverseBindMatrices": ibm, "skeleton": 1}],
    "meshes": [
        {"name": "Pole", "primitives": [{"attributes": pole_attrs, "indices": pole_index, "material": 0}]},
        {"name": "Cage", "primitives": [{"attributes": cage_attrs, "indices": cage_index, "material": 1}]},
        {"name": "Core", "weights": [0.0], "extras": {"targetNames": ["swell"]},
         "primitives": [{"attributes": core_attrs, "indices": core_index, "material": 2,
                         "targets": [{"POSITION": core_target}]}]},
    ],
    "materials": [
        {"name": "Iron", "pbrMetallicRoughness": {"baseColorFactor": [0.09, 0.1, 0.13, 1], "metallicFactor": 0.6,
                                                  "roughnessFactor": 0.5}},
        {"name": "Gilt", "pbrMetallicRoughness": {"baseColorFactor": [0.6, 0.45, 0.2, 1], "metallicFactor": 0.9,
                                                  "roughnessFactor": 0.35}},
        {"name": "Ember", "emissiveFactor": [1.0, 0.42, 0.0],
         "pbrMetallicRoughness": {"baseColorFactor": [1.0, 0.42, 0.0, 1], "metallicFactor": 0.0, "roughnessFactor": 0.4}},
    ],
    "animations": [
        {"name": "idle",
         "channels": [{"sampler": 0, "target": {"node": 2, "path": "rotation"}},
                      {"sampler": 1, "target": {"node": 5, "path": "weights"}}],
         "samplers": [{"input": idle_times, "output": idle_turns, "interpolation": "LINEAR"},
                      {"input": idle_times, "output": idle_weights, "interpolation": "LINEAR"}]},
        {"name": "flare",
         "channels": [{"sampler": 0, "target": {"node": 2, "path": "rotation"}},
                      {"sampler": 1, "target": {"node": 5, "path": "weights"}}],
         "samplers": [{"input": flare_times, "output": flare_turns, "interpolation": "LINEAR"},
                      {"input": flare_times, "output": flare_weights, "interpolation": "LINEAR"}]},
    ],
    "accessors": accessors,
    "bufferViews": views,
    "buffers": [{"byteLength": len(blob),
                 "uri": "data:application/octet-stream;base64," + base64.b64encode(bytes(blob)).decode("ascii")}],
}
(HERE / "assets").mkdir(exist_ok=True)
(HERE / "assets" / "lantern.gltf").write_text(json.dumps(gltf, indent=1) + "\n", encoding="utf-8", newline="\n")
