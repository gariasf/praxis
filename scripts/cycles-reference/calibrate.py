# SPDX-License-Identifier: MIT
"""Checks the light units render.py builds with against their closed forms:
the luminance of a white Lambertian plane facing each light, seen from
above, is E/π for the illuminance E the light gives it. Prints a line per
light and exits non-zero unless each lands within 1%.

    blender -b --factory-startup --python-exit-code 1 --python calibrate.py -- [--wrong-unit]

`--wrong-unit` builds point lights as if Cycles' power were their
intensity, which the check must fail: a test of the check itself.
"""
import math
import os
import sys
import tempfile

import bpy
import numpy as np
from mathutils import Vector

# render.py is imported from beside this script, writing no bytecode there.
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import render  # noqa: E402

TOLERANCE = 0.01


def lambertian_plane(scene):
    """A white Lambertian plane on z = 0, filling an orthographic camera's
    view from half a metre above it, and no other light."""
    render.world(scene, (0.0, 0.0, 0.0), (0.0, 0.0, 0.0))
    plane_data = bpy.data.meshes.new("plane")
    plane_data.from_pydata([(-50, -50, 0), (50, -50, 0), (50, 50, 0), (-50, 50, 0)], [], [(0, 1, 2, 3)])
    material = bpy.data.materials.new("lambertian")
    nodes = material.node_tree.nodes
    nodes.clear()
    diffuse = nodes.new("ShaderNodeBsdfDiffuse")
    diffuse.inputs["Color"].default_value = (1.0, 1.0, 1.0, 1.0)
    material.node_tree.links.new(diffuse.outputs[0], nodes.new("ShaderNodeOutputMaterial").inputs[0])
    plane_data.materials.append(material)
    render.link(scene, "plane", plane_data)
    camera_data = bpy.data.cameras.new("camera")
    camera_data.type = "ORTHO"
    camera_data.ortho_scale = 0.01
    scene.camera = render.link(scene, "camera", camera_data)
    scene.camera.location = (0.0, 0.0, 0.5)


def measured(scene, directory, name):
    scene.render.filepath = os.path.join(directory, f"{name}.exr")
    bpy.ops.render.render(write_still=True)
    image = bpy.data.images.load(scene.render.filepath)
    pixels = np.empty(len(image.pixels), np.float32)
    image.pixels.foreach_get(pixels)
    return float(pixels.reshape(-1, 4)[:, :3].mean())


def main():
    render.require_blender()
    if "--wrong-unit" in render.arguments():
        render.POINT_POWER_PER_INTENSITY = 1.0
    directory = tempfile.mkdtemp(prefix="praxis-calibration-")
    white = (1.0, 1.0, 1.0)
    # Each case: its name, what it adds to the plane's scene, and the
    # illuminance that gives the plane.
    cases = [
        ("point at 1 m", lambda scene: render.point_light(
            scene, Vector((0, 0, 1)), white, 1.0, render.UNBOUNDED_RANGE, True), 1.0),
        ("point at 10 m", lambda scene: render.point_light(
            scene, Vector((0, 0, 10)), white, 1.0, render.UNBOUNDED_RANGE, True), 1.0 / 100.0),
        # KHR_lights_punctual's window, 1 − (10 / 20)⁴ = 0.9375.
        ("point at 10 m of range 20 m", lambda scene: render.point_light(
            scene, Vector((0, 0, 10)), white, 1.0, 20.0, True), 0.9375 / 100.0),
        ("sun head-on", lambda scene: render.sun(scene, Vector((0, 0, -1)), white, 1.0), 1.0),
        ("ambient", lambda scene: render.world(scene, white, (0.0, 0.0, 0.0)), 1.0),
    ]
    failed = False
    for name, add_light, illuminance in cases:
        scene = render.new_scene(16, 16, 64)
        lambertian_plane(scene)
        add_light(scene)
        luminance = measured(scene, directory, name.replace(" ", "-"))
        expected = illuminance / math.pi
        error = abs(luminance / expected - 1.0)
        failed |= error > TOLERANCE
        verdict = "FAIL" if error > TOLERANCE else "ok"
        print(f"{verdict:<5} {name:<28} {luminance:.6f}, E/pi {expected:.6f} ({error:.2%} off)")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
