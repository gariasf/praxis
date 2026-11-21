# Simulated vehicles: state of the art

Researched 2026-11-21 for ticket #952 (parent #944). The owner's bar for cars, set that day: fully simulated, in the class of Spartan Engine's car and Grand Theft Auto V, with gears, suspension and body sway. This note says what such a wheeled vehicle is made of, who provides which part, and what it asks of the rest of the engine. It ends in options and chooses none, and it chooses no physics engine.

**Summary.** The bar is met by a vehicle model written above a rigid-body engine, not by any engine's vehicle module, so it does not decide the physics engine.

- **Parts against providers.** Jolt 5.7.0 and PhysX 5.13.0 model the driveline (torque curve, clutch, gearbox, differentials) and anti-roll bars, and simplify the rest: a spring and damper along one direction per wheel, a tyre that is a friction curve, no aerodynamics. Rapier, Bullet and Godot hold a ray-cast vehicle with no driveline at all. Spartan Engine's car, the bar, uses none of PhysX's vehicle module: it is its own model on PhysX bodies, joints and sweeps, with a Magic Formula tyre, a suspension of links and its own driveline and aerodynamics.
- **The step.** Jolt's and PhysX's vehicles step at about the Tick's rate by their own documents: Jolt is stable at 60 Hz with one collision step, and PhysX's sample runs at 60 Hz and adds 3 substeps of the vehicle alone below 5 m/s (192 Hz in a Tick). The fuller models run faster: Spartan's whole scene at 200 Hz with the tyre at 800 Hz, VDrift's car at 900 Hz, Chrono's tyre at 1,000 Hz. In a Tick that is 3 or 4 substeps of the vehicle (192 or 256 Hz) with 4 tyre substeps inside each. One source prices a vehicle: a full Chrono vehicle takes about 0.6 s of one CPU thread per simulated second, on a processor its paper does not name, which is 9.4 ms a Tick (computed) against the 1.84 to 2.26 ms a Tick leaves for physics, animation, AI and Lua together. No source prices Jolt's, PhysX's or Spartan's.
- **The engine.** What the model needs of one is rigid bodies and joints, a shape cast per wheel, a hook that substeps one vehicle inside the Tick, f64 or a frame rebased near the car, and a Rust binding. Jolt, Rapier and PhysX all offer the first four. No Rust binding found covers Jolt's or PhysX's vehicle module, and PhysX is f32 alone.
- **The world at speed.** The streamer's figures were all taken at 50 m/s (180 km/h). From them: at 100 m/s about 6.1 Far proxies and 0.8 Chunks a second enter `world`'s radii, against 180 loads a second allowed, and a load has 2.6 s to finish. Late mips at speed, from the height of a car, are not measured.
- **Options.** Six, from a ray-cast vehicle to a simulation library, in *Options*; six questions for a prototype, the first being what one vehicle costs a Tick on the M4 Pro.

Sources: the entries `docs/research/open-engines-register.md` lists for this survey, each reopened at the commit the register names (Spartan Engine `cb258a1`, Jolt Physics v5.7.0, PhysX SDK 5.13.0, Project Chrono 10.0.0, Rapier `475caa3`, Bullet 3.25, Godot 4.7.2, Wicked Engine v0.72.139, Lumix Engine `8ce5ab5`, Torque3D `9521d43`, VDrift `f4767dd`); PhysX's vehicle guide at 5.8.0, the newest its site lists; the Rust bindings of Jolt and PhysX; the abstracts of three tyre papers; and what the developers of five games have published. Each claim relied on is in `docs/references.toml`, cited by key, and every quote there was matched by script against its fetched source on 2026-11-21. Nothing was measured and no vehicle code was run: a rate or a cost below is its source's.

Licence classes are #944's: **A**, adaptable with citation; **P**, Apache-2.0 alone, usable as a dependency and read as source, adapting it a case for the owner; **E**, evidence only. Spartan Engine's source was opened at `cb258a1`, its last commit under MIT, and nowhere later [ref:spartan-engine-cb258a1-licence]. Unreal, Unity and Flax are their vendors' documentation.

Temporal class: none of the rows below is a lighting term. A vehicle is simulation state, drawn as ADR 0022's blend of two Ticks, which reads no screen-space history.

Boundaries: mounts, boats, aircraft, motorcycles and tracked vehicles, engine sound, and the form of terrain's height query and collision shape, which is #948's.

## Parts against providers

Each cell says **M** (modelled), **S** (simplified) or **–** (absent), as read at the release named.

