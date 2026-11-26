"""
Shared glTF 2.0 (.glb) binary builder and procedural mesh primitives.
"""

import json
import math
import os
import struct
from typing import List, Optional, Tuple


# ---------------------------------------------------------------------------
# Math helpers
# ---------------------------------------------------------------------------

def norm3(v: Tuple[float, float, float]) -> Tuple[float, float, float]:
    l = math.sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
    if l < 1e-9:
        return (0.0, 1.0, 0.0)
    return (v[0] / l, v[1] / l, v[2] / l)


def sub3(a: Tuple[float, float, float], b: Tuple[float, float, float]) -> Tuple[float, float, float]:
    return (a[0] - b[0], a[1] - b[1], a[2] - b[2])


def cross3(a: Tuple[float, float, float], b: Tuple[float, float, float]) -> Tuple[float, float, float]:
    return (
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    )


def euler_to_quat(rx: float, ry: float, rz: float) -> Tuple[float, float, float, float]:
    """Convert Euler XYZ (radians) to glTF quaternion (x, y, z, w)."""
    cx, sx = math.cos(rx * 0.5), math.sin(rx * 0.5)
    cy, sy = math.cos(ry * 0.5), math.sin(ry * 0.5)
    cz, sz = math.cos(rz * 0.5), math.sin(rz * 0.5)

    w = cz * cy * cx + sz * sy * sx
    x = cz * cy * sx - sz * sy * cx
    y = cz * sy * cx + sz * cy * sx
    z = sz * cy * cx - cz * sy * sx
    l = math.sqrt(x * x + y * y + z * z + w * w)
    return (x / l, y / l, z / l, w / l)


# ---------------------------------------------------------------------------
# Geometry builders (return positions, normals, indices)
# ---------------------------------------------------------------------------

MeshData = Tuple[List[Tuple[float, float, float]], List[Tuple[float, float, float]], List[int]]


def merge_meshes(parts: List[MeshData]) -> MeshData:
    positions: List[Tuple[float, float, float]] = []
    normals: List[Tuple[float, float, float]] = []
    indices: List[int] = []
    for p_pos, p_norm, p_idx in parts:
        offset = len(positions)
        positions.extend(p_pos)
        normals.extend(p_norm)
        indices.extend(i + offset for i in p_idx)
    return positions, normals, indices


def make_tapered_box(
    w_bot: float,
    d_bot: float,
    w_top: float,
    d_top: float,
    h: float,
    center: Tuple[float, float, float] = (0.0, 0.0, 0.0),
) -> MeshData:
    """Create a flat-shaded tapered box centered at `center`."""
    cx, cy, cz = center
    hy = h * 0.5
    xb, zb = w_bot * 0.5, d_bot * 0.5
    xt, zt = w_top * 0.5, d_top * 0.5

    v = [
        (cx - xb, cy - hy, cz + zb),
        (cx + xb, cy - hy, cz + zb),
        (cx + xb, cy - hy, cz - zb),
        (cx - xb, cy - hy, cz - zb),
        (cx - xt, cy + hy, cz + zt),
        (cx + xt, cy + hy, cz + zt),
        (cx + xt, cy + hy, cz - zt),
        (cx - xt, cy + hy, cz - zt),
    ]

    faces = [
        (0, 1, 5, 4),  # front (+Z)
        (1, 2, 6, 5),  # right (+X)
        (2, 3, 7, 6),  # back (-Z)
        (3, 0, 4, 7),  # left (-X)
        (4, 5, 6, 7),  # top (+Y)
        (3, 2, 1, 0),  # bottom (-Y)
    ]

    positions: List[Tuple[float, float, float]] = []
    normals: List[Tuple[float, float, float]] = []
    indices: List[int] = []

    for a, b, c, d in faces:
        n = norm3(cross3(sub3(v[b], v[a]), sub3(v[c], v[a])))
        base = len(positions)
        positions.extend([v[a], v[b], v[c], v[d]])
        normals.extend([n, n, n, n])
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3])

    return positions, normals, indices


