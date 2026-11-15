//! A Far proxy's atlas (ADR 0026): one rectangle a planar face of its first
//! Primitive, packed with gutters, and each LOD's UVs in it.

use glam::{UVec2, Vec2, Vec3};

use crate::cooked::MAX_TEXTURE_SIDE;
use crate::scenes::Geometry;

const GUTTER_TEXELS: u32 = 4;
const GRID_TEXELS: u32 = 8;
const LIKE_NORMAL_COSINE: f32 = std::f32::consts::FRAC_1_SQRT_2;

pub(super) fn settings() -> String {
    format!(
        "charted a texel a pixel, a rectangle a planar face with {GUTTER_TEXELS} texels of gutter on a grid of {GRID_TEXELS}, a coarser LOD's faces in the nearest chart whose normal is within a cosine of {LIKE_NORMAL_COSINE} of theirs"
    )
}

fn padded_texels(size_texels: UVec2) -> UVec2 {
    (size_texels + 2 * GUTTER_TEXELS).map(|texels| texels.next_multiple_of(GRID_TEXELS))
}

struct Chart {
    normal: Vec3,
    corner: Vec3,
    // `across` × `down` is `normal`.
    across: Vec3,
    down: Vec3,
    size_texels: UVec2,
    position_texels: UVec2,
}

impl Chart {
    fn fit(normal: Vec3, points: &[Vec3], edges: &[[Vec3; 2]], texel_metres: f32) -> Self {
        let extent_metres = |axis: Vec3| {
            let along = points.iter().map(|point| point.dot(axis));
            let lowest = along.clone().fold(f32::INFINITY, f32::min);
            (lowest, along.fold(f32::NEG_INFINITY, f32::max) - lowest)
        };
        let rectangles = edges.iter().map(|[from, to]| {
            let across = (*to - *from).normalize();
            let down = normal.cross(across);
            let ((left, width), (top, height)) = (extent_metres(across), extent_metres(down));
            if height > width {
                let lowest = Vec2::new(top, -(left + width));
                (down, -across, lowest, Vec2::new(height, width))
            } else {
                (across, down, Vec2::new(left, top), Vec2::new(width, height))
            }
        });
        let (across, down, lowest_metres, size_metres) = rectangles
            .min_by(|(.., first), (.., second)| {
                first.element_product().total_cmp(&second.element_product())
            })
            .expect("a face has an edge");
        let in_plane = across * lowest_metres.x + down * lowest_metres.y;
        Self {
            normal,
            corner: in_plane + normal * points[0].dot(normal),
            across,
            down,
            size_texels: (size_metres / texel_metres)
                .ceil()
                .as_uvec2()
                .max(UVec2::ONE),
            position_texels: UVec2::ZERO,
        }
    }

    /// `point` in `unit_metres` across and down from the corner, and in
    /// metres out of the plane.
    fn in_chart(&self, point: Vec3, unit_metres: f32) -> Vec3 {
        let from_corner = point - self.corner;
        Vec3::new(
            from_corner.dot(self.across) / unit_metres,
            from_corner.dot(self.down) / unit_metres,
            from_corner.dot(self.normal),
        )
    }

    /// `point`'s texel in the atlas, held inside the chart, and its distance
    /// from the chart's rectangle in metres.
    fn nearest_texel(&self, point: Vec3, texel_metres: f32) -> (Vec2, f32) {
        let from_corner = self.in_chart(point, texel_metres);
        let held = (from_corner.truncate()).clamp(Vec2::ZERO, self.size_texels.as_vec2());
        let outside_metres = (from_corner.truncate() - held) * texel_metres;
        (
            self.position_texels.as_vec2() + held,
            outside_metres.extend(from_corner.z).length(),
        )
    }
}