| Provider: class | Engine, clutch, gearbox | Differentials | Suspension | Anti-roll | Tyres | Aerodynamics | Brakes, steering | Wheel reads the ground by |
|---|---|---|---|---|---|---|---|---|
| Jolt Physics v5.7.0, `WheeledVehicleController`: A | M: normalised torque curve with inertia, gear ratios, automatic or manual shifts; the clutch a linear coupling [ref:jolt-5.7.0-engine] [ref:jolt-5.7.0-transmission] | M: torque split, limited slip as a speed-ratio threshold [ref:jolt-5.7.0-differential] | S: one direction and one spring per wheel, on wheels that are not bodies [ref:jolt-5.7.0-wheel-settings] [ref:jolt-5.7.0-virtual-wheels] | M [ref:jolt-5.7.0-anti-roll] | S: two friction curves, against slip ratio and slip angle, times the suspension's impulse [ref:jolt-5.7.0-tyre] | – | S: a brake and a handbrake torque and a largest steer angle per wheel (`Jolt/Physics/Vehicle/WheeledVehicleController.h`) | a ray, sphere or cylinder cast [ref:jolt-5.7.0-collision-testers] |
| PhysX SDK 5.13.0, vehicle module: P | M: torque graph, clutch at two accuracies, gearbox and autobox [ref:physx-5.13.0-drivetrain] | M: multi-wheel, four-wheel with a limited-slip bias, tank [ref:physx-5.13.0-drivetrain] | S: a linear spring and damper, with graphs of toe, camber and force points against compression [ref:physx-5.13.0-suspension] | M [ref:physx-5.13.0-suspension] | S: stiffness against load and a three-point friction graph against slip [ref:physx-5.13.0-tyre]; a Magic Formula tyre as a sample component [ref:physx-5.13.0-custom-tyre] | – | M: a brake response distributed to the wheels, Ackermann correction (`physx/include/vehicle/braking/PxVehicleBrakingParams.h`, `physx/include/vehicle/steering/PxVehicleSteeringParams.h`) | a ray or a sweep, or a plane the caller gives [ref:physx-5.13.0-road-queries] |
| Spartan Engine `cb258a1`, its own car on PhysX bodies: A | M: torque curve, turbo, 7-speed gearbox, rev-match; propshafts and halfshafts as bodies [ref:spartan-engine-cb258a1-car] [ref:spartan-engine-cb258a1-car-multibody] | M: open, locked, limited slip [ref:spartan-engine-cb258a1-car] | M: links as bodies and joints, bump stops, bump steer, camber and toe [ref:spartan-engine-cb258a1-car-multibody] | M: bodies and a torsion joint [ref:spartan-engine-cb258a1-car-multibody] | M: Magic Formula with load, temperature and surface factors, slip relaxation [ref:spartan-engine-cb258a1-car-tyre] [ref:spartan-engine-cb258a1-tyre-substeps] | M: drag, downforce, ground effect [ref:spartan-engine-cb258a1-car] | M: fade, bias, ABS; Ackermann, self-aligning torque [ref:spartan-engine-cb258a1-car] | sweeps [ref:spartan-engine-cb258a1-car] |
| Project Chrono 10.0.0: A | M: from shafts with a torque converter down to a speed-torque curve [ref:chrono-10.0.0-powertrain] | M: shaft elements (`doxygen/documentation/manuals/vehicle/wheeled_driveline.md`) | M: templates of bodies and joints [ref:chrono-10.0.0-overview] | M (`src/chrono_vehicle/wheeled_vehicle/antirollbar/ChAntirollBarRSD.h`, as the register read it) | M: rigid, Pacejka 1989 and 2002, TMeasy, Fiala, finite element [ref:chrono-10.0.0-tyres] [ref:chrono-10.0.0-pac02] | S: drag on the chassis, off by default [ref:chrono-10.0.0-aerodynamic-drag] | S brakes, M steering linkages (`doxygen/documentation/manuals/vehicle/vehicle_overview.md`) | one point, four points or an envelope [ref:chrono-10.0.0-tyre-step] |
| VDrift `f4767dd`, its own car on Bullet bodies: E | M | M | M: basic, MacPherson and wishbone geometry (`src/physics/carsuspension.cpp`) | M | M: three tyre models, two of them Pacejka [ref:vdrift-f4767dd-tyres] | M: drag and lift devices (`src/physics/aerodevice.h`) | M | a ray [ref:vdrift-f4767dd-step] |
| Torque3D `9521d43`, its own model: A | S: one engine torque scaled by wheel speed, no gearbox [ref:torque3d-9521d43-vehicle] | – | S: a spring and damper per wheel | S: "antiSway" | S: tyres as springs with friction coefficients | not read | S | not read |
| Rapier `475caa3`: P. Bullet 3.25: A, its vehicle files under their author's notice. Godot 4.7.2: A | –: the caller sets a force per wheel [ref:rapier-475caa3-vehicle] [ref:bullet-3.25-vehicle] [ref:godot-4.7.2-vehicle] | – | S: a spring with two damping rates | – | S: a friction limit per wheel | – | S: a brake and a steer angle per wheel, from the caller | one ray |

