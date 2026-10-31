# SPDX-License-Identifier: MIT
"""A Reference scene that `praxis export-scene` wrote, rendered in Cycles as
the reference praxis's capture is judged against: direct light only, every
light at its exposed value, the ambient as a white world all around, and
glTF's metallic-roughness Materials as Principled BSDFs of single-scatter
GGX. Writes the exposed linear colour, Rec. 709, to an OpenEXR file.

    blender -b --factory-startup --python-exit-code 1 --python render.py --
        <export directory> <exr> <width>x<height> [<samples>]

calibrate.py checks the light units these builders assume.
"""
import json
import math
import os
import re
import struct
import sys

import bpy
import numpy as np
from mathutils import Matrix, Quaternion, Vector

# The Blender series these scripts are written against, so that a reference
# render is reproducible.
BLENDER_SERIES = (5, 2)
SCENE_FORMAT_VERSION = 1
# Cycles gives a point light of power P the radiant intensity P / 4π
# (calibrate.py measures it), so a light of intensity I takes 4π I.
POINT_POWER_PER_INTENSITY = 4.0 * math.pi
# A range past every scene, standing for a point light with none.
UNBOUNDED_RANGE = 1e9
# glTF's +Y up to Blender's +Z up, (x, y, z) to (x, −z, y): a quarter turn
# about +X.
GLTF_TO_BLENDER = Matrix.Rotation(math.pi / 2.0, 4, "X")
DEFAULT_SAMPLES = 512
# praxis's roughness floor, read from the shader that holds it.
SHADING_WGSL = os.path.join(os.path.dirname(os.path.abspath(__file__)), "../../src/render/shaders/shading.wgsl")
with open(SHADING_WGSL) as shading:
    MINIMUM_ROUGHNESS = float(re.search(r"const MINIMUM_ROUGHNESS: f32 = ([0-9.]+);", shading.read()).group(1))


def require_blender():
    if bpy.app.version[:2] != BLENDER_SERIES:
        sys.exit(f"Blender {bpy.app.version_string} found; these scripts need {BLENDER_SERIES[0]}.{BLENDER_SERIES[1]}.x")


def arguments():
    return sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []


def new_scene(width, height, samples):
    """An empty scene that Cycles renders on a GPU where it finds one,
    direct light only, undenoised, each pixel the mean of its own area."""
    bpy.ops.wm.read_factory_settings(use_empty=True)
    scene = bpy.context.scene
    scene.render.engine = "CYCLES"
    preferences = bpy.context.preferences.addons["cycles"].preferences
    for backend in ("OPTIX", "CUDA", "HIP", "METAL", "ONEAPI"):
        try:
            preferences.compute_device_type = backend
        except TypeError:
            continue
        preferences.get_devices()
        if any(device.type == backend for device in preferences.devices):
            for device in preferences.devices:
                device.use = device.type == backend
            scene.cycles.device = "GPU"
            break
    print(f"render device: {scene.cycles.device} {preferences.compute_device_type if scene.cycles.device == 'GPU' else ''}")
    cycles = scene.cycles
    cycles.samples = samples
    cycles.use_adaptive_sampling = False
    cycles.use_denoising = False
    cycles.max_bounces = 0
    cycles.blur_glossy = 0.0
    cycles.sample_clamp_direct = 0.0
    cycles.light_sampling_threshold = 0.0
    cycles.pixel_filter_type = "BOX"
    cycles.filter_width = 1.0
    scene.render.resolution_x, scene.render.resolution_y = width, height
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = "OPEN_EXR"
    scene.render.image_settings.color_depth = "32"
    scene.view_settings.view_transform = "Standard"
    return scene


def link(scene, name, data):
    instance = bpy.data.objects.new(name, data)
    scene.collection.objects.link(instance)
    return instance


