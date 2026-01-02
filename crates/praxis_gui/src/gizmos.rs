//! Transform gizmos for runtime scene editing.

use praxis_ecs::{Entity, Transform, World};
use praxis_math::{Quat, Vec3};

/// Gizmo operation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    /// Translate/move mode.
    Translate,
    /// Rotate mode.
    Rotate,
    /// Scale mode.
    Scale,
}

impl Default for GizmoMode {
    fn default() -> Self {
        Self::Translate
    }
}

/// A single gizmo handle for manipulating transforms.
#[derive(Debug, Clone)]
pub struct Gizmo {
    /// The entity this gizmo is attached to.
    pub entity: Entity,
    /// Current operation mode.
    pub mode: GizmoMode,
    /// Whether the gizmo is currently active (being manipulated).
    pub active: bool,
}

impl Gizmo {
    /// Creates a new gizmo for an entity.
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            mode: GizmoMode::default(),
            active: false,
        }
    }

    /// Sets the gizmo mode.
    pub fn set_mode(&mut self, mode: GizmoMode) {
        self.mode = mode;
    }

    /// Cycles to the next gizmo mode.
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            GizmoMode::Translate => GizmoMode::Rotate,
            GizmoMode::Rotate => GizmoMode::Scale,
            GizmoMode::Scale => GizmoMode::Translate,
        };
    }
}

/// Manager for transform gizmos.
pub struct TransformGizmos {
    /// Whether gizmos are enabled.
    pub enabled: bool,
    /// Active gizmos.
    gizmos: Vec<Gizmo>,
    /// Current gizmo mode.
    pub mode: GizmoMode,
}

impl Default for TransformGizmos {
    fn default() -> Self {
        Self {
            enabled: true,
            gizmos: Vec::new(),
            mode: GizmoMode::Translate,
        }
    }
}

impl TransformGizmos {
    /// Creates a new transform gizmos manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders gizmo UI controls.
    pub fn render(&mut self, ctx: &egui::Context, _world: &mut World) {
        if !self.enabled {
            return;
        }

        egui::Window::new("Transform Gizmos")
            .default_pos(egui::pos2(750.0, 50.0))
            .default_width(250.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Gizmo Controls");
                ui.separator();

                ui.label("Active Mode:");
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.mode == GizmoMode::Translate, "Translate (T)")
                        .clicked()
                    {
                        self.mode = GizmoMode::Translate;
                    }
                    if ui
                        .selectable_label(self.mode == GizmoMode::Rotate, "Rotate (R)")
                        .clicked()
                    {
                        self.mode = GizmoMode::Rotate;
                    }
                    if ui
                        .selectable_label(self.mode == GizmoMode::Scale, "Scale (S)")
                        .clicked()
                    {
                        self.mode = GizmoMode::Scale;
                    }
                });

                ui.separator();

                ui.label(format!("Active Gizmos: {}", self.gizmos.len()));

                if ui.button("Clear All Gizmos").clicked() {
                    self.gizmos.clear();
                }

                ui.separator();
                ui.label("Gizmo List:");

                let mut to_remove = Vec::new();

                for (idx, gizmo) in self.gizmos.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("Entity {:?}", gizmo.entity));

                        if ui.small_button("Remove").clicked() {
                            to_remove.push(idx);
                        }

                        let mode_str = match gizmo.mode {
                            GizmoMode::Translate => "T",
                            GizmoMode::Rotate => "R",
                            GizmoMode::Scale => "S",
                        };
                        ui.label(format!("[{mode_str}]"));
                    });
                }

                for idx in to_remove.iter().rev() {
                    self.gizmos.remove(*idx);
                }
            });
    }

    /// Adds a gizmo for an entity.
    pub fn add_gizmo(&mut self, entity: Entity) {
        if !self.gizmos.iter().any(|g| g.entity == entity) {
            let mut gizmo = Gizmo::new(entity);
            gizmo.mode = self.mode;
            self.gizmos.push(gizmo);
        }
    }

    /// Removes a gizmo for an entity.
    pub fn remove_gizmo(&mut self, entity: Entity) {
        self.gizmos.retain(|g| g.entity != entity);
    }

    /// Gets a mutable reference to a gizmo for an entity.
    pub fn get_gizmo_mut(&mut self, entity: Entity) -> Option<&mut Gizmo> {
        self.gizmos.iter_mut().find(|g| g.entity == entity)
    }

    /// Gets all active gizmos.
    pub fn gizmos(&self) -> &[Gizmo] {
        &self.gizmos
    }

    /// Sets the global gizmo mode and updates all gizmos.
    pub fn set_mode(&mut self, mode: GizmoMode) {
        self.mode = mode;
        for gizmo in &mut self.gizmos {
            gizmo.mode = mode;
        }
    }

    /// Cycles to the next gizmo mode.
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            GizmoMode::Translate => GizmoMode::Rotate,
            GizmoMode::Rotate => GizmoMode::Scale,
            GizmoMode::Scale => GizmoMode::Translate,
        };
        self.set_mode(self.mode);
    }

    /// Applies a translation delta to a gizmo's entity.
    pub fn apply_translation(&self, world: &mut World, entity: Entity, delta: Vec3) {
        let mut query = world.query::<&mut Transform>();
        if let Ok(mut transform) = query.get_mut(world.inner_mut(), entity) {
            transform.translation += delta;
        }
    }

    /// Applies a rotation delta to a gizmo's entity.
    pub fn apply_rotation(&self, world: &mut World, entity: Entity, delta: Quat) {
        let mut query = world.query::<&mut Transform>();
        if let Ok(mut transform) = query.get_mut(world.inner_mut(), entity) {
            transform.rotation = delta * transform.rotation;
        }
    }

    /// Applies a scale delta to a gizmo's entity.
    pub fn apply_scale(&self, world: &mut World, entity: Entity, delta: Vec3) {
        let mut query = world.query::<&mut Transform>();
        if let Ok(mut transform) = query.get_mut(world.inner_mut(), entity) {
            transform.scale *= delta;
        }
    }

    /// Toggles gizmo system enabled state.
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Sets the enabled state.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