Engines that add nothing to a provider: Wicked Engine sets up Jolt's controllers [ref:wicked-engine-fc8385b-vehicle], Lumix Engine PhysX 4.1's older `PxVehicleDrive4W` [ref:lumix-8ce5ab5-vehicle], Flax PhysX's vehicle module by its manual [ref:flax-docs-vehicles]. Unity's wheel collider is a ray with "a slip-based tire friction model" [ref:unity-6.6-wheel-collider] [ref:unity-6.6-wheel-ray]. Unreal's documentation lists engine, transmission, clutch, suspension, wheel and aerofoil components, and since 5.8 "two-way interaction between the vehicle and the ground" [ref:ue-5.9-modular-vehicles] [ref:unreal-5.8-chaos-modular-vehicles]; its tyre model is not in the pages read.

## Comparison

| Option | Buys | Published cost, hardware | Step it needs, by its source | Maturity | Fit | Reference code: class |
|---|---|---|---|---|---|---|
| A. Ray-cast vehicle | A body that drives on four springs: steering, a friction limit, no gears | None published | The outer step; Unity splits it into substeps whose counts its page does not give [ref:unity-6.6-vehicle-substeps] | Shipped: Bullet, Godot, Unity | Below the bar; what traffic could be stepped as. Rapier's is Rust | Rapier P, Bullet's files under their author's notice, Godot A |
| B. An engine's vehicle module as shipped | Gears, clutch, differentials, sway against anti-roll bars | None published | 64 Hz: Jolt is "stable when running at 60 Hz with 1 collision step"; PhysX's sample adds 3 substeps under 5 m/s | Shipped engines; Jolt's readme names two shipped titles and does not say whether they use its vehicles | The bar's gears and sway, not its tyre or suspension. A Rust binding of the module, which does not exist | Jolt A, PhysX P |
| C. An engine's driveline, tyre written here | B with a Magic Formula tyre | None published | As B | PhysX: a sample component. Jolt: the callback sets a limit, not a force | PhysX only, f32 and class P | PhysX P |
| D. Own model above rigid bodies, strut suspension | Every part of the model, on any engine; reproducible under ADR 0022 by construction | None published | No source states a floor. Evidence: 200 Hz with the tyre at 800 Hz (Spartan), 900 Hz (VDrift), 360 Hz for the tyre (Forza Motorsport) | An open-source racing game (VDrift); Spartan is by its readme a personal research engine | Engine-agnostic: needs bodies, a shape cast per wheel and a substep hook | Spartan A (at `cb258a1`), VDrift E |
| E. D with suspension and driveline as bodies and joints | Geometry that produces bump steer, camber change and roll centres rather than curves of them | None published | 200 Hz for the whole scene, 64 position iterations (Spartan) | One research engine | Asks the most of the engine's joints and of f32 | Spartan A (at `cb258a1`) |
| F. A simulation library at run time | Validated tyres, linkages, deformable soil | Chrono: about 0.6 s of one CPU thread per simulated second for one vehicle, processor not named | 1 ms (Chrono's tyre default); 0.5 ms (BeamNG) | Engineering software; BeamNG ships a game on its own engine | 4 to 5 times the per-Tick allowance for one vehicle; no Rust binding | Chrono A |

## The model

**Driveline.** An engine is a torque curve against speed, with inertia; a clutch couples it to a gearbox of fixed ratios; differentials split the torque between wheels. The coupling is stiff: Jolt solves engine and wheel speeds together "because the system is very stiff", by implicit Euler [ref:jolt-5.7.0-drivetrain-solve], and PhysX offers its clutch as "a potentially more expensive but more accurate solution" or a cheaper one [ref:physx-5.13.0-drivetrain]. Spartan and Chrono instead build shafts and differentials as bodies or shaft elements [ref:spartan-engine-cb258a1-car-multibody] [ref:chrono-10.0.0-powertrain]. Caveat: a driveline is the part every provider above the ray-cast vehicle already has.

**Suspension.** The simple form is a spring and damper along one direction per wheel, with anti-roll bars as springs between a pair of wheels [ref:jolt-5.7.0-anti-roll]; PhysX adds graphs that turn compression into toe, camber and a moved force point [ref:physx-5.13.0-suspension]. The full form is the linkage itself: Chrono's templates [ref:chrono-10.0.0-overview], and Spartan's links, whose closed loops "need enough position iterations", 64, to hold their joints [ref:spartan-engine-cb258a1-car-multibody]. Caveat: Jolt's wheels are not bodies [ref:jolt-5.7.0-virtual-wheels] and its wheel settings hold no mass, so its model has no unsprung mass.

**Tyres.** Three families. A *friction curve* scales a normalised curve against slip by the load, the form Jolt takes from Marco Monster's notes [ref:monster-car-physics-traction] [ref:jolt-5.7.0-tyre]. An *analytical* model derives the shear forces from load, slip and stiffness, as Dugoff, Fancher and Segel's does [ref:dugoff-1970-tyre-traction], and Fiala's, which Chrono describes as "based on a brush model assumption" [ref:chrono-10.0.0-tyres]. The *Magic Formula* fits measured tyres with "a formula with coefficients which describe some of the typifying quantities of a tyre", for steady states [ref:bakker-1987-magic-formula]. Slip divides by speed, so every slip-based tyre has "numerical problems" near a standstill; Bernard and Clover make slip a state variable [ref:bernard-clover-1995-low-speed-tyre], Chrono's tyres carry slip state equations [ref:chrono-10.0.0-tyre-limits], Spartan relaxes slip over a length and blends to static friction at rest [ref:spartan-engine-cb258a1-tyre-substeps], and PhysX swaps the tyre for a velocity constraint, its "sticky" state [ref:physx-5.8.0-guide-sticky-tyre]. Caveat: Avalanche's talk on Just Cause 4 argues the Magic Formula "may not be optimal for arcade and 'simcade' handling" and proposes a model with fewer parameters [ref:young-2019-just-cause-4-tyres].

**Weight transfer and body roll.** Neither is a model of its own: both fall out of a rigid chassis on springs, and matter because "the weight distribution dramatically affects the maximum traction force per wheel" [ref:monster-car-physics-traction]. What differs is where a tyre's force enters the body: PhysX tunes a force application point [ref:physx-5.13.0-suspension]; Spartan splits lateral transfer between geometry and springs by roll-centre heights [ref:spartan-engine-cb258a1-car].

**Aerodynamics, brakes, steering.** No engine's vehicle module has aerodynamics; Chrono has drag alone, off by default [ref:chrono-10.0.0-aerodynamic-drag]; Spartan and VDrift add downforce or lift. These are forces on the chassis and need nothing of the engine. Brakes are a torque or a force per wheel in every provider read; ABS, fade and Ackermann steering are in Spartan, Ackermann also in PhysX.

## Providers

**Spartan Engine** at `cb258a1`, class A. The bar. Its car uses none of PhysX's vehicle module (no `PxVehicle` name appears under `source/car/` or `source/physics/`): a PhysX chassis body with the tyre, suspension, driveline and aerodynamics computed in its own code each step [ref:spartan-engine-cb258a1-car-source], 20,644 lines under `source/car/`, 6,485 of them in `CarSimulation.cpp` (counted at that commit; they include its HUD, sound and telemetry). The scene steps at a fixed 200 Hz [ref:spartan-engine-cb258a1-physics-step], and inside a step the tyre and the wheel's spin take 4 substeps "against a frozen chassis", because "slip relaxation and a wheel locking are both far faster than one physics step" [ref:spartan-engine-cb258a1-tyre-substeps]. Fit: evidence that the bar is reachable above an engine that supplies bodies, joints and sweeps. Its readme calls it a personal research engine [ref:spartan-engine-cb258a1-readme]; its Magic Formula version is its readme's word.

**Jolt Physics** v5.7.0, class A. The fullest module under MIT. The vehicle is a constraint that steps as a listener of the physics step [ref:jolt-5.7.0-vehicle-constraint]. Its tyre callback computes "max tire impulses", the limit its constraint solver works within, so a tyre written into it shapes the peak and not the force below it [ref:jolt-5.7.0-tyre-max-impulse]. f32 by default with a double-precision build, deterministic for one binary [ref:jolt-5.7.0-precision-determinism]. Binding: the Rust binding its readme names [ref:jolt-5.7.0-readme] wraps Jolt 5.6.0 through a C interface, calls itself "an early work in progress" and passes a `double-precision` feature through [ref:jolt-rust-937c09c-readme]; that C interface names a vehicle once, in an enum of constraint kinds [ref:joltc-429235d-vehicle]. Fit: option B as it stands, and the rigid bodies for D.

**PhysX** SDK 5.13.0, class P. A vehicle is a sequence of components the caller orders, any of which can be replaced: its guide gives "the replacement of the tire component from a low fidelity model to a high fidelty model such as Pacejka" as its example, and lets vehicles run on several threads [ref:physx-5.8.0-guide-components]. f32 [ref:physx-5.13.0-real-is-f32]. Binding: `physx-rs` generates bindings for PhysX 5.1 and its repository is archived [ref:physx-rs-d0d571e-readme]. Fit: the only module where option C is a supported path; f32 and class P weigh against it.

**Rapier** `475caa3`, class P; **Bullet** 3.25, class A; **Godot** 4.7.2, class A. One vehicle three times: Rapier's controller says it was "ported and modified from Bullet" [ref:rapier-475caa3-vehicle], and Godot's keeps Bullet's field names. Bullet's vehicle files carry their author's 2005 notice and not the zlib text [ref:bullet-3.25-vehicle-notice], which #944's filter does not name, so adapting from them, or from Rapier's port, is the owner's call. Godot's own documentation says its vehicle "isn't designed to provide realistic 3D vehicle physics" [ref:godot-4.7.2-vehicle]. Rapier is Rust, with an f64 crate [ref:rapier-475caa3-f64] and local determinism [ref:rapier-475caa3-determinism]. Fit: below the bar as a car; option A.

**Project Chrono** 10.0.0, class A. The upper bound: a multi-physics package in f64 whose vehicle is co-simulated subsystems [ref:chrono-10.0.0-overview]. Its tyre step defaults to 1 ms and a demo steps at 2 ms [ref:chrono-10.0.0-tyre-step] [ref:chrono-10.0.0-demo-step]. No Rust binding was found. Fit: option F, and an independent reference to test a model against, as Cycles is for images.

**VDrift** `f4767dd`, class E, evidence only. A second own model above an engine's rigid bodies, Bullet's here: the game steps its world at 1/90 s and the car takes 10 substeps of its own [ref:vdrift-f4767dd-game-step] [ref:vdrift-f4767dd-car-substeps]. **Torque3D** `9521d43`, class A: a third, with no gearbox [ref:torque3d-9521d43-vehicle]; below the bar.

## The step

Physics steps once per 64 Hz Tick and substeps inside it (ADR 0022). Rates below are their sources'; the multiples of 64 are computed.

| Source | Rate | In a 15.625 ms Tick |
|---|---|---|
| Jolt: "stable when running at 60 Hz with 1 collision step"; a spring may reach half the step rate [ref:jolt-5.7.0-precision-determinism] [ref:jolt-5.7.0-spring-frequency] | 60 Hz | 1 step; springs to 32 Hz, against a default suspension of 1.5 Hz [ref:jolt-5.7.0-wheel-settings] |
| PhysX guide: "a minimum of five simulation updates per spring cycle" [ref:physx-5.8.0-guide-spring-updates] | by spring | 6.4 to 12.8 updates a cycle for its "typical family car", by its formula |
| PhysX sample: 60 Hz, 3 substeps under 5 m/s [ref:physx-5.13.0-substeps] | 180 Hz at low speed | 3 substeps, 192 Hz |
| Spartan: 200 Hz, tyre 4 substeps [ref:spartan-engine-cb258a1-physics-step] [ref:spartan-engine-cb258a1-tyre-substeps] | 200 and 800 Hz | 3 or 4 substeps (192 or 256 Hz); tyre 768 or 1,024 Hz |
| Forza Motorsport: tyre "8 points of contact [...] at 360 cycles per second", from one point at 60 [ref:forza-motorsport-2023-tyre] | 360 Hz | 6 substeps, 384 Hz |
| VDrift: 1/90 s, 10 car substeps [ref:vdrift-f4767dd-game-step] [ref:vdrift-f4767dd-car-substeps] | 900 Hz | 14 substeps, 896 Hz |
| Chrono: tyre step 1 ms by default [ref:chrono-10.0.0-tyre-step] | 1,000 Hz | 16 substeps, 1,024 Hz |
| BeamNG: "exactly 0.5ms", a soft body of nodes and beams on a thread of its own [ref:beamng-physics-step] [ref:beamng-soft-body] | 2,000 Hz | 32 substeps, 2,048 Hz |

**What is substepped.** Both PhysX and Spartan substep the vehicle and not the world. PhysX's group repeats suspension, tyre, drivetrain and the vehicle's rigid body "without recalculating the plane underneath the wheel", which is "much cheaper than setting a smaller timestep for the whole vehicle" [ref:physx-5.13.0-substep-group]; Spartan's tyre runs "against a frozen chassis" and hands PhysX the average. A vehicle substep is therefore a loop inside one system of the fixed schedule, not a faster Tick.

**Against the Tick.** The line on #17 leaves 1.84 to 2.26 ms a Tick on the M4 Pro for physics, animation, AI and Lua together, 12 to 14% of a Tick (computed). The one published cost per vehicle is Chrono's, 9.4 ms a Tick on one thread of an unnamed processor (0.6 × 15.625 ms, computed) [ref:zhou-2023-chrono-hil-real-time-factor], 4.1 to 5.1 times the whole allowance; the same paper finds real time hard past 30 such vehicles across processes. One budget from a game of this kind: Just Cause 4 gave all its physics "8.5 ms on 4 threads out of 33 ms", 26% of a 30 fps frame, on Xbox One and PlayStation 4 [ref:kerner-2019-just-cause-4-physics-budget]. What options B to E cost per vehicle no source says.

**Reproducibility.** A substep count that depends only on simulation state, as PhysX's sample's does on forward speed, keeps ADR 0022's guarantee for one binary. Jolt and Rapier state the same guarantee for themselves [ref:jolt-5.7.0-precision-determinism] [ref:rapier-475caa3-determinism]. A model written in Rust inside the fixed schedule inherits ADR 0022's rules; one behind a C++ library inherits that library's.

## Positions

ADR 0014 asks physics for f64 internals or f32 frames within 1 km of their origin. A car at 100 m/s leaves a 1 km frame in 10 s (computed). Jolt has a double-precision build and Rapier an f64 crate; PhysX is f32 and moves its origin. Spartan, on PhysX, moves the origin once the camera is 16 m from it, because snapping to a grid "left up to 32 m of residual coordinates [...] enough to perturb the small TGS suspension/steering increments at high tick rates" [ref:spartan-engine-cb258a1-origin-rebase]. f32's spacing is 0.0038 mm between 32 and 64 m and 0.061 mm between 512 and 1,024 m (computed), so a suspension of joints at 200 Hz, option E, is evidence that the 1 km clause may be too loose for it; whether it is, is a prototype's question. An f32 frame also has one origin: vehicles simulated far from the camera each need a frame near them, which f64 does not ask.

## The ground

A wheel reads, each step: a contact point and normal, the surface's kind, and the velocity of what it stands on (the fields of Jolt's `Wheel`). Providers ask with one ray (Rapier, Bullet, Godot, Unity), a sphere or cylinder cast (Jolt), a sweep (PhysX, Spartan), or several points across the tread (Forza's 8, Chrono's four points or envelope). The query runs once a step and substeps reuse its plane [ref:physx-5.13.0-substep-group]; PhysX also accepts a plane and friction from the caller in place of a scene query [ref:physx-5.13.0-road-queries], which is where a height query would enter. So terrain owes a wheel a height, a normal and a surface kind at a point, at least four times per vehicle per Tick, and a shape a cast can hit where a kerb or a rock matters; their form is #948's. Soft ground: Chrono's handling tyres "are not suitable for off-road vehicle simulation, i.e. when using a deformable terrain" [ref:chrono-10.0.0-tyre-limits], its soil model computing contact "in close to real-time" [ref:chrono-10.0.0-scm-terrain]; Spartan scales friction by surface kind [ref:spartan-engine-cb258a1-car-tyre]. Water under a car is #950's buoyancy.

