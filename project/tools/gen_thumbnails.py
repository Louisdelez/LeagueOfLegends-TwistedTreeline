#!/usr/bin/env python3
"""Generate thumbnail PNGs for all GLB models — bright, visible renders."""
import os
import trimesh
from PIL import Image, ImageDraw
import numpy as np

PROJECT = "/media/louisdelez/SSD500/Workflows/LeagueOfLegends/project"
ASSETS = f"{PROJECT}/assets"
CLIENT_ASSETS = f"{PROJECT}/crates/sg-client/assets"
THUMB_DIR = f"{ASSETS}/models/thumbnails"
THUMB_SIZE = 128

os.makedirs(THUMB_DIR, exist_ok=True)

def find_glbs():
    glbs = []
    seen = set()
    for root_dir in [ASSETS, CLIENT_ASSETS]:
        for dirpath, _, filenames in os.walk(root_dir):
            for f in filenames:
                if f.endswith('.glb') and f not in seen:
                    full = os.path.join(dirpath, f)
                    if os.path.getsize(full) > 10_000_000:
                        continue
                    seen.add(f)
                    glbs.append((f, full))
    return glbs

def render_thumbnail(glb_path, out_path):
    try:
        scene = trimesh.load(glb_path)
        if isinstance(scene, trimesh.Scene):
            if len(scene.geometry) == 0:
                return False
            mesh = scene.to_geometry()
        else:
            mesh = scene

        if not hasattr(mesh, 'vertices') or len(mesh.vertices) == 0:
            return False

        verts = np.array(mesh.vertices)
        if len(verts) < 3:
            return False

        # Center and normalize
        center = (verts.max(axis=0) + verts.min(axis=0)) / 2
        verts = verts - center
        scale = np.abs(verts).max()
        if scale < 1e-6:
            return False
        verts = verts / scale * 0.42

        # Create image with gradient background
        img = Image.new('RGBA', (THUMB_SIZE, THUMB_SIZE), (35, 35, 45, 255))
        draw = ImageDraw.Draw(img)

        # Draw a subtle gradient background
        for y in range(THUMB_SIZE):
            t = y / THUMB_SIZE
            r = int(30 + t * 15)
            g = int(30 + t * 15)
            b = int(40 + t * 10)
            draw.line([(0, y), (THUMB_SIZE, y)], fill=(r, g, b, 255))

        # Project faces if available, otherwise just vertices
        if hasattr(mesh, 'faces') and len(mesh.faces) > 0:
            faces = mesh.faces
            # Sort faces by average Z (depth sorting)
            face_z = np.mean(verts[faces][:, :, 2], axis=1)
            sorted_idx = np.argsort(face_z)

            for fi in sorted_idx:
                face = faces[fi]
                pts = []
                for vi in face:
                    v = verts[vi]
                    px = int((v[0] + 0.5) * THUMB_SIZE)
                    py = int((-v[1] + 0.5) * THUMB_SIZE)
                    pts.append((px, py))

                if len(pts) >= 3:
                    # Color based on normal/depth
                    avg_z = np.mean(verts[face][:, 2])
                    avg_y = np.mean(verts[face][:, 1])
                    brightness = 0.5 + avg_z * 0.4 + avg_y * 0.3
                    brightness = max(0.2, min(1.0, brightness))

                    r = int(100 * brightness + 60)
                    g = int(120 * brightness + 50)
                    b = int(160 * brightness + 40)
                    r = min(255, r)
                    g = min(255, g)
                    b = min(255, b)

                    # Draw filled polygon
                    draw.polygon(pts, fill=(r, g, b, 255), outline=(r+20, g+20, b+20, 100))
        else:
            # Fallback: draw bright points
            for v in verts:
                px = int((v[0] + 0.5) * THUMB_SIZE)
                py = int((-v[1] + 0.5) * THUMB_SIZE)
                if 0 <= px < THUMB_SIZE and 0 <= py < THUMB_SIZE:
                    h = (v[2] + 0.5)
                    r = int(120 + h * 100)
                    g = int(140 + h * 80)
                    b = int(180 + h * 60)
                    draw.ellipse([px-2, py-2, px+2, py+2], fill=(min(r,255), min(g,255), min(b,255), 255))

        img.save(out_path)
        return True

    except Exception as e:
        print(f"  ERROR: {e}")
        return False

def main():
    # Delete old thumbnails
    for f in os.listdir(THUMB_DIR):
        os.remove(os.path.join(THUMB_DIR, f))

    glbs = find_glbs()
    print(f"Found {len(glbs)} GLB files")

    ok = 0
    for name, path in sorted(glbs):
        stem = os.path.splitext(name)[0]
        out = os.path.join(THUMB_DIR, f"{stem}.png")
        print(f"  {stem}...", end=" ", flush=True)
        if render_thumbnail(path, out):
            print("OK")
            ok += 1
        else:
            print("FAIL")

    print(f"\nDone: {ok}/{len(glbs)}")

if __name__ == "__main__":
    main()
