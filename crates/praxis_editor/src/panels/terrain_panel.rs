//! Terrain editing panel for the editor.

use crate::EditorPanel;
use praxis_terrain::{
    editing::{BrushFalloff, BrushShape},
    TerrainEditOperation, TerrainEditTool,
};

/// Terrain editing panel providing tools for sculpting and painting terrain.
pub struct TerrainPanel {
    /// Active terrain editing tool.
    pub tool: TerrainEditTool,

    /// Whether the panel is open.
    is_open: bool,

    /// Current brush size.
    brush_size: f32,

    /// Current brush strength.
    brush_strength: f32,

    /// Target height for flatten/set operations.
    target_height: f32,

    /// Selected paint layer index.
    paint_layer_index: usize,

    /// Vegetation density for painting.
    vegetation_density: f32,
}

impl TerrainPanel {
    /// Creates a new terrain panel.
    pub fn new() -> Self {
        Self {
            tool: TerrainEditTool::new(),
            is_open: true,
            brush_size: 5.0,
            brush_strength: 0.5,
            target_height: 0.0,
            paint_layer_index: 0,
            vegetation_density: 2.0,
        }
    }

    /// Updates the terrain tool based on panel settings.
    fn update_tool(&mut self) {
        self.tool.set_radius(self.brush_size);
        self.tool.set_strength(self.brush_strength);
        self.tool.heightmap_brush.target_height = self.target_height;
        self.tool.paint_brush.layer_index = self.paint_layer_index;
        self.tool.vegetation_painter.density = self.vegetation_density;
    }
}

impl Default for TerrainPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorPanel for TerrainPanel {
    fn title(&self) -> &str {
        "Terrain Editor"
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _world: Option<&praxis_ecs::World>,
        _render_context: Option<&mut praxis_graphics::RenderContext>,
    ) {
        ui.heading("Terrain Sculpting");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Operation:");
            ui.selectable_value(
                &mut self.tool.operation,
                TerrainEditOperation::Raise,
                "Raise",
            );
            ui.selectable_value(
                &mut self.tool.operation,
                TerrainEditOperation::Lower,
                "Lower",
            );
            ui.selectable_value(
                &mut self.tool.operation,
                TerrainEditOperation::Smooth,
                "Smooth",
            );
            ui.selectable_value(
                &mut self.tool.operation,
                TerrainEditOperation::Flatten,
                "Flatten",
            );
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Brush Size:");
            ui.add(egui::Slider::new(&mut self.brush_size, 1.0..=50.0).suffix(" m"));
        });

        ui.horizontal(|ui| {
            ui.label("Strength:");
            ui.add(egui::Slider::new(&mut self.brush_strength, 0.0..=1.0));
        });

        if self.tool.operation == TerrainEditOperation::Flatten
            || self.tool.operation == TerrainEditOperation::SetHeight
        {
            ui.horizontal(|ui| {
                ui.label("Target Height:");
                ui.add(egui::Slider::new(&mut self.target_height, 0.0..=100.0).suffix(" m"));
            });
        }

        ui.add_space(10.0);

        ui.label("Brush Shape:");
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.tool.heightmap_brush.shape,
                BrushShape::Circle,
                "Circle",
            );
            ui.selectable_value(
                &mut self.tool.heightmap_brush.shape,
                BrushShape::Square,
                "Square",
            );
        });

        ui.label("Falloff:");
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.tool.heightmap_brush.falloff,
                BrushFalloff::Linear,
                "Linear",
            );
            ui.selectable_value(
                &mut self.tool.heightmap_brush.falloff,
                BrushFalloff::Smooth,
                "Smooth",
            );
            ui.selectable_value(
                &mut self.tool.heightmap_brush.falloff,
                BrushFalloff::Constant,
                "Constant",
            );
        });

        ui.add_space(20.0);
        ui.separator();
        ui.heading("Material Painting");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Layer:");
            ui.selectable_value(&mut self.paint_layer_index, 0, "Layer 0");
            ui.selectable_value(&mut self.paint_layer_index, 1, "Layer 1");
            ui.selectable_value(&mut self.paint_layer_index, 2, "Layer 2");
            ui.selectable_value(&mut self.paint_layer_index, 3, "Layer 3");
        });

        ui.add_space(20.0);
        ui.separator();
        ui.heading("Vegetation");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Density:");
            ui.add(egui::Slider::new(&mut self.vegetation_density, 0.1..=10.0).suffix(" /m²"));
        });

        ui.button("Paint Vegetation").clicked();

        ui.button("Erase Vegetation").clicked();

        ui.button("Generate All Vegetation").clicked();

        ui.add_space(20.0);
        ui.separator();

        if ui
            .button(if self.tool.is_active {
                "✓ Tool Active"
            } else {
                "Activate Tool"
            })
            .clicked()
        {
            if self.tool.is_active {
                self.tool.deactivate();
            } else {
                self.tool.activate();
            }
        }

        self.update_tool();
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open(&mut self, open: bool) {
        self.is_open = open;
    }
}

/// Extension trait for terrain panel integration.
pub trait TerrainPanelExt {
    /// Gets a reference to the terrain panel.
    fn terrain_panel(&self) -> Option<&TerrainPanel>;

    /// Gets a mutable reference to the terrain panel.
    fn terrain_panel_mut(&mut self) -> Option<&mut TerrainPanel>;
}