def point_light(scene, location, color, intensity, light_range, casts_shadow):
    """A point light of radius 0 and luminous intensity `intensity` ×
    `color`, at `location` in Blender's axes, under KHR_lights_punctual's
    range window, clamp(1 − (distance / range)⁴, 0, 1)
    [ref:khr-lights-punctual-range], as a light node tree on the ray's
    length."""
    light = bpy.data.lights.new("point", "POINT")
    light.energy = intensity * POINT_POWER_PER_INTENSITY
    light.color = color
    light.shadow_soft_size = 0.0
    light.use_soft_falloff = False
    light.use_shadow = casts_shadow
    nodes, links = light.node_tree.nodes, light.node_tree.links
    path = nodes.new("ShaderNodeLightPath")
    ratio = nodes.new("ShaderNodeMath")
    ratio.operation = "DIVIDE"
    ratio.inputs[1].default_value = light_range
    fourth_power = nodes.new("ShaderNodeMath")
    fourth_power.operation = "POWER"
    fourth_power.inputs[1].default_value = 4.0
    window = nodes.new("ShaderNodeMath")
    window.operation = "SUBTRACT"
    window.use_clamp = True
    window.inputs[0].default_value = 1.0
    links.new(path.outputs["Ray Length"], ratio.inputs[0])
    links.new(ratio.outputs[0], fourth_power.inputs[0])
    links.new(fourth_power.outputs[0], window.inputs[1])
    links.new(window.outputs[0], nodes["Emission"].inputs["Strength"])
    link(scene, "point", light).location = location


def sun(scene, direction, color, illuminance):
    """A sun of angle 0 travelling along `direction`, in Blender's axes,
    giving `illuminance` × `color` on a surface facing it."""
    light = bpy.data.lights.new("sun", "SUN")
    light.energy = illuminance
    light.color = color
    light.angle = 0.0
    instance = link(scene, "sun", light)
    instance.rotation_mode = "QUATERNION"
    instance.rotation_quaternion = Vector((0.0, 0.0, -1.0)).rotation_difference(direction)


def world(scene, ambient_illuminance, background_luminance):
    """Camera rays see `background_luminance`; every other ray a white world
    of radiance E/π, which gives a surface facing it the illuminance E."""
    environment = bpy.data.worlds.new("world")
    scene.world = environment
    nodes, links = environment.node_tree.nodes, environment.node_tree.links
    nodes.clear()
    ambient = nodes.new("ShaderNodeBackground")
    ambient.inputs["Color"].default_value = (*(value / math.pi for value in ambient_illuminance), 1.0)
    background = nodes.new("ShaderNodeBackground")
    background.inputs["Color"].default_value = (*background_luminance, 1.0)
    path = nodes.new("ShaderNodeLightPath")
    mix = nodes.new("ShaderNodeMixShader")
    output = nodes.new("ShaderNodeOutputWorld")
    links.new(path.outputs["Is Camera Ray"], mix.inputs[0])
    links.new(ambient.outputs[0], mix.inputs[1])
    links.new(background.outputs[0], mix.inputs[2])
    links.new(mix.outputs[0], output.inputs[0])


COMPONENT_TYPES = {5120: np.int8, 5121: np.uint8, 5122: np.int16, 5123: np.uint16, 5125: np.uint32, 5126: np.float32}
WIDTHS = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4}


class Gltf:
    """A glTF file's document and buffers, .gltf or .glb."""

    def __init__(self, path):
        with open(path, "rb") as file:
            data = file.read()
        self.directory = os.path.dirname(path)
        binary = None
        if data[:4] == b"glTF":
            json_length = struct.unpack_from("<I", data, 12)[0]
            self.document = json.loads(data[20:20 + json_length])
            binary = data[20 + json_length + 8:]
        else:
            self.document = json.loads(data)
        self.buffers = [
            binary if "uri" not in buffer else self.read(buffer["uri"])
            for buffer in self.document.get("buffers", [])
        ]

    def read(self, uri):
        with open(os.path.join(self.directory, uri), "rb") as file:
            return file.read()

    def view(self, index):
        view = self.document["bufferViews"][index]
        start = view.get("byteOffset", 0)
        return self.buffers[view["buffer"]][start:start + view["byteLength"]], view.get("byteStride")

    def accessor(self, index):
        accessor = self.document["accessors"][index]
        component = np.dtype(COMPONENT_TYPES[accessor["componentType"]])
        width = WIDTHS[accessor["type"]]
        data, stride = self.view(accessor["bufferView"])
        values = np.ndarray(
            (accessor["count"], width), component, data, accessor.get("byteOffset", 0),
            (stride or component.itemsize * width, component.itemsize),
        ).astype(np.float64 if component.kind == "f" or accessor.get("normalized") else np.int64)
        if accessor.get("normalized"):
            # glTF's normalized integers: c / max, signed ones held to −1.
            values = np.maximum(values / np.iinfo(component).max, -1.0)
        return values

    def image_bytes(self, index):
        image = self.document["images"][index]
        return self.view(image["bufferView"])[0] if "bufferView" in image else self.read(image["uri"])


