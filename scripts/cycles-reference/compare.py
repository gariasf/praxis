# SPDX-License-Identifier: MIT
"""praxis's display transform in numpy, and praxis's capture beside the
Cycles reference. Every run first checks the port against the GPU's own
values in display-ramp.txt, within one 8-bit step.

    blender -b --factory-startup --python-exit-code 1 --python compare.py -- ramp
    ... -- compare <praxis png> <cycles exr> <cycles png> <difference png>
    ... -- crop <praxis png> <cycles png> <x> <y> <width> <height> <png>

`compare` writes the Cycles render through the display transform and the
difference: praxis's capture dimmed to a quarter, under red where praxis
is brighter and blue where Cycles is, at four times the mean of the
channels' differences. `crop` writes praxis | Cycles | difference of a
region, each pixel doubled.
"""
import os
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
    signed = (praxis - cycles).mean(-1)
    shown = 0.25 * praxis.mean(-1, keepdims=True).repeat(3, -1)
    shown[..., 0] += np.clip(4.0 * signed, 0.0, 1.0)
    shown[..., 2] += np.clip(-4.0 * signed, 0.0, 1.0)
    return shown


if __name__ == "__main__":
    require_blender()
    check_ramp()
    command, *rest = arguments()
    if command == "compare":
        praxis_path, exr_path, cycles_path, difference_path = rest
        praxis, cycles = load(praxis_path), display(load(exr_path))
        if praxis.shape != cycles.shape:
            sys.exit(f"FAIL  {praxis_path} is {praxis.shape[:2]}, the Cycles render {cycles.shape[:2]}")
        save(cycles, cycles_path)
        save(difference(praxis, cycles), difference_path)
        steps = np.abs(praxis - cycles).mean() * 255.0
        print(f"ok    mean difference {steps:.2f} 8-bit steps")
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