## The world at speed

Figures the repo holds: `world` has 256 m Chunks, a 256 m Load radius and a 2 km View distance (`src/scenes/mod.rs`); its bench path and its traversal fly at 50 m/s, 60 m up (`src/scenes/world.rs`); the bench path's Baseline on the M4 Pro reads 0 Late Chunks, 0 Late Far proxies and 8 Late mips over 42 Chunks loaded (`bench/baselines/apple-m4-pro/world.json`), and the traversal at M4's exit, before Far proxies, read 0 Late Chunks and 230 Late mips (`docs/performance.md`, M4 exit, the bench rerun); on that traversal 274 loads took 1 to 6 frames, so the streamer's lead stayed at its floor of 16 frames (the same file, M4 exit, Also seen); a frame makes at most 3 loads (`src/streaming/io.rs`); a Chunk is requested when the camera is within the Load radius plus its closing speed times the lead, and never farther out than the Load radius plus 256 m (`src/streaming/mod.rs`). The pre-mortem's 3 new Chunks a second is for a 2 km radius at 50 m/s.

Computed from those, at 60 frames a second:

| | 50 m/s (180 km/h) | 100 m/s (360 km/h) |
|---|---|---|
| Chunks entering the 256 m Load radius | 0.39 a second | 0.78 a second |
| Far proxies entering the 2 km View distance | 3.1 a second | 6.1 a second |
| Loads allowed | 180 a second | 180 a second |
| Distance the 16-frame lead requests ahead | 13 m | 27 m |
| Longest a load may take and not be late | 5.1 s, 307 frames | 2.6 s, 153 frames |
| Loaded ground ahead of the car | 5.1 s | 2.6 s |