def make_box(
    w: float, h: float, d: float, center: Tuple[float, float, float] = (0.0, 0.0, 0.0)
) -> MeshData:
    return make_tapered_box(w, d, w, d, h, center)


def make_uv_sphere(
    rx: float,
    ry: float,
    rz: float,
    rings: int = 10,
    sectors: int = 14,
    center: Tuple[float, float, float] = (0.0, 0.0, 0.0),
) -> MeshData:
    cx, cy, cz = center
    positions: List[Tuple[float, float, float]] = []
    normals: List[Tuple[float, float, float]] = []
    indices: List[int] = []

    for r in range(rings + 1):
        phi = math.pi * r / rings
        y_unit = math.cos(phi)
        sin_phi = math.sin(phi)
        for s in range(sectors + 1):
            theta = 2.0 * math.pi * s / sectors
            x_unit = sin_phi * math.cos(theta)
            z_unit = sin_phi * math.sin(theta)
            positions.append((cx + rx * x_unit, cy + ry * y_unit, cz + rz * z_unit))
            nx = x_unit / max(rx, 1e-6)
            ny = y_unit / max(ry, 1e-6)
            nz = z_unit / max(rz, 1e-6)
            normals.append(norm3((nx, ny, nz)))

    stride = sectors + 1
    for r in range(rings):
        for s in range(sectors):
            i0 = r * stride + s
            i1 = i0 + 1
            i2 = (r + 1) * stride + s
            i3 = i2 + 1
            if r != 0:
                indices.extend([i0, i1, i2])
            if r != rings - 1:
                indices.extend([i1, i3, i2])

    return positions, normals, indices


def make_cylinder(
    r_bot: float,
    r_top: float,
    h: float,
    sectors: int = 12,
    center: Tuple[float, float, float] = (0.0, 0.0, 0.0),
) -> MeshData:
    cx, cy, cz = center
    hy = h * 0.5
    positions: List[Tuple[float, float, float]] = []
    normals: List[Tuple[float, float, float]] = []
    indices: List[int] = []

    side_start = 0
    slope = (r_bot - r_top) / max(h, 1e-6)
    for ring_y, r in [(cy - hy, r_bot), (cy + hy, r_top)]:
        for s in range(sectors + 1):
            ang = 2.0 * math.pi * s / sectors
            ca, sa = math.cos(ang), math.sin(ang)
            positions.append((cx + r * ca, ring_y, cz + r * sa))
            normals.append(norm3((ca, slope, sa)))

    stride = sectors + 1
    for s in range(sectors):
        b0 = side_start + s
        b1 = b0 + 1
        t0 = side_start + stride + s
        t1 = t0 + 1
        indices.extend([b0, t0, t1, b0, t1, b1])

    # Top cap
    top_center_idx = len(positions)
    positions.append((cx, cy + hy, cz))
    normals.append((0.0, 1.0, 0.0))
    top_ring_start = len(positions)
    for s in range(sectors + 1):
        ang = 2.0 * math.pi * s / sectors
        positions.append((cx + r_top * math.cos(ang), cy + hy, cz + r_top * math.sin(ang)))
        normals.append((0.0, 1.0, 0.0))
    for s in range(sectors):
        indices.extend([top_center_idx, top_ring_start + s + 1, top_ring_start + s])

    # Bottom cap
    bot_center_idx = len(positions)
    positions.append((cx, cy - hy, cz))
    normals.append((0.0, -1.0, 0.0))
    bot_ring_start = len(positions)
    for s in range(sectors + 1):
        ang = 2.0 * math.pi * s / sectors
        positions.append((cx + r_bot * math.cos(ang), cy - hy, cz + r_bot * math.sin(ang)))
        normals.append((0.0, -1.0, 0.0))
    for s in range(sectors):
        indices.extend([bot_center_idx, bot_ring_start + s, bot_ring_start + s + 1])

    return positions, normals, indices


