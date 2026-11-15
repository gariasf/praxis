"""Writes the hand-lowered forms of build_shadow_mask's MSL.

Usage: lower.py <MSL wgpu-hal compiles for build_shadow_mask, unchecked> <output directory>
Each form lands in <output directory>/<form>/build_shadow_mask.metal.
"""
import pathlib
import re
import sys

source = pathlib.Path(sys.argv[1]).read_text()
output = pathlib.Path(sys.argv[2])

QUERY_WITH_TRIANGLES = "metal::raytracing::intersection_query<metal::raytracing::instancing, metal::raytracing::triangle_data>"
QUERY_WITHOUT_TRIANGLES = "metal::raytracing::intersection_query<metal::raytracing::instancing>"

OPACITY_SELECTS = re.compile(
    r"        metal::raytracing::opacity_cull_mode cull_mode = \n.*?\n            \);\n"
    r"        params\.set_opacity_cull_mode\(cull_mode\);\n"
    r"        params\.force_opacity\(\n.*?\n        \);\n",
    re.DOTALL,
)
OPACITY_CONSTANTS = (
    "        params.set_opacity_cull_mode(metal::raytracing::opacity_cull_mode::none);\n"
    "        params.force_opacity(metal::raytracing::forced_opacity::opaque);\n"
)
READ_EVERY_FIELD = re.compile(
    r"    if \(ty != metal::raytracing::intersection_type::none\) \{\n.*?\n    \}\n    return intersection;",
    re.DOTALL,
)
READ_DISTANCE = (
    "    if (ty != metal::raytracing::intersection_type::none) {\n"
    "        intersection.t = intersector.get_committed_distance();\n"
    "    }\n"
    "    return intersection;"
)


def constant_opacity(text):
    text, selects = OPACITY_SELECTS.subn(OPACITY_CONSTANTS, text)
    assert selects == 2, selects
    text, reads = READ_EVERY_FIELD.subn(READ_DISTANCE, text)
    assert reads == 1, reads
    text, barycentrics = re.subn(r"        intersection\.barycentrics = .*\n", "", text)
    assert barycentrics == 1, barycentrics
    return text