// Triangles that share a vertex must lie in one plane: a vertex per position
// and face normal.
fn faces(geometry: &Geometry) -> Vec<Vec<usize>> {
    fn root(parents: &mut [usize], mut vertex: usize) -> usize {
        while parents[vertex] != vertex {
            parents[vertex] = parents[parents[vertex]];
            vertex = parents[vertex];
        }
        vertex
    }
    let mut parents: Vec<usize> = (0..geometry.positions.len()).collect();
    for triangle in geometry.indices.as_chunks::<3>().0 {
        let [first, second, third] = triangle.map(|index| root(&mut parents, index as usize));
        parents[second] = first;
        parents[third] = first;
    }
    let mut face_of_root = vec![usize::MAX; parents.len()];
    let mut faces: Vec<Vec<usize>> = Vec::new();
    for vertex in 0..parents.len() {
        let root = root(&mut parents, vertex);
        if face_of_root[root] == usize::MAX {
            face_of_root[root] = faces.len();
            faces.push(Vec::new());
        }
        faces[face_of_root[root]].push(vertex);
    }
    faces
}

fn pack(charts: &mut [Chart]) -> anyhow::Result<u32> {
    let mut order: Vec<usize> = (0..charts.len()).collect();
    order.sort_by_key(|&chart| std::cmp::Reverse(charts[chart].size_texels.y));
    let mut side_texels = GRID_TEXELS;
    while side_texels <= MAX_TEXTURE_SIDE {
        let (mut column, mut row, mut shelf_rows, mut fits) = (0, 0, 0, true);
        let mut positions = Vec::with_capacity(charts.len());
        for &chart in &order {
            let padded = padded_texels(charts[chart].size_texels);
            if column + padded.x > side_texels {
                (column, row, shelf_rows) = (0, row + shelf_rows, 0);
            }
            shelf_rows = shelf_rows.max(padded.y);
            fits &= padded.x <= side_texels && row + shelf_rows <= side_texels;
            positions.push(UVec2::new(column, row) + GUTTER_TEXELS);
            column += padded.x;
        }
        if fits {
            for (&chart, position) in order.iter().zip(positions) {
                charts[chart].position_texels = position;
            }
            return Ok(side_texels);
        }
        side_texels *= 2;
    }
    anyhow::bail!("the Far proxy's charts do not fit an atlas of {MAX_TEXTURE_SIDE} texels a side")
}

pub(super) struct Atlas {
    charts: Vec<Chart>,
    side_texels: u32,
    texel_metres: f32,
}

impl Atlas {
    /// Gives `shell`'s vertices their UVs; none for a shell without a
    /// triangle.
    pub(super) fn chart_shell(
        shell: &mut Geometry,
        texel_metres: f32,
    ) -> anyhow::Result<Option<Self>> {
        if shell.indices.is_empty() {
            return Ok(None);
        }
        let faces = faces(shell);
        let position = |vertex: usize| Vec3::from(shell.positions[vertex]);
        let mut face_of = vec![0; shell.positions.len()];
        for (face, vertices) in faces.iter().enumerate() {
            for &vertex in vertices {
                face_of[vertex] = face;
            }
        }
        let mut edges = vec![Vec::new(); faces.len()];
        for triangle in shell.indices.as_chunks::<3>().0 {
            let [first, second, third] = triangle.map(|index| position(index as usize));
            edges[face_of[triangle[0] as usize]].extend([
                [first, second],
                [second, third],
                [third, first],
            ]);
        }
        let mut charts: Vec<Chart> = (faces.iter().zip(&edges))
            .map(|(vertices, edges)| {
                let points: Vec<Vec3> = vertices.iter().map(|&vertex| position(vertex)).collect();
                let normal = Vec3::from(shell.normals[vertices[0]]);
                Chart::fit(normal, &points, edges, texel_metres)
            })
            .collect();
        let side_texels = pack(&mut charts)?;
        let atlas = Self {
            charts,
            side_texels,
            texel_metres,
        };
        let uvs = face_of.iter().enumerate();
        let uvs = uvs.map(|(vertex, &face)| atlas.uv(&atlas.charts[face], position(vertex)));
        shell.uvs = uvs.collect();
        Ok(Some(atlas))
    }

