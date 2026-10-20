"""Writes rig.gltf: a model that moves by its own animations, for the animation checks (P3.12).

  Bar     a bar two units tall, skinned to two joints: Root at its foot and Tip half way up. Below the middle its
          vertices follow Root, above it Tip.
  Puff    a box with one morph target that doubles its size.

  bend    an animation of 2 s: Tip turns from upright to 90 degrees about z at 1 s, and back at 2 s.
  puff    an animation of 1 s: Puff's morph weight goes from 0 to 1.

Everything is in one file, its buffer inside it as base64. Standard library only: python make_rig.py
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


def add(data, fmt, count, kind, target=None, minmax=None):
    """Append packed data as a buffer view and an accessor; its accessor index."""
    while len(blob) % 4:
        blob.append(0)
    offset = len(blob)
    blob.extend(data)
    view = {"buffer": 0, "byteOffset": offset, "byteLength": len(data)}
    if target:
        view["target"] = target
    views.append(view)
    accessor = {"bufferView": len(views) - 1, "componentType": fmt, "count": count, "type": kind}
    if minmax:
        accessor["min"], accessor["max"] = minmax
    accessors.append(accessor)
    return len(accessors) - 1


FLOAT, USHORT, UBYTE = 5126, 5123, 5121
ARRAY, ELEMENTS = 34962, 34963


def floats(values):
    return struct.pack("<%df" % len(values), *values)


def bounds(points):
    return [min(p[i] for p in points) for i in range(3)], [max(p[i] for p in points) for i in range(3)]


# ── the bar: four rings of four corners, from y 0 to 2 ──
half = 0.1
rings = [0.0, 2 / 3, 4 / 3, 2.0]
corners = [(-half, -half), (half, -half), (half, half), (-half, half)]
bar_points = [(x, y, z) for y in rings for (x, z) in corners]
bar_indices = []
for ring in range(len(rings) - 1):
    for side in range(4):
        a = ring * 4 + side
        b = ring * 4 + (side + 1) % 4
        c = a + 4
        d = b + 4
        bar_indices += [a, b, d, a, d, c]
bar_joints = []
bar_weights = []
for (x, y, z) in bar_points:
    bar_joints += [0 if y < 1 else 1, 0, 0, 0]
    bar_weights += [1.0, 0.0, 0.0, 0.0]
bar_pos = add(floats([c for p in bar_points for c in p]), FLOAT, len(bar_points), "VEC3", ARRAY, bounds(bar_points))
bar_idx = add(struct.pack("<%dH" % len(bar_indices), *bar_indices), USHORT, len(bar_indices), "SCALAR", ELEMENTS)
bar_jnt = add(struct.pack("<%dB" % len(bar_joints), *bar_joints), UBYTE, len(bar_points), "VEC4", ARRAY)
bar_wgt = add(floats(bar_weights), FLOAT, len(bar_points), "VEC4", ARRAY)

# The joints' inverse bind matrices (column-major): Root at the origin, Tip one unit up.
identity = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]
tip_inverse = [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, -1, 0, 1]
ibm = add(floats(identity + tip_inverse), FLOAT, 2, "MAT4")

# ── the puff: a box half a unit across, and a target that doubles it ──
s = 0.25
box_points = [(x, y, z) for x in (-s, s) for y in (-s, s) for z in (-s, s)]
box_faces = [(0, 1, 3, 2), (4, 6, 7, 5), (0, 4, 5, 1), (2, 3, 7, 6), (0, 2, 6, 4), (1, 5, 7, 3)]
box_indices = []
for (a, b, c, d) in box_faces:
    box_indices += [a, b, c, a, c, d]
box_pos = add(floats([c for p in box_points for c in p]), FLOAT, 8, "VEC3", ARRAY, bounds(box_points))
box_idx = add(struct.pack("<%dH" % len(box_indices), *box_indices), USHORT, len(box_indices), "SCALAR", ELEMENTS)
deltas = [c for p in box_points for c in p]  # each corner moved as far again: twice the size at weight 1
box_delta = add(floats(deltas), FLOAT, 8, "VEC3", None, bounds(box_points))

# ── the animations ──
bend_times = add(floats([0.0, 1.0, 2.0]), FLOAT, 3, "SCALAR", None, ([0.0], [2.0]))
quarter = math.sin(math.pi / 4)
bend_turns = add(floats([0, 0, 0, 1, 0, 0, quarter, quarter, 0, 0, 0, 1]), FLOAT, 3, "VEC4")
puff_times = add(floats([0.0, 1.0]), FLOAT, 2, "SCALAR", None, ([0.0], [1.0]))
puff_weights = add(floats([0.0, 1.0]), FLOAT, 2, "SCALAR")

gltf = {
    "asset": {"version": "2.0", "generator": "QQ make_rig.py"},
    "scene": 0,
    "scenes": [{"nodes": [0]}],
    "nodes": [
        {"name": "Rig", "children": [1, 3, 4]},
        {"name": "Root", "children": [2]},
        {"name": "Tip", "translation": [0, 1, 0]},
        {"name": "Bar", "mesh": 0, "skin": 0},
        {"name": "Puff", "mesh": 1, "translation": [1.5, 1, 0]},
    ],
    "skins": [{"name": "Arm", "joints": [1, 2], "inverseBindMatrices": ibm, "skeleton": 1}],
    "meshes": [
        {"name": "Bar", "primitives": [{"attributes": {"POSITION": bar_pos, "JOINTS_0": bar_jnt, "WEIGHTS_0": bar_wgt},
                                        "indices": bar_idx, "material": 0}]},
        {"name": "Puff", "weights": [0.0], "extras": {"targetNames": ["grow"]},
         "primitives": [{"attributes": {"POSITION": box_pos}, "indices": box_idx, "material": 1,
                         "targets": [{"POSITION": box_delta}]}]},
    ],
    "materials": [
        {"name": "Ember", "pbrMetallicRoughness": {"baseColorFactor": [1.0, 0.42, 0.0, 1.0], "metallicFactor": 0.0,
                                                   "roughnessFactor": 0.5}},
        {"name": "Gilt", "pbrMetallicRoughness": {"baseColorFactor": [0.79, 0.64, 0.36, 1.0], "metallicFactor": 0.0,
                                                  "roughnessFactor": 0.5}},
    ],
    "animations": [
        {"name": "bend", "channels": [{"sampler": 0, "target": {"node": 2, "path": "rotation"}}],
         "samplers": [{"input": bend_times, "output": bend_turns, "interpolation": "LINEAR"}]},
        {"name": "puff", "channels": [{"sampler": 0, "target": {"node": 4, "path": "weights"}}],
         "samplers": [{"input": puff_times, "output": puff_weights, "interpolation": "LINEAR"}]},
    ],
    "accessors": accessors,
    "bufferViews": views,
    "buffers": [{"byteLength": len(blob),
                 "uri": "data:application/octet-stream;base64," + base64.b64encode(bytes(blob)).decode("ascii")}],
}
(HERE / "rig.gltf").write_text(json.dumps(gltf, indent=1) + "\n", encoding="utf-8", newline="\n")