`bench/budgets.json` holds no Budget stated against a speed: its lines are a frame's times and memory, and Late Chunks, Late Far proxies and Late mips are counters held to Baselines recorded on a 50 m/s path. The streamer's tests fly at 50 m/s too, and nothing in the repo is measured faster. By these figures the rate of arrivals and the lead hold at twice that with a wide margin, while loads keep taking 1 to 6 frames. Three things the figures do not cover: Late mips, whose count at 50 m/s was taken from 60 m up and not from the road; the slowest frame when loads bunch; and collision. Physics needs ground under every vehicle it steps, and the Load radius is measured from the camera, so a vehicle stepped beyond it has nothing to stand on.

## Drawing and tracing

**Moving Instances.** A car is at least a body and four wheels, 5 Moving Instances; with every member of Spartan's suspension drawn, up to 37 (`max_suspension_members = 8` per corner in `source/car/CarMultibody.h`; computed). The dynamic TLAS's cap of 10,000 then holds 2,000 or 270 cars (computed), and a Moving Instance past it casts no shadow (`docs/research/world-design-pre-mortem.md`).

**Interpolation.** ADR 0022 blends the previous and current World transform of each Moving Instance. A wheel of 0.3 m radius, Jolt's default, turns half a revolution per Tick at 60 m/s and 0.83 of one at 100 m/s (computed), and a blend of two rotations takes the shorter way round. A wheel's spin would need to be blended as an angle, a rule beside the blend that no line holds.