def split(any_hit_query):
    helpers = f"""struct NearestHit {{
    bool found;
    float distance;
    bool front_face;
}};

bool dynamic_tlas_in_reach(
    metal::float3 origin,
    metal::float3 direction,
    float start_distance,
    float end_distance,
    constant CameraUniform& camera
) {{
    if (camera.dynamic_tlas_entries == 0u) {{
        return false;
    }}
    metal::float3 bounds_min = camera.dynamic_tlas_bounds_min;
    metal::float3 bounds_max = camera.dynamic_tlas_bounds_max;
    metal::float3 extent = metal::max(metal::abs(origin), metal::max(metal::abs(bounds_min), metal::abs(bounds_max)));
    float margin = DYNAMIC_TLAS_BOUNDS_MARGIN_SCALE * metal::max(metal::max(extent.x, extent.y), metal::max(extent.z, 1.0));
    return segment_meets_box(origin, direction, start_distance, end_distance, bounds_min - metal::float3(margin), bounds_max + metal::float3(margin));
}}

bool any_intersection(
    metal::float3 origin,
    metal::float3 direction,
    float start_distance,
    float end_distance,
    metal::raytracing::instance_acceleration_structure static_tlas,
    metal::raytracing::instance_acceleration_structure dynamic_tlas,
    constant CameraUniform& camera
) {{
    metal::raytracing::intersection_params params;
    params.set_opacity_cull_mode(metal::raytracing::opacity_cull_mode::none);
    params.force_opacity(metal::raytracing::forced_opacity::opaque);
    params.accept_any_intersection(true);
    metal::raytracing::ray ray = metal::raytracing::ray(origin, direction, start_distance, end_distance);
    {any_hit_query} query = {{}};
    query.reset(ray, static_tlas, 255u, params);
    query.next();
    if (query.get_committed_intersection_type() != metal::raytracing::intersection_type::none) {{
        return true;
    }}
    if (!dynamic_tlas_in_reach(origin, direction, start_distance, end_distance, camera)) {{
        return false;
    }}
    query.reset(ray, dynamic_tlas, 255u, params);
    query.next();
    return query.get_committed_intersection_type() != metal::raytracing::intersection_type::none;
}}

NearestHit committed_hit(
    {QUERY_WITH_TRIANGLES} query
) {{
    metal::raytracing::intersection_type type = query.get_committed_intersection_type();
    NearestHit hit = NearestHit {{false, 0.0, false}};
    if (type != metal::raytracing::intersection_type::none) {{
        hit.found = true;
        hit.distance = query.get_committed_distance();
    }}
    if (type == metal::raytracing::intersection_type::triangle) {{
        hit.front_face = query.is_committed_triangle_front_facing();
    }}
    return hit;
}}

NearestHit nearest_intersection(
    metal::float3 origin,
    metal::float3 direction,
    float start_distance,
    float end_distance,
    metal::raytracing::instance_acceleration_structure static_tlas,
    metal::raytracing::instance_acceleration_structure dynamic_tlas,
    constant CameraUniform& camera
) {{
    metal::raytracing::intersection_params params;
    params.set_opacity_cull_mode(metal::raytracing::opacity_cull_mode::none);
    params.force_opacity(metal::raytracing::forced_opacity::opaque);
    params.accept_any_intersection(false);
    {QUERY_WITH_TRIANGLES} query = {{}};
    query.reset(metal::raytracing::ray(origin, direction, start_distance, end_distance), static_tlas, 255u, params);
    query.next();
    NearestHit static_hit = committed_hit(query);
    float dynamic_end_distance = static_hit.found ? static_hit.distance : end_distance;
    if (!dynamic_tlas_in_reach(origin, direction, start_distance, dynamic_end_distance, camera)) {{
        return static_hit;
    }}
    query.reset(metal::raytracing::ray(origin, direction, start_distance, dynamic_end_distance), dynamic_tlas, 255u, params);
    query.next();
    NearestHit dynamic_hit = committed_hit(query);
    if (!dynamic_hit.found) {{
        return static_hit;
    }}
    return dynamic_hit;
}}

"""
    traces = """    bool hidden = false;
    bool trace_any = leaves_surface;
    float start_distance = 0.0;
    if (!leaves_surface) {
        NearestHit nearest = nearest_intersection(origin_2, direction_2, 0.0, distance, static_tlas, dynamic_tlas, camera);
        if (nearest.found) {
            if (nearest.front_face) {
                hidden = true;
            } else if (!((nearest.distance + offset_1) >= distance)) {
                trace_any = true;
                start_distance = nearest.distance + offset_1;
            }
        }
    }
    if (trace_any) {
        hidden = any_intersection(origin_2, direction_2, start_distance, distance, static_tlas, dynamic_tlas, camera);
    }
    return hidden;
}
"""
    text, lowered = re.subn(
        r"RayIntersection ray_query_get_intersection_true\(.*?(?=bool shadowed\()",
        lambda _: helpers,
        source,
        flags=re.DOTALL,
    )
    assert lowered == 1, lowered
    text, loops = re.subn(
        r"    flags_1 = leaves_surface \? 4u : 0u;\n    while\(true\) \{.*?    bool _e106 = hidden;\n    return _e106;\n\}\n",
        lambda _: traces,
        text,
        flags=re.DOTALL,
    )
    assert loops == 1, loops
    text, declarations = re.subn(r"    bool hidden = false;\n(?=    bool local_4)", "", text)
    assert declarations == 1, declarations
    return text


forms = {
    "control": source,
    "constant-opacity": constant_opacity(source),
    "split": split(QUERY_WITH_TRIANGLES),
    "split-no-triangle-data": split(QUERY_WITHOUT_TRIANGLES),
}
for form, text in forms.items():
    folder = output / form
    folder.mkdir(parents=True, exist_ok=True)
    (folder / "build_shadow_mask.metal").write_text(text)
    print(form, len(text.splitlines()))