    /// Gives `lod`, a coarser LOD of the charted shell, its UVs.
    pub(super) fn map_lod(&self, lod: &mut Geometry) {
        let position = |vertex: usize| Vec3::from(lod.positions[vertex]);
        let mut uvs = vec![[0.0; 2]; lod.positions.len()];
        for face in faces(lod) {
            let normal = Vec3::from(lod.normals[face[0]]);
            let nearest = |like_cosine: f32| {
                let like = (self.charts.iter())
                    .filter(|chart| chart.normal.dot(normal) >= like_cosine)
                    .map(|chart| {
                        let distances = (face.iter()).map(|&vertex| {
                            chart.nearest_texel(position(vertex), self.texel_metres).1
                        });
                        (distances.sum::<f32>(), chart)
                    });
                like.min_by(|(first, _), (second, _)| first.total_cmp(second))
            };
            let (_, chart) = nearest(LIKE_NORMAL_COSINE)
                .or_else(|| nearest(f32::NEG_INFINITY))
                .expect("an atlas has a chart");
            for vertex in face {
                uvs[vertex] = self.uv(chart, position(vertex));
            }
        }
        lod.uvs = uvs;
    }

    fn uv(&self, chart: &Chart, point: Vec3) -> [f32; 2] {
        let (texel, _) = chart.nearest_texel(point, self.texel_metres);
        (texel / self.side_texels as f32).to_array()
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use super::*;
    use crate::cook::far_proxy::faceted;

    const TEXEL_METRES: f32 = 0.1473;
    const TURN_RADIANS: f32 = 0.3;

    const QUADS: [([f32; 2], Vec3, f32); 6] = [
        ([4.0, 3.0], Vec3::new(0.0, 0.0, 0.0), TURN_RADIANS),
        ([2.0, 9.0], Vec3::new(10.0, 0.0, -5.0), TURN_RADIANS),
        ([30.0, 0.3], Vec3::new(20.0, 3.0, 5.0), TURN_RADIANS),
        ([1.0, 1.0], Vec3::new(30.0, 0.0, 9.0), TURN_RADIANS),
        ([7.0, 2.5], Vec3::new(40.0, 0.0, 2.0), TURN_RADIANS),
        (
            [5.0, 2.0],
            Vec3::new(40.0, 0.0, 5.0),
            TURN_RADIANS + FRAC_PI_2,
        ),
    ];

    fn upright_quads(sizes_feet_and_turns: &[([f32; 2], Vec3, f32)]) -> Geometry {
        let (mut positions, mut indices) = (Vec::new(), Vec::new());
        for ([width, height], foot, turn_radians) in sizes_feet_and_turns {
            let first = positions.len() as u32;
            for (across, up) in [(-0.5, 0.0), (0.5, 0.0), (0.5, 1.0), (-0.5, 1.0)] {
                let corner = Vec3::new(across * width, up * height, 0.0);
                let turned = glam::Quat::from_rotation_y(*turn_radians) * corner;
                positions.push((*foot + turned).to_array());
            }
            indices.extend([0, 1, 2, 0, 2, 3].map(|corner| first + corner));
        }
        faceted(&positions, &indices)
    }

    fn chart_holding(atlas: &Atlas, uv: [f32; 2]) -> Option<usize> {
        let texel = Vec2::from(uv) * atlas.side_texels as f32;
        atlas.charts.iter().position(|chart| {
            let within = texel - chart.position_texels.as_vec2();
            within.cmpge(Vec2::splat(-1e-3)).all()
                && within.cmple(chart.size_texels.as_vec2() + 1e-3).all()
        })
    }

    #[test]
    fn each_face_has_a_chart_of_its_own_on_the_grid_with_its_gutter() {
        let mut shell = upright_quads(&QUADS);
        let atlas = Atlas::chart_shell(&mut shell, TEXEL_METRES)
            .unwrap()
            .unwrap();

        assert_eq!(atlas.side_texels, 256);
        let mut cells = vec![false; (256 / GRID_TEXELS).pow(2) as usize];
        for chart in &atlas.charts {
            let first_cell = (chart.position_texels - GUTTER_TEXELS) / GRID_TEXELS;
            assert_eq!(chart.position_texels % GRID_TEXELS, UVec2::splat(4));
            let past = chart.position_texels + chart.size_texels + 4;
            let end_cell = past.map(|texels| texels.div_ceil(GRID_TEXELS));
            assert!(end_cell.cmple(UVec2::splat(256 / GRID_TEXELS)).all());
            for row in first_cell.y..end_cell.y {
                for column in first_cell.x..end_cell.x {
                    let cell = &mut cells[(row * (256 / GRID_TEXELS) + column) as usize];
                    assert!(!*cell, "two charts share a cell of the grid");
                    *cell = true;
                }
            }
        }
        for (quad, ([width, height], ..)) in QUADS.iter().enumerate() {
            let charts = shell.uvs[quad * 4..][..4].iter();
            let charts: Vec<_> = charts.map(|uv| chart_holding(&atlas, *uv)).collect();
            assert!(charts[0].is_some() && charts.iter().all(|chart| *chart == charts[0]));
            let chart = &atlas.charts[charts[0].unwrap()];
            let texels = [width.max(*height), width.min(*height)]
                .map(|metres| (metres / TEXEL_METRES).ceil() as u32);
            assert_eq!(chart.size_texels.to_array(), texels);
            let diagonal = Vec2::from(shell.uvs[quad * 4 + 2]) - Vec2::from(shell.uvs[quad * 4]);
            let metres = diagonal.abs() * atlas.side_texels as f32 * TEXEL_METRES;
            assert!((metres.max_element() - width.max(*height)).abs() < 1e-3);
            assert!((metres.min_element() - width.min(*height)).abs() < 1e-3);
        }
    }

    #[test]
    fn a_coarser_lods_face_reads_the_nearest_chart_facing_its_way_held_inside_it() {
        let mut shell = upright_quads(&QUADS);
        let atlas = Atlas::chart_shell(&mut shell, TEXEL_METRES)
            .unwrap()
            .unwrap();
        let chart_of = |quad: usize| chart_holding(&atlas, shell.uvs[quad * 4]);
        let mut lod_quads = QUADS.map(|(size, foot, turn)| (size, foot, turn + 0.2));
        lod_quads[0].1.y -= 0.3;
        lod_quads[4].0 = [9.0, 2.5];
        let on_the_turned_chart = ([1.0, 1.0], QUADS[5].1, TURN_RADIANS);
        let facing_away = ([1.0, 1.0], QUADS[3].1, TURN_RADIANS + PI);
        let mut lod =
            upright_quads(&[&lod_quads[..], &[on_the_turned_chart, facing_away]].concat());

        atlas.map_lod(&mut lod);

        let charts: Vec<_> = (lod.uvs.as_chunks::<4>().0.iter())
            .map(|quad| {
                let chart = chart_holding(&atlas, quad[0]);
                assert!(
                    chart.is_some() && quad.iter().all(|uv| chart_holding(&atlas, *uv) == chart)
                );
                chart
            })
            .collect();
        let expected: Vec<_> = (0..6).chain([4, 3]).map(chart_of).collect();
        assert_eq!(charts, expected);
    }

    #[test]
    fn a_face_longer_than_the_largest_atlas_fails_and_a_triangle_without_area_has_no_face() {
        let mut too_long = upright_quads(&[([1300.0, 1.0], Vec3::ZERO, 0.0)]);
        assert!(Atlas::chart_shell(&mut too_long, TEXEL_METRES).is_err());

        let collinear = [[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let mut no_face = faceted(&collinear, &[0, 1, 2]);
        assert!(matches!(
            Atlas::chart_shell(&mut no_face, TEXEL_METRES),
            Ok(None)
        ));
    }
}