def rotate_y_mesh(mesh: MeshData, angle_rad: float) -> MeshData:
    """Rotate mesh vertices and normals around the Y axis."""
    c, s = math.cos(angle_rad), math.sin(angle_rad)
    pos, norm, idx = mesh
    r_pos = [(x * c + z * s, y, -x * s + z * c) for (x, y, z) in pos]
    r_norm = [(x * c + z * s, y, -x * s + z * c) for (x, y, z) in norm]
    return r_pos, r_norm, list(idx)


# ---------------------------------------------------------------------------
# GLB Builder
# ---------------------------------------------------------------------------

class GLBBuilder:
    def __init__(self, scene_name: str = "Scene") -> None:
        self.scene_name = scene_name
        self.bin_data = bytearray()
        self.buffer_views: List[dict] = []
        self.accessors: List[dict] = []
        self.materials: List[dict] = []
        self.meshes: List[dict] = []
        self.nodes: List[dict] = []
        self.animations: List[dict] = []

    def _pad4(self) -> None:
        rem = len(self.bin_data) % 4
        if rem:
            self.bin_data.extend(b"\x00" * (4 - rem))

    def add_buffer_view(self, raw: bytes, target: Optional[int] = None) -> int:
        self._pad4()
        offset = len(self.bin_data)
        self.bin_data.extend(raw)
        bv: dict = {
            "buffer": 0,
            "byteOffset": offset,
            "byteLength": len(raw),
        }
        if target is not None:
            bv["target"] = target
        idx = len(self.buffer_views)
        self.buffer_views.append(bv)
        return idx

    def add_vec3_accessor(
        self,
        vecs: List[Tuple[float, float, float]],
        target: Optional[int] = None,
        with_minmax: bool = False,
    ) -> int:
        raw = b"".join(struct.pack("<3f", *v) for v in vecs)
        bv_idx = self.add_buffer_view(raw, target=target)
        acc: dict = {
            "bufferView": bv_idx,
            "componentType": 5126,
            "count": len(vecs),
            "type": "VEC3",
        }
        if with_minmax and vecs:
            xs = [v[0] for v in vecs]
            ys = [v[1] for v in vecs]
            zs = [v[2] for v in vecs]
            acc["min"] = [min(xs), min(ys), min(zs)]
            acc["max"] = [max(xs), max(ys), max(zs)]
        idx = len(self.accessors)
        self.accessors.append(acc)
        return idx

    def add_vec4_accessor(self, vecs: List[Tuple[float, float, float, float]]) -> int:
        raw = b"".join(struct.pack("<4f", *v) for v in vecs)
        bv_idx = self.add_buffer_view(raw)
        acc = {
            "bufferView": bv_idx,
            "componentType": 5126,
            "count": len(vecs),
            "type": "VEC4",
        }
        idx = len(self.accessors)
        self.accessors.append(acc)
        return idx

    def add_scalar_float_accessor(self, vals: List[float], with_minmax: bool = False) -> int:
        raw = b"".join(struct.pack("<f", v) for v in vals)
        bv_idx = self.add_buffer_view(raw)
        acc: dict = {
            "bufferView": bv_idx,
            "componentType": 5126,
            "count": len(vals),
            "type": "SCALAR",
        }
        if with_minmax and vals:
            acc["min"] = [min(vals)]
            acc["max"] = [max(vals)]
        idx = len(self.accessors)
        self.accessors.append(acc)
        return idx

    def add_indices_accessor(self, indices: List[int]) -> int:
        raw = b"".join(struct.pack("<H", i) for i in indices)
        bv_idx = self.add_buffer_view(raw, target=34963)
        acc = {
            "bufferView": bv_idx,
            "componentType": 5123,
            "count": len(indices),
            "type": "SCALAR",
        }
        idx = len(self.accessors)
        self.accessors.append(acc)
        return idx

    def add_material(
        self,
        name: str,
        rgba: Tuple[float, float, float, float],
        roughness: float = 0.7,
        metallic: float = 0.0,
        alpha_mode: Optional[str] = None,
        emissive: Optional[Tuple[float, float, float]] = None,
    ) -> int:
        mat: dict = {
            "name": name,
            "pbrMetallicRoughness": {
                "baseColorFactor": list(rgba),
                "roughnessFactor": roughness,
                "metallicFactor": metallic,
            },
        }
        if alpha_mode:
            mat["alphaMode"] = alpha_mode
        if emissive:
            mat["emissiveFactor"] = list(emissive)
        idx = len(self.materials)
        self.materials.append(mat)
        return idx

    def add_mesh(self, name: str, primitives_spec: List[Tuple[MeshData, int]]) -> int:
        primitives = []
        for (positions, normals, indices), mat_idx in primitives_spec:
            pos_acc = self.add_vec3_accessor(positions, target=34962, with_minmax=True)
            norm_acc = self.add_vec3_accessor(normals, target=34962, with_minmax=False)
            idx_acc = self.add_indices_accessor(indices)
            primitives.append({
                "attributes": {
                    "POSITION": pos_acc,
                    "NORMAL": norm_acc,
                },
                "indices": idx_acc,
                "material": mat_idx,
            })
        mesh_idx = len(self.meshes)
        self.meshes.append({"name": name, "primitives": primitives})
        return mesh_idx

    def add_node(
        self,
        name: str,
        translation: Tuple[float, float, float] = (0.0, 0.0, 0.0),
        rotation: Tuple[float, float, float, float] = (0.0, 0.0, 0.0, 1.0),
        mesh: Optional[int] = None,
        children: Optional[List[int]] = None,
    ) -> int:
        node: dict = {
            "name": name,
            "translation": list(translation),
            "rotation": list(rotation),
        }
        if mesh is not None:
            node["mesh"] = mesh
        if children:
            node["children"] = children
        idx = len(self.nodes)
        self.nodes.append(node)
        return idx

    def add_animation(
        self,
        name: str,
        tracks: List[Tuple[int, str, List[float], List[tuple]]],
    ) -> None:
        samplers = []
        channels = []
        for node_idx, path, times, values in tracks:
            t_acc = self.add_scalar_float_accessor(times, with_minmax=True)
            if path == "translation":
                v_acc = self.add_vec3_accessor(values)
            elif path == "rotation":
                v_acc = self.add_vec4_accessor(values)
            else:
                raise ValueError(f"Unsupported path: {path}")
            s_idx = len(samplers)
            samplers.append({
                "input": t_acc,
                "interpolation": "LINEAR",
                "output": v_acc,
            })
            channels.append({
                "sampler": s_idx,
                "target": {"node": node_idx, "path": path},
            })
        self.animations.append({
            "name": name,
            "samplers": samplers,
            "channels": channels,
        })

    def build_glb(self, out_path: str, root_nodes: Optional[List[int]] = None) -> None:
        self._pad4()
        if root_nodes is None:
            root_nodes = [0]
        gltf: dict = {
            "asset": {
                "version": "2.0",
                "generator": "StartupTycoon Procedural Asset Generator",
            },
            "scene": 0,
            "scenes": [{"name": self.scene_name, "nodes": root_nodes}],
            "nodes": self.nodes,
            "meshes": self.meshes,
            "materials": self.materials,
            "accessors": self.accessors,
            "bufferViews": self.buffer_views,
            "buffers": [{"byteLength": len(self.bin_data)}],
        }
        if self.animations:
            gltf["animations"] = self.animations

        json_bytes = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
        json_pad = (4 - (len(json_bytes) % 4)) % 4
        json_bytes += b" " * json_pad

        bin_bytes = bytes(self.bin_data)
        total_length = 12 + 8 + len(json_bytes) + 8 + len(bin_bytes)

        os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
        with open(out_path, "wb") as f:
            f.write(struct.pack("<III", 0x46546C67, 2, total_length))
            f.write(struct.pack("<II", len(json_bytes), 0x4E4F534A))
            f.write(json_bytes)
            f.write(struct.pack("<II", len(bin_bytes), 0x004E4942))
            f.write(bin_bytes)