def blender_mesh(name, gltf, index, materials):
    """Mesh `index` of `gltf` in its own axes, a material slot per
    Primitive; Primitives that share attributes share vertices."""
    vertex_starts, positions, normals, uvs, corners, slots = {}, [], [], [], [], []
    vertex_count = 0
    for slot, primitive in enumerate(gltf.document["meshes"][index]["primitives"]):
        if primitive.get("mode", 4) != 4:
            sys.exit(f"{name}: primitive {slot} is not a triangle list")
        attributes = primitive["attributes"]
        key = (attributes["POSITION"], attributes.get("NORMAL"), attributes.get("TEXCOORD_0"))
        if key not in vertex_starts:
            vertex_starts[key] = vertex_count
            position = gltf.accessor(key[0])
            positions.append(position)
            normals.append(gltf.accessor(key[1]) if key[1] is not None else None)
            uvs.append(gltf.accessor(key[2]) if key[2] is not None else np.zeros((len(position), 2)))
            vertex_count += len(position)
        if "indices" in primitive:
            primitive_corners = gltf.accessor(primitive["indices"]).ravel()
        else:
            primitive_corners = np.arange(gltf.document["accessors"][key[0]]["count"])
        corners.append(primitive_corners + vertex_starts[key])
        slots.append(np.full(len(primitive_corners) // 3, slot))
    # Blender's own normals stand unless every Primitive gives its own.
    normals = None if any(normal is None for normal in normals) else np.concatenate(normals)
    corners, uvs = np.concatenate(corners), np.concatenate(uvs)
    mesh = bpy.data.meshes.new(name)
    mesh.vertices.add(vertex_count)
    mesh.vertices.foreach_set("co", np.concatenate(positions).ravel())
    mesh.loops.add(len(corners))
    mesh.loops.foreach_set("vertex_index", corners)
    mesh.polygons.add(len(corners) // 3)
    mesh.polygons.foreach_set("loop_start", np.arange(0, len(corners), 3))
    mesh.polygons.foreach_set("material_index", np.concatenate(slots))
    # glTF's texture coordinates start at the image's top, Blender's at its
    # bottom.
    corner_uvs = uvs[corners] * (1.0, -1.0) + (0.0, 1.0)
    mesh.uv_layers.new(name="UV").data.foreach_set("uv", corner_uvs.ravel())
    mesh.update(calc_edges=True)
    if normals is not None:
        mesh.normals_split_custom_set_from_vertices(normals)
    for material in materials:
        mesh.materials.append(material)
    return mesh


class MaterialBuilder:
    """Blender materials from `scene.json`'s, their images loaded once."""

    def __init__(self, directory, exposure):
        self.directory, self.exposure, self.images = directory, exposure, {}

    def image(self, texture, color_space):
        key = (texture["file"], texture.get("image"), color_space)
        if key not in self.images:
            path = os.path.join(self.directory, texture["file"])
            if "image" in texture:
                data = Gltf(path).image_bytes(texture["image"])
                image = bpy.data.images.new(os.path.basename(path), 8, 8)
                image.pack(data=data, data_len=len(data))
                image.source = "FILE"
            else:
                image = bpy.data.images.load(path)
            image.colorspace_settings.name = color_space
            self.images[key] = image
        return self.images[key]

    def build(self, name, entry):
        material = bpy.data.materials.new(name)
        nodes, links = material.node_tree.nodes, material.node_tree.links
        bsdf = nodes["Principled BSDF"]
        bsdf.distribution = "GGX"
        bsdf.inputs["IOR"].default_value = 1.5
        textures = {texture["channel"]: texture for texture in entry["textures"]}

        def sampled(channel, color_space):
            node = nodes.new("ShaderNodeTexImage")
            node.image = self.image(textures[channel], color_space)
            return node.outputs["Color"]

        def scaled(target, factor, value):
            """`target` takes `factor`, times `value` when a texture gives one."""
            if value is None:
                target.default_value = factor if np.isscalar(factor) else (*factor, 1.0)
                return
            product = nodes.new("ShaderNodeVectorMath")
            product.operation = "MULTIPLY"
            product.inputs[1].default_value = (factor,) * 3 if np.isscalar(factor) else factor
            links.new(value, product.inputs[0])
            links.new(product.outputs[0], target)

        albedo = sampled("albedo", "sRGB") if "albedo" in textures else None
        scaled(bsdf.inputs["Base Color"], entry["base_color"][:3], albedo)
        # glTF's metallic-roughness texture holds roughness in green and
        # metallic in blue, each a factor's multiplier
        # [ref:gltf-2.0-metallic-roughness].
        metallic = roughness = None
        if "metallic_roughness" in textures:
            separate = nodes.new("ShaderNodeSeparateColor")
            links.new(sampled("metallic_roughness", "Non-Color"), separate.inputs[0])
            roughness, metallic = separate.outputs["Green"], separate.outputs["Blue"]
        scaled(bsdf.inputs["Metallic"], entry["metallic"], metallic)
        # glTF's Appendix B never evaluates α = 0: it clamps α to a small
        # positive value or reflects as a mirror
        # [ref:gltf-2.0-zero-roughness]. praxis clamps; Cycles' mirror would
        # reflect no point light of radius 0, so the clamp is praxis's.
        floor = nodes.new("ShaderNodeMath")
        floor.operation = "MAXIMUM"
        floor.inputs[1].default_value = MINIMUM_ROUGHNESS
        scaled(floor.inputs[0], entry["roughness"], roughness)
        links.new(floor.outputs[0], bsdf.inputs["Roughness"])
        if "normal" in textures:
            # glTF scales the tangent-space normal's x and y,
            # (texel · 2 − 1) ⊙ (scale, scale, 1); Blender's Strength blends
            # towards the surface normal instead, so the scale is applied
            # here and the result encoded back for the Normal Map node.
            scale = entry["normal_scale"]
            scaled_normal = nodes.new("ShaderNodeVectorMath")
            scaled_normal.operation = "MULTIPLY_ADD"
            scaled_normal.inputs[1].default_value = (2.0 * scale, 2.0 * scale, 2.0)
            scaled_normal.inputs[2].default_value = (-scale, -scale, -1.0)
            encoded = nodes.new("ShaderNodeVectorMath")
            encoded.operation = "MULTIPLY_ADD"
            encoded.inputs[1].default_value = (0.5, 0.5, 0.5)
            encoded.inputs[2].default_value = (0.5, 0.5, 0.5)
            normal_map = nodes.new("ShaderNodeNormalMap")
            normal_map.uv_map = "UV"
            links.new(sampled("normal", "Non-Color"), scaled_normal.inputs[0])
            links.new(scaled_normal.outputs[0], encoded.inputs[0])
            links.new(encoded.outputs[0], normal_map.inputs["Color"])
            links.new(normal_map.outputs[0], bsdf.inputs["Normal"])
        # The occlusion texture goes unused: Cycles' rays find what hides the
        # ambient, which the texture approximates.
        emissive = sampled("emissive", "sRGB") if "emissive" in textures else None
        scaled(bsdf.inputs["Emission Color"], entry["emissive"], emissive)
        bsdf.inputs["Emission Strength"].default_value = self.exposure
        return material


def instancer(name, source):
    """Geometry nodes that place `source` at each point of a point cloud,
    turned by its `rotation` and scaled by its `scale`."""
    tree = bpy.data.node_groups.new(name, "GeometryNodeTree")
    tree.interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    tree.interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")
    nodes, links = tree.nodes, tree.links
    source_info = nodes.new("GeometryNodeObjectInfo")
    source_info.inputs["Object"].default_value = source
    place = nodes.new("GeometryNodeInstanceOnPoints")
    for attribute, data_type in (("rotation", "QUATERNION"), ("scale", "FLOAT_VECTOR")):
        named = nodes.new("GeometryNodeInputNamedAttribute")
        named.data_type = data_type
        named.inputs["Name"].default_value = attribute
        links.new(named.outputs["Attribute"], place.inputs[attribute.capitalize()])
    links.new(nodes.new("NodeGroupInput").outputs[0], place.inputs["Points"])
    links.new(source_info.outputs["Geometry"], place.inputs["Instance"])
    links.new(place.outputs["Instances"], nodes.new("NodeGroupOutput").inputs[0])
    return tree


def to_blender(points):
    """glTF positions or directions as Blender's."""
    return np.asarray(points, np.float64) @ np.array(GLTF_TO_BLENDER.to_3x3()).T


def build(directory, width, height, samples):
    with open(os.path.join(directory, "scene.json")) as file:
        exported = json.load(file)
    if exported["version"] != SCENE_FORMAT_VERSION:
        sys.exit(f"scene.json version {exported['version']}; this script reads {SCENE_FORMAT_VERSION}")
    scene = new_scene(width, height, samples)
    camera = exported["camera"]
    exposure = 1.0 / (1.2 * 2.0 ** camera["ev100"])
    # Everything is placed relative to the camera, in f64 until then, so
    # far scenes keep their precision in Blender's f32.
    origin = np.array(camera["position"])
    camera_data = bpy.data.cameras.new("camera")
    camera_data.sensor_fit = "VERTICAL"
    camera_data.angle_y = camera["vertical_field_of_view"]
    camera_data.clip_start = 0.1
    camera_data.clip_end = 1e6
    scene.camera = link(scene, "camera", camera_data)
    x, y, z, w = camera["rotation"]
    scene.camera.matrix_world = GLTF_TO_BLENDER @ Quaternion((w, x, y, z)).to_matrix().to_4x4()

    environment = exported["environment"]
    world(
        scene,
        [value * exposure for value in environment["ambient_illuminance"]],
        [value * exposure for value in environment["background_luminance"]],
    )
    for light in exported["directional_lights"]:
        sun(scene, Vector(to_blender(light["direction"])), light["color"], light["illuminance"] * exposure)
    for light in exported["point_lights"]:
        location = Vector(to_blender(np.array(light["position"]) - origin))
        point_light(scene, location, light["color"], light["intensity"] * exposure, light["range"], light["casts_shadow"])

    builder = MaterialBuilder(directory, exposure)
    materials = {name: builder.build(name, entry) for name, entry in exported["materials"].items()}
    meshes = {}
    for name, entry in exported["meshes"].items():
        slots = [materials[material] if material else None for material in entry["materials"]]
        meshes[name] = blender_mesh(name, Gltf(os.path.join(directory, entry["file"])), entry["mesh"], slots)

    # Renderables that draw one Mesh with one Material become one point
    # cloud whose points instance it.
    groups = {}
    for renderable in exported["renderables"]:
        groups.setdefault((renderable["mesh"], renderable["material"]), []).append(renderable)
    for (mesh_name, material_name), renderables in groups.items():
        mesh = meshes[mesh_name]
        if material_name is not None:
            mesh = mesh.copy()
            for slot in range(len(mesh.materials)):
                mesh.materials[slot] = materials[material_name]
        source = link(scene, f"{mesh_name} {material_name}", mesh)
        source.hide_render = True
        translations = to_blender(np.array([entry["translation"] for entry in renderables]) - origin)
        # [x, y, z, w] to Blender's (w, x, y, z), then turned into Blender's
        # axes: the axis change's quaternion times each.
        rotations = np.array([entry["rotation"] for entry in renderables])[:, [3, 0, 1, 2]]
        cosine, sine = GLTF_TO_BLENDER.to_quaternion()[:2]
        w, x, y, z = rotations.T
        rotations = np.stack([cosine * w - sine * x, cosine * x + sine * w, cosine * y - sine * z, cosine * z + sine * y], 1)
        points = bpy.data.meshes.new(f"{mesh_name} {material_name} points")
        points.vertices.add(len(renderables))
        points.vertices.foreach_set("co", translations.ravel())
        points.attributes.new("rotation", "QUATERNION", "POINT").data.foreach_set("value", rotations.ravel())
        scales = np.array([entry["scale"] for entry in renderables])
        points.attributes.new("scale", "FLOAT_VECTOR", "POINT").data.foreach_set("vector", scales.ravel())
        placed = link(scene, f"{mesh_name} {material_name} instances", points)
        placed.modifiers.new("instances", "NODES").node_group = instancer(placed.name, source)
    return scene


if __name__ == "__main__":
    require_blender()
    directory, output, size, *rest = arguments()
    width, height = (int(value) for value in size.split("x"))
    scene = build(os.path.abspath(directory), width, height, int(rest[0]) if rest else DEFAULT_SAMPLES)
    scene.render.filepath = os.path.abspath(output)
    bpy.ops.render.render(write_still=True)