**Damage.** A body Mesh deformed at run time meets what the pre-mortem lists for fracture: arena room, a Mesh that bypasses cooking, and a BLAS that wgpu-core rebuilds in full [ref:wgpu-30-blas-rebuild-only], 0.6 to 0.75 ms of GPU for 5,120 triangles by the pre-mortem's measurement. The `crowd` scene already holds the three forms a deforming Mesh can take: a refit through wgpu-hal on Metal alone, a rebuild, and Shadow proxies (`docs/guides/blas-refit.html`). Swapping in cooked damaged Meshes avoids all three. How Grand Theft Auto V deforms its cars was not found in its developers' words.

## Many vehicles

No source read steps every vehicle in full. Spartan has two modes, "full = multibody tires, cheap = arcade chassis plus visual wheels" [ref:spartan-engine-cb258a1-cheap-mode]. Chrono's traffic framework keeps the full vehicle for the driver and a reduced model "for vehicles not in focus" [ref:zhou-2023-chrono-hil-real-time-factor]. Cyberpunk 2077's patch notes list engine, clutch and gearbox simulation under driving, and for traffic a "Visual improvement to traffic movement: turning and suspension" [ref:cyberpunk-2077-1.5-vehicles]. The tiers this suggests: the full model for the player's car and those near it; option A's body on four rays for traffic that can be hit; a transform moved along a lane, with no physics and no ground, for the rest; and nothing but a count beyond the Load radius.

