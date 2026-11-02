# SPDX-License-Identifier: MIT
"""praxis's display transform in numpy, and praxis's capture beside the
Cycles reference. Every run first checks the port against the GPU's own
values in display-ramp.txt, within one 8-bit step.

    blender -b --factory-startup --python-exit-code 1 --python compare.py -- ramp
    ... -- compare <praxis png> <cycles exr> <cycles png> <difference png>
           [--baseline <scene> <frame> <json> [--record]]
    ... -- crop <praxis png> <cycles png> <x> <y> <width> <height> <png>

`compare` writes the Cycles render through the display transform and the
difference: praxis's capture dimmed to a quarter, under red where praxis
is brighter and blue where Cycles is, at four times the channel that
differs most, so a change of hue shows too; it prints the two figures of
their distance, the mean 8-bit difference of their display colours and the
RMSE of their display-linear light. With `--baseline`, it judges the figures
against the scene's Cycles Baseline, `bench/cycles/<scene>.json`, and fails
when either exceeds its recorded value by its tolerance; with `--record`, it
writes that Baseline from this run instead. `crop` writes praxis | Cycles |
difference of a region, each pixel doubled.
"""
import argparse
import json
import os
import subprocess
import sys

import bpy
import numpy as np

# render.py is imported from beside this script, writing no bytecode there.
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from render import arguments, require_blender  # noqa: E402

# The constants and matrices of display.wgsl and gt7.wgsl, which cite
# their sources; each function below is its namesake there. GT7's part is
# ported, through gt7.wgsl, from Polyphony Digital's sample implementation,
# gt7_tone_mapping.cpp 1.0 [ref:gt7-tone-mapping-cpp]:
# https://blog.selfshadow.com/publications/s2025-shading-course/pdi/supplemental/gt7_tone_mapping.cpp
#
# MIT License
#
# Copyright (c) 2025 Polyphony Digital Inc.
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
GT7_PAPER_WHITE = 2.5
PQ_M1, PQ_M2 = 0.1593017578125, 78.84375
PQ_C1, PQ_C2, PQ_C3 = 0.8359375, 18.8515625, 18.6875
REC709_TO_REC2020 = np.array([
    [0.6274039, 0.3292830, 0.0433131],
    [0.0690973, 0.9195404, 0.0113623],
    [0.0163914, 0.0880133, 0.8955953],
])
REC2020_TO_REC709 = np.array([
    [1.6604910, -0.5876411, -0.0728499],
    [-0.1245505, 1.1328999, -0.0083494],
    [-0.0181508, -0.1005789, 1.1187297],
])
REC2020_TO_LMS = np.array([[1688, 2146, 262], [683, 2951, 462], [99, 309, 3688]]) / 4096
LMS_TO_ICTCP = np.array([[2048, 2048, 0], [6610, -13613, 7003], [17933, -17390, -543]]) / 4096
ICTCP_TO_LMS = np.array([[1, 0.00860904, 0.11103], [1, -0.00860904, -0.11103], [1, 0.560031, -0.320627]])
LMS_TO_REC2020 = np.array([
    [3.43661, -2.50645, 0.0698454],
    [-0.79133, 1.9836, -0.192271],
    [-0.0259499, -0.0989137, 1.12486],
])
EIGHT_BIT_STEP = 1.0 / 255.0
# How far a run may exceed its Cycles Baseline before it fails. Three
# captures and renders of every exported scene at one commit on the M4 Pro
# spread each figure by under 0.000001 steps and 0.0000001 RMSE (the
# captures repeat bit for bit, Cycles' Metal render to within 0.005 on a
# few pixels), so these are ten times the Baseline's own rounding.
MEAN_DIFFERENCE_TOLERANCE_STEPS = 0.01
LINEAR_RMSE_TOLERANCE = 0.0001


