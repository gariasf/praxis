// GT7's tone mapping (Polyphony Digital, SIGGRAPH 2025 Physically Based
// Shading course [ref:gt7-slides],
// blog.selfshadow.com/publications/s2025-shading-course/pdi/s2025_pbs_pdi_slides_v1.1.pdf),
// its SDR path in ICtCp, ported from Polyphony's sample implementation,
// gt7_tone_mapping.cpp 1.0 [ref:gt7-tone-mapping-cpp]:
// https://blog.selfshadow.com/publications/s2025-shading-course/pdi/supplemental/gt7_tone_mapping.cpp
//
// MIT License
//
// Copyright (c) 2025 Polyphony Digital Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// GT7's SDR paper white, 250 cd/m², in its frame-buffer units of 100 cd/m²:
// GT7 maps SDR to it [ref:gt7-slides].
const GT7_PAPER_WHITE: f32 = 2.5;

// SMPTE ST 2084's (PQ) constants, as the sample writes them and ITU-R
// BT.2100-3 gives them [ref:bt2100-pq].
const PQ_M1: f32 = 0.1593017578125;
const PQ_M2: f32 = 78.84375;
const PQ_C1: f32 = 0.8359375;
const PQ_C2: f32 = 18.8515625;
const PQ_C3: f32 = 18.6875;

// The GT tone curve at GT7's parameters, of a frame-buffer value, peaking at
// GT7_PAPER_WHITE: a power toe blended into a linear section that runs to
// 0.444 of the peak, then an exponential shoulder towards 1.19 times the
// peak, which `gt7_tonemap` clamps to the peak.
fn gt7_curve(frame_buffer: f32) -> f32 {
    let peak = GT7_PAPER_WHITE;
    let alpha = 0.25;
    let mid_point = 0.538;
    let linear_section = 0.444;
    let toe_strength = 1.28;
    // The shoulder's rise past the linear section's end, over the peak: the
    // sample's k.
    let shoulder_rise = (linear_section - 1.0) / (alpha - 1.0);
    let shoulder_limit = peak * linear_section + peak * shoulder_rise;
    let shoulder_scale = -peak * shoulder_rise * exp(linear_section / shoulder_rise);
    let shoulder_rate = -1.0 / (shoulder_rise * peak);
    if frame_buffer < 0.0 {
        return 0.0;
    }
    if frame_buffer < linear_section * peak {
        let linear_weight = smoothstep(0.0, mid_point, frame_buffer);
        let toe = mid_point * pow(frame_buffer / mid_point, toe_strength);
        return (1.0 - linear_weight) * toe + linear_weight * frame_buffer;
    }
    return shoulder_limit + shoulder_scale * exp(frame_buffer * shoulder_rate);
}

// PQ's inverse EOTF: frame-buffer values, 100 cd/m² each, to PQ signal.
fn pq_encode(frame_buffer: vec3<f32>) -> vec3<f32> {
    let normalized = pow(max(frame_buffer, vec3<f32>(0.0)) * (100.0 / 10000.0), vec3<f32>(PQ_M1));
    return exp2(PQ_M2 * (log2(PQ_C1 + PQ_C2 * normalized) - log2(1.0 + PQ_C3 * normalized)));
}

// PQ's EOTF: PQ signal, clamped to [0, 1], to frame-buffer values.
fn pq_decode(signal: vec3<f32>) -> vec3<f32> {
    let power = pow(clamp(signal, vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(1.0 / PQ_M2));
    let linear = max(power - PQ_C1, vec3<f32>(0.0)) / (PQ_C2 - PQ_C3 * power);
    return pow(linear, vec3<f32>(1.0 / PQ_M1)) * (10000.0 / 100.0);
}

// ICtCp from linear Rec. 2020 frame-buffer values (ITU-R BT.2100-3, Table 7
// [ref:bt2100-ictcp]); the inverse below is the sample's, to six digits.
fn rec2020_to_ictcp(rgb: vec3<f32>) -> vec3<f32> {
    let lms = pq_encode(vec3<f32>(
        dot(rgb, vec3<f32>(1688.0, 2146.0, 262.0)),
        dot(rgb, vec3<f32>(683.0, 2951.0, 462.0)),
        dot(rgb, vec3<f32>(99.0, 309.0, 3688.0)),
    ) / 4096.0);
    return vec3<f32>(
        dot(lms, vec3<f32>(2048.0, 2048.0, 0.0)),
        dot(lms, vec3<f32>(6610.0, -13613.0, 7003.0)),
        dot(lms, vec3<f32>(17933.0, -17390.0, -543.0)),
    ) / 4096.0;
}

// Linear Rec. 2020 frame-buffer values from ICtCp, negatives clamped to 0.
fn ictcp_to_rec2020(ictcp: vec3<f32>) -> vec3<f32> {
    let lms = pq_decode(vec3<f32>(
        dot(ictcp, vec3<f32>(1.0, 0.00860904, 0.11103)),
        dot(ictcp, vec3<f32>(1.0, -0.00860904, -0.11103)),
        dot(ictcp, vec3<f32>(1.0, 0.560031, -0.320627)),
    ));
    return max(vec3<f32>(
        dot(lms, vec3<f32>(3.43661, -2.50645, 0.0698454)),
        dot(lms, vec3<f32>(-0.79133, 1.9836, -0.192271)),
        dot(lms, vec3<f32>(-0.0259499, -0.0989137, 1.12486)),
    ), vec3<f32>(0.0));
}

// GT7's SDR tone mapping of linear Rec. 2020 frame-buffer values to [0, 1]:
// the curve per channel, blended 40/60 with that result's intensity carrying
// the input's chroma in ICtCp, which fades to white as the input's intensity
// passes the paper white's.
fn gt7_tonemap(rgb: vec3<f32>) -> vec3<f32> {
    let ictcp = rec2020_to_ictcp(rgb);
    let skewed = vec3<f32>(gt7_curve(rgb.r), gt7_curve(rgb.g), gt7_curve(rgb.b));
    let skewed_intensity = rec2020_to_ictcp(skewed).x;
    let paper_white_intensity = rec2020_to_ictcp(vec3<f32>(GT7_PAPER_WHITE)).x;
    let chroma_scale = 1.0 - smoothstep(0.98, 1.16, ictcp.x / paper_white_intensity);
    let scaled = ictcp_to_rec2020(vec3<f32>(skewed_intensity, ictcp.yz * chroma_scale));
    let blended = 0.4 * skewed + 0.6 * scaled;
    return min(blended, vec3<f32>(GT7_PAPER_WHITE)) / GT7_PAPER_WHITE;
}