## What shipped games say

By their developers' own account: Forza Motorsport's tyre has 8 contact points at 360 Hz [ref:forza-motorsport-2023-tyre]; BeamNG simulates the car as a soft body at 2,000 Hz [ref:beamng-physics-step]; Cyberpunk 2077 added clutch and ABS simulation and reworked its gearbox in a patch [ref:cyberpunk-2077-1.5-vehicles]; Just Cause 4 weighed the Magic Formula and chose a simpler tyre [ref:young-2019-just-cause-4-tyres]; Uncharted 4's 4x4 has a motor simulation and a tyre collision response [ref:pereira-2017-uncharted-4-4x4]. For Grand Theft Auto V no developers' account of what is simulated was found. What its cars appear to do, in press comparisons with Grand Theft Auto IV, is observation and supports nothing here.

## Does the bar decide the physics engine

No. The bar names Spartan's car, and that car takes from PhysX only what Jolt and Rapier also provide: rigid bodies, joints, shape casts, and a callback before each step. A fully simulated car requires of a physics engine:

1. A constant step, and a place to run one vehicle's substeps inside it.
2. A shape cast per wheel against whatever the ground is, returning the surface's kind.
3. Joints that hold under many iterations, if the suspension is bodies and joints (option E only).
4. f64, or an origin that can follow each simulated vehicle.
5. The same result for the same binary and inputs.
6. A Rust binding of whatever is used.

Items 1, 2 and 5 hold for all three engines the ladder's map names; Rapier's shape cast is its query pipeline's `cast_shape` (`src/pipeline/query_pipeline.rs`). Item 4 holds for Jolt and Rapier in f64 and for PhysX by origin shifts. Item 6 is Rapier's by construction and, for the vehicle modules of Jolt and PhysX, unmet today. What the bar does decide is narrower: an engine's vehicle module is not a reason to choose it, since every module stops short of the bar's tyre and suspension and the rest is written here either way. The engine stays with Gameplay core's charting.