def smoothstep(edge0, edge1, x):
    t = np.clip((x - edge0) / (edge1 - edge0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)


def gt7_curve(frame_buffer):
    peak, alpha, mid_point, linear_section, toe_strength = GT7_PAPER_WHITE, 0.25, 0.538, 0.444, 1.28
    shoulder_rise = (linear_section - 1.0) / (alpha - 1.0)
    shoulder_limit = peak * linear_section + peak * shoulder_rise
    shoulder_scale = -peak * shoulder_rise * np.exp(linear_section / shoulder_rise)
    shoulder_rate = -1.0 / (shoulder_rise * peak)
    linear_weight = smoothstep(0.0, mid_point, frame_buffer)
    toe = mid_point * np.power(np.maximum(frame_buffer, 0.0) / mid_point, toe_strength)
    low = (1.0 - linear_weight) * toe + linear_weight * frame_buffer
    high = shoulder_limit + shoulder_scale * np.exp(frame_buffer * shoulder_rate)
    return np.where(frame_buffer < 0.0, 0.0, np.where(frame_buffer < linear_section * peak, low, high))


def pq_encode(frame_buffer):
    normalized = np.power(np.maximum(frame_buffer, 0.0) * (100.0 / 10000.0), PQ_M1)
    return np.power((PQ_C1 + PQ_C2 * normalized) / (1.0 + PQ_C3 * normalized), PQ_M2)


def pq_decode(signal):
    power = np.power(np.clip(signal, 0.0, 1.0), 1.0 / PQ_M2)
    linear = np.maximum(power - PQ_C1, 0.0) / (PQ_C2 - PQ_C3 * power)
    return np.power(linear, 1.0 / PQ_M1) * (10000.0 / 100.0)


def rec2020_to_ictcp(rgb):
    return pq_encode(rgb @ REC2020_TO_LMS.T) @ LMS_TO_ICTCP.T


def ictcp_to_rec2020(ictcp):
    return np.maximum(pq_decode(ictcp @ ICTCP_TO_LMS.T) @ LMS_TO_REC2020.T, 0.0)


def gt7_tonemap(rgb):
    ictcp = rec2020_to_ictcp(rgb)
    skewed = gt7_curve(rgb)
    skewed_intensity = rec2020_to_ictcp(skewed)[..., 0]
    paper_white_intensity = rec2020_to_ictcp(np.full(3, GT7_PAPER_WHITE))[0]
    chroma_scale = 1.0 - smoothstep(0.98, 1.16, ictcp[..., 0] / paper_white_intensity)
    scaled = ictcp_to_rec2020(np.stack(
        [skewed_intensity, ictcp[..., 1] * chroma_scale, ictcp[..., 2] * chroma_scale], -1))
    return np.minimum(0.4 * skewed + 0.6 * scaled, GT7_PAPER_WHITE) / GT7_PAPER_WHITE


def tonemap(exposed):
    display = gt7_tonemap((exposed @ REC709_TO_REC2020.T) * GT7_PAPER_WHITE)
    return np.clip(display @ REC2020_TO_REC709.T, 0.0, 1.0)


def srgb_encode(display):
    clamped = np.clip(display, 0.0, 1.0)
    return np.where(clamped <= 0.0031308, 12.92 * clamped, 1.055 * np.power(clamped, 1.0 / 2.4) - 0.055)


def srgb_decode(encoded):
    clamped = np.clip(encoded, 0.0, 1.0)
    return np.where(clamped <= 0.04045, clamped / 12.92, np.power((clamped + 0.055) / 1.055, 2.4))


def display(exposed):
    """sRGB-encoded display colour of exposed linear Rec. 709 colour, as the
    tonemapping pass writes it before its dither."""
    return srgb_encode(tonemap(np.asarray(exposed, np.float64)))


def check_ramp():
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "display-ramp.txt")
    ramp = np.loadtxt(path, comments="#")
    error = np.abs(display(ramp[:, :3]) - ramp[:, 3:]).max()
    if error > EIGHT_BIT_STEP:
        sys.exit(f"FAIL  display ramp: the port is {error * 255:.2f} 8-bit steps from the GPU")
    print(f"ok    display ramp within {error * 255:.3f} of an 8-bit step of the GPU")


def load(path):
    """An image's raw values, rows top-down, as RGB."""
    image = bpy.data.images.load(os.path.abspath(path))
    image.colorspace_settings.name = "Non-Color"
    width, height = image.size
    pixels = np.empty(width * height * 4, np.float32)
    image.pixels.foreach_get(pixels)
    bpy.data.images.remove(image)
    return pixels.reshape(height, width, 4)[::-1, :, :3].astype(np.float64)


def save(rgb, path):
    height, width, _ = rgb.shape
    image = bpy.data.images.new(os.path.basename(path), width, height, alpha=False)
    image.colorspace_settings.name = "Non-Color"
    rgba = np.ones((height, width, 4), np.float32)
    rgba[:, :, :3] = np.clip(rgb[::-1], 0.0, 1.0)
    image.pixels.foreach_set(rgba.ravel())
    image.filepath_raw = os.path.abspath(path)
    image.file_format = "PNG"
    image.save()
    bpy.data.images.remove(image)


def difference(praxis, cycles):
    channels = praxis - cycles
    largest = np.abs(channels).argmax(-1)[..., None]
    signed = np.take_along_axis(channels, largest, -1)[..., 0]
    shown = 0.25 * praxis.mean(-1, keepdims=True).repeat(3, -1)
    shown[..., 0] += np.clip(4.0 * signed, 0.0, 1.0)
    shown[..., 2] += np.clip(-4.0 * signed, 0.0, 1.0)
    return shown


def figures(praxis, cycles_linear):
    """How far praxis's capture is from the Cycles render through the
    display transform: the mean difference of their sRGB-encoded display
    colours in 8-bit steps, and the RMSE of their display-linear light."""
    mean_steps = np.abs(praxis - srgb_encode(cycles_linear)).mean() * 255.0
    linear_rmse = np.sqrt(np.mean((srgb_decode(praxis) - cycles_linear) ** 2))
    return mean_steps, linear_rmse


