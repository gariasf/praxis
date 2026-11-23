## Parent

#994 (M5.5 spec), stories 19 to 26; lane 4 of `docs/agents/audit.md`, whose evidence this is; the charting [decision](https://github.com/gariasf/praxis/issues/17#issuecomment-7081648200), item 3.

## Question

What do probe-shaped rays cost on the acceleration structures as they stand, on the M4 Pro? ADR 0020 gives the probe update 2 ms of GPU. No probe ray has been traced in a praxis scene: the survey's figure (2,097,152 rays in 5.5 to 6.5 ms) is from a scratch program on a synthetic scene, taken before Far proxies were in the static TLAS, and a shadow ray's cost does not carry over, since shadow rays are coherent and stop at any hit. M6's charting needs the table.

## What to build

A prototype branch, pushed and never merged, with no PR, Golden or Baseline. A compute pass traces rays from a 32×8×32 grid of probes 4 m apart around the camera, in uniform directions, to the closest hit in both TLAS; it fetches the hit triangle for a normal and traces one sun shadow ray from the hit. It shades nothing. Ray count, ray length, the dynamic TLAS's fill and the shader runtime checks are switches of the prototype only.

The rows, each the GPU time of the pass:

| Scene | Rays a frame | Varied |
|---|---|---|
| `world`, its densest 2 km view | 65,536; 524,288; 2,097,152 | ray length 256 m and 2 km |
| `town` | 65,536; 524,288; 2,097,152 | the dynamic TLAS empty and at 10,000 Moving Instances (625 of `crowd`'s figures, 16 Shadow proxies each) |
| `town`, dynamic TLAS at 10,000 | none | that TLAS's build: CPU and GPU time |
| one row of the above, chosen as the nearest to 2 ms | as that row | wgpu's shader runtime checks off |

## How it is judged

Each figure is the median of three interleaved rounds in a quiet turn on the M4 Pro, published as Experiment records (`praxis bench --experiment <name>`, or `praxis publish` of the results). The resolution comment gives the table, the run conditions (adapter, the branch's commit, resolution, load average, `pmset -g batt`, peers), and, computed from it, how many rays 2 ms buys on each scene. It sets no threshold and recommends no update form: both are M6's charting. The RTX 2070 SUPER's leg is its own ticket, run from this branch.

## Start here

- `src/render/shadow_mask_pass.rs` and `src/render/shaders/shadow_mask.wgsl`: ray queries in a compute pass against the static and the dynamic TLAS, the form to follow.
- `src/render/acceleration_structures.rs`: `DYNAMIC_TLAS_CAPACITY`, and the static TLAS's Anchor, in whose frame a ray is composed (ADR 0014).
- `src/scenes/mod.rs`, `spawn_crowd_as` with `CrowdForm::Proxies`: the figures that fill the dynamic TLAS.
- `src/scenes/world.rs`, `the_densest_view_fits_the_capacities_the_budgets_give`: which view is the densest.
- `src/render/shaders/mod.rs`: where `wgpu::ShaderRuntimeChecks` are set per module; the branch `prototype/873-shadow-mask-checks` and #873's comments for what turning them off involved.
- `docs/research/global-illumination-and-lighting.md`, "Measured on the M4 Pro": the scratch program's method, for a like-for-like reading.
- `docs/agents/machine.md`: Quiet turns and Timed runs.
- Known costs to keep apart from the rays': a ray-query pass's timestamps on the M4 Pro are checked against the time of a whole submission of repeated passes before they are trusted; reading a hit's Instance data cost shadow rays about 5% there (#880), and the triangle fetch needs it.
- Ruled out: Material or light work at a hit; an irradiance blend; a scene variant on `main`; a threshold.

## Acceptance criteria

- [ ] The branch is pushed as `prototype/<this ticket>-probe-ray-pricing` and opens no PR.
- [ ] The resolution comment holds every row of the table with its run conditions, and the rays 2 ms buys per scene.
- [ ] The Experiment records are on the Bench history, and the comment names them.
- [ ] If meshopt 0.7 is not in the frozen commit, the comment says the figures were taken on 0.6's Far proxies.

## Verification seam

The Experiment records and the resolution comment. Size: prototype, not merged.

## Blocked by

- #997