## Cost of delay

M5 took the far field: Far proxies, the 2 km View distance, the Load radius and the streamer's lead. M6 is about to take global illumination, with the dynamic TLAS's place in it.

- **Every option** adds beside both. A vehicle is Moving Instances in the dynamic TLAS, within a cap that exists; nothing in ADRs 0017, 0019, 0020 or 0026 is rebuilt.
- **A and B** ask nothing more.
- **C, D and E** add the wheel-spin rule beside ADR 0022's blend, and a per-Tick Budget line that #17 already holds as fog.
- **E** may tighten ADR 0014's 1 km clause for physics; the ADR's other clauses stand.
- **F** adds a C++ dependency and a thread outside the fixed schedule, which ADR 0022 would have to admit.
- **Any option with physically stepped traffic beyond the Load radius** asks the streamer for ground away from the camera. That would change what M4 and M5 built, and the lane-following tier avoids it.
- **Speed** asks only for a traversal at a car's top speed, a bench run with no new code.

No option rebuilds a decision M5 took or M6 is about to take. Delay costs one thing: the per-vehicle figure stays unknown until Gameplay core is charted.

## Options

- **A. Ray-cast vehicle.** Rapier's controller or one like it. Below the bar; the form traffic would take. Open: Bullet's notice, if anything is adapted.
- **B. An engine's vehicle module.** Jolt's or PhysX's as shipped: gears, clutch, differentials, sway. Open: whether a friction-curve tyre and a strut read as "fully simulated" to the owner; a binding for the module; and it ties the car to the engine.
- **C. An engine's driveline with a tyre written here.** PhysX's custom tyre component. Open: PhysX's f32 and licence class; Jolt cannot take it.
- **D. Own model above rigid bodies.** Chassis as a body, wheels as casts, tyre, driveline, strut suspension with curves, and aerodynamics in Rust in the fixed schedule, substepped per vehicle. Open: its size (Spartan's is thousands of lines), the lowest rate that holds, and its cost per vehicle.
- **E. D with a suspension of bodies and joints.** Spartan's full form. Open: 64 position iterations per car, a 200 Hz scene, and f32's limits.
- **F. A simulation library.** Chrono at run time. Open: 4 to 5 times the per-Tick allowance for one vehicle by the one published figure, and no binding. Beside any other option it can serve as the reference a model is tested against.

A to F are not exclusive: A is the traffic tier of any of B to E.

## Prototype questions

Each is a cost or a limit no source publishes.

1. What one vehicle costs a Tick, for option D at 3 and 4 substeps with 4 tyre substeps and for option A, at 1, 10 and 100 vehicles, on the M4 Pro and the RTX 2070 SUPER's machine: `town`, on its avenue.
2. The lowest rate at which a Magic Formula tyre with slip relaxation holds at a standstill, over a kerb and at top speed: `town`.
3. Whether a suspension of bodies and joints keeps ADR 0014's 0.1 mm at 1 km from an f32 frame's origin at 192 to 256 Hz: `world`, driven 1 km from the origin.
4. Late Chunks, Late Far proxies, Late mips and the slowest frame with the traversal at 100 m/s and at road height: `world`, with `praxis traverse`; it needs no vehicle.
5. A wheel past 60 m/s under the Tick blend, spin blended as an angle: `orrery`.
6. A car body's BLAS rebuilt or refitted once per impact: `crowd`, whose forms already time both.

## Not verified

1. Grand Theft Auto V: no developers' account of its vehicle simulation was found, and the session's web searches ran out before Rockstar's own pages could be searched further.
2. The talks on Just Cause 4 and Uncharted 4 were read as abstracts; neither was watched.
3. The three tyre papers were read as abstracts on their publisher's pages. Pacejka's book, Fiala's 1954 paper and any text on suspension geometry (Milliken, Gillespie) were not opened; the brush model is cited through Chrono's manual.
4. Chrono's 0.6 names no processor and no step size, and its paper measures a traffic framework, not a game.
5. Per-vehicle costs for Jolt, PhysX and Spartan: none found. Spartan's car was read, not run, and its commits after `cb258a1` were not opened.
6. PhysX's guide was read at 5.8.0 and its source at 5.13.0. Unreal's tyre model and substeps, Unity's substep counts and how Unreal's City Sample steps its traffic are not in the pages read.
7. That `physx-rs` is archived is its repository's metadata. No Rust binding of Chrono was searched for beyond the register's finding.
8. Torque3D's aerodynamics and wheel query were not read. The cells of VDrift's and Torque3D's rows without a citation rest on file and field names.
9. A car's top speed is not sourced: 100 m/s stands for twice the measured path.