def record_baseline(path, scene, frame, shape, mean_steps, linear_rmse):
    """The scene's Cycles Baseline from this run: both figures at the
    checked-out commit, with the frame, resolution and Blender that made them."""
    repository = os.path.dirname(os.path.abspath(__file__))
    commit = subprocess.run(["git", "-C", repository, "rev-parse", "HEAD"], capture_output=True, text=True,
                            check=True).stdout.strip()
    baseline = {
        "scene": scene, "frame": frame, "width": shape[1], "height": shape[0], "commit": commit,
        "blender": bpy.app.version_string,
        "mean_difference_steps": round(mean_steps, 3), "linear_rmse": round(linear_rmse, 5),
    }
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "w") as file:
        json.dump(baseline, file, indent=2)
        file.write("\n")
    print(f"ok    {scene} Baseline recorded: mean difference {mean_steps:.3f} 8-bit steps, "
          f"linear RMSE {linear_rmse:.5f}, {path}")


def judge_baseline(path, scene, frame, shape, mean_steps, linear_rmse):
    """One line against the scene's Cycles Baseline; the process fails when
    either figure exceeds its recorded value by its tolerance."""
    with open(path) as file:
        baseline = json.load(file)
    recorded = (baseline["scene"], baseline["frame"], baseline["width"], baseline["height"])
    if recorded != (scene, frame, shape[1], shape[0]):
        sys.exit(f"FAIL  {scene} ran frame {frame} at {shape[1]}x{shape[0]}, its Baseline is {recorded[0]} "
                 f"frame {recorded[1]} at {recorded[2]}x{recorded[3]}")
    baseline_steps, baseline_rmse = baseline["mean_difference_steps"], baseline["linear_rmse"]
    summary = (f"{scene} mean difference {mean_steps:.3f} 8-bit steps (Baseline {baseline_steps:.3f}), "
               f"linear RMSE {linear_rmse:.5f} (Baseline {baseline_rmse:.5f})")
    exceeded = []
    if mean_steps > baseline_steps + MEAN_DIFFERENCE_TOLERANCE_STEPS:
        exceeded.append(f"mean difference past {baseline_steps + MEAN_DIFFERENCE_TOLERANCE_STEPS:.3f}")
    if linear_rmse > baseline_rmse + LINEAR_RMSE_TOLERANCE:
        exceeded.append(f"linear RMSE past {baseline_rmse + LINEAR_RMSE_TOLERANCE:.5f}")
    if exceeded:
        sys.exit(f"FAIL  {summary}: {', '.join(exceeded)}")
    print(f"ok    {summary}")


def compare_arguments(rest):
    parser = argparse.ArgumentParser(prog="compare.py -- compare", add_help=False)
    for name in ("praxis_path", "exr_path", "cycles_path", "difference_path"):
        parser.add_argument(name)
    parser.add_argument("--baseline", nargs=3, metavar=("SCENE", "FRAME", "JSON"))
    parser.add_argument("--record", action="store_true")
    return parser.parse_args(rest)


if __name__ == "__main__":
    require_blender()
    check_ramp()
    command, *rest = arguments()
    if command == "compare":
        options = compare_arguments(rest)
        praxis, cycles_linear = load(options.praxis_path), tonemap(load(options.exr_path))
        if praxis.shape != cycles_linear.shape:
            sys.exit(f"FAIL  {options.praxis_path} is {praxis.shape[:2]}, the Cycles render {cycles_linear.shape[:2]}")
        save(srgb_encode(cycles_linear), options.cycles_path)
        save(difference(praxis, srgb_encode(cycles_linear)), options.difference_path)
        mean_steps, linear_rmse = figures(praxis, cycles_linear)
        if options.baseline is None:
            print(f"ok    mean difference {mean_steps:.3f} 8-bit steps, linear RMSE {linear_rmse:.5f}")
        else:
            scene, frame, path = options.baseline[0], int(options.baseline[1]), options.baseline[2]
            judge = record_baseline if options.record else judge_baseline
            judge(path, scene, frame, praxis.shape, mean_steps, linear_rmse)
    elif command == "crop":
        praxis_path, cycles_path, *region, output = rest
        x, y, width, height = (int(value) for value in region)
        praxis = load(praxis_path)[y:y + height, x:x + width]
        cycles = load(cycles_path)[y:y + height, x:x + width]
        gap = np.ones((height, 4, 3))
        joined = np.concatenate([praxis, gap, cycles, gap, difference(praxis, cycles)], 1)
        save(joined.repeat(2, 0).repeat(2, 1), output)
    elif command != "ramp":
        sys.exit(f"unknown command {command}")
