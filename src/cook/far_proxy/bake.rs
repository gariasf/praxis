//! A Far proxy's atlas (ADR 0026): one rectangle a planar face of its first
//! Primitive, packed with gutters, and each LOD's UVs in it.

use glam::{Vec2, Vec3};

use crate::cooked::MAX_TEXTURE_SIDE;
use crate::scenes::Geometry;

const GUTTER_TEXELS: u32 = 4;
const GRID_TEXELS: u32 = 8;
const LIKE_NORMAL_COSINE: f32 = std::f32::consts::FRAC_1_SQRT_2;

pub(super) fn settings() -> String {
    format!(
        "charted a texel a pixel, a rectangle a planar face with {GUTTER_TEXELS} texels of gutter on a grid of {GRID_TEXELS}, a coarser LOD's faces in the nearest chart facing within 45 degrees"
    )
}

fn padded(texels: u32) -> u32 {
    (texels + 2 * GUTTER_TEXELS).next_multiple_of(GRID_TEXELS)
}

struct Chart {
    normal: Vec3,
    /// Its first texel's corner, and the unit vectors along its columns and
    /// its rows: `across` × `down` is `normal`.
    corner: Vec3,
    across: Vec3,
    down: Vec3,
    /// Its columns and rows of texels, and its first texel's in the atlas.
    size: [u32; 2],
    position: [u32; 2],
}

impl Chart {
    /// The least-area rectangle about a face's `points` along one of its
    /// `edges`, its rows no longer than its columns.
    fn fit(normal: Vec3, points: &[Vec3], edges: &[[Vec3; 2]], texel_metres: f32) -> Self {
        let span = |axis: Vec3| {
            let along = points.iter().map(|point| point.dot(axis));
            let lowest = along.clone().fold(f32::INFINITY, f32::min);
            (lowest, along.fold(f32::NEG_INFINITY, f32::max) - lowest)
        };
        let rectangles = edges.iter().map(|[from, to]| {
            let mut across = (*to - *from).normalize();
            if span(normal.cross(across)).1 > span(across).1 {
                across = normal.cross(across);
            }
            let down = normal.cross(across);
            (across, down, span(across), span(down))
        });
        let area =
            |(_, _, (_, width), (_, height)): &(Vec3, Vec3, (f32, f32), (f32, f32))| width * height;
        let (across, down, (left, width), (top, height)) = rectangles
            .min_by(|first, second| area(first).total_cmp(&area(second)))
            .expect("a face has an edge");
        Self {
            normal,
            corner: across * left + down * top + normal * points[0].dot(normal),
            across,
            down,
            size: [width, height].map(|metres| ((metres / texel_metres).ceil() as u32).max(1)),
            position: [0; 2],
        }
    }

    /// `point` in `unit_metres` across and down from its corner, and in
    /// metres out of its plane.
    fn local(&self, point: Vec3, unit_metres: f32) -> Vec3 {
        let from_corner = point - self.corner;
        Vec3::new(
            from_corner.dot(self.across) / unit_metres,
            from_corner.dot(self.down) / unit_metres,
            from_corner.dot(self.normal),
        )
    }

    fn size_texels(&self) -> Vec2 {
        Vec2::new(self.size[0] as f32, self.size[1] as f32)
    }

    fn distance_metres(&self, point: Vec3, texel_metres: f32) -> f32 {
        let local = self.local(point, texel_metres);
        let within = local.truncate().clamp(Vec2::ZERO, self.size_texels());
        ((local.truncate() - within) * texel_metres)
            .extend(local.z)
            .length()
    }
}

/// Each vertex's face and the number of faces: a face is the triangles that
/// share vertices, which lie in one plane in a Geometry with a vertex per
/// position and face normal.
fn faces(geometry: &Geometry) -> (Vec<usize>, usize) {
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
    let (mut face_of_root, mut count) = (vec![usize::MAX; parents.len()], 0);
    let face_of = (0..parents.len()).map(|vertex| {
        let root = root(&mut parents, vertex);
        if face_of_root[root] == usize::MAX {
            face_of_root[root] = count;
            count += 1;
        }
        face_of_root[root]
    });
    (face_of.collect(), count)
}

/// Places each chart on shelves, tallest first, each on the grid with its
/// gutter, in the smallest square atlas of a power-of-two side that holds
/// them all, and gives that side.
fn pack(charts: &mut [Chart]) -> anyhow::Result<u32> {
    let mut order: Vec<usize> = (0..charts.len()).collect();
    order.sort_by_key(|&chart| std::cmp::Reverse(charts[chart].size[1]));
    let mut side = GRID_TEXELS;
    while side <= MAX_TEXTURE_SIDE {
        let (mut column, mut row, mut shelf_rows, mut fits) = (0, 0, 0, true);
        let mut positions = Vec::with_capacity(charts.len());
        for &chart in &order {
            let [columns, rows] = charts[chart].size.map(padded);
            if column + columns > side {
                (column, row, shelf_rows) = (0, row + shelf_rows, 0);
            }
            shelf_rows = shelf_rows.max(rows);
            fits &= columns <= side && row + shelf_rows <= side;
            positions.push([column + GUTTER_TEXELS, row + GUTTER_TEXELS]);
            column += columns;
        }
        if fits {
            for (&chart, position) in order.iter().zip(positions) {
                charts[chart].position = position;
            }
            return Ok(side);
        }
        side *= 2;
    }
    anyhow::bail!("the Far proxy's charts do not fit an atlas of {MAX_TEXTURE_SIDE} texels a side")
}

pub(super) struct Atlas {
    charts: Vec<Chart>,
    side: u32,
    texel_metres: f32,
}

impl Atlas {
    /// Charts `shell`'s planar faces at `texel_metres` a texel and gives its
    /// vertices their UVs; none for a shell without a triangle.
    pub(super) fn chart(shell: &mut Geometry, texel_metres: f32) -> anyhow::Result<Option<Self>> {
        if shell.indices.is_empty() {
            return Ok(None);
        }
        let (face_of, count) = faces(shell);
        let position = |vertex: usize| Vec3::from(shell.positions[vertex]);
        let (mut points, mut edges) = (vec![Vec::new(); count], vec![Vec::new(); count]);
        let mut normals = vec![Vec3::ZERO; count];
        for (vertex, &face) in face_of.iter().enumerate() {
            points[face].push(position(vertex));
            normals[face] = Vec3::from(shell.normals[vertex]);
        }
        for triangle in shell.indices.as_chunks::<3>().0 {
            let [first, second, third] = triangle.map(|index| position(index as usize));
            edges[face_of[triangle[0] as usize]].extend([
                [first, second],
                [second, third],
                [third, first],
            ]);
        }
        let mut charts: Vec<Chart> = (0..count)
            .map(|face| Chart::fit(normals[face], &points[face], &edges[face], texel_metres))
            .collect();
        let side = pack(&mut charts)?;
        let atlas = Self {
            charts,
            side,
            texel_metres,
        };
        let uvs = face_of.iter().enumerate();
        let uvs = uvs.map(|(vertex, &face)| atlas.uv(&atlas.charts[face], position(vertex)));
        shell.uvs = uvs.collect();
        Ok(Some(atlas))
    }

    /// Gives `lod`, a coarser LOD of the charted shell, its UVs: each face's
    /// in the chart nearest it of those facing within 45° of it, or of all.
    pub(super) fn map(&self, lod: &mut Geometry) {
        let (face_of, count) = faces(lod);
        let mut vertices = vec![Vec::new(); count];
        for (vertex, &face) in face_of.iter().enumerate() {
            vertices[face].push(vertex);
        }
        let position = |vertex: usize| Vec3::from(lod.positions[vertex]);
        let mut uvs = vec![[0.0; 2]; face_of.len()];
        for face in vertices {
            let normal = Vec3::from(lod.normals[face[0]]);
            let nearest = |like_cosine: f32| {
                let like = (self.charts.iter())
                    .filter(|chart| chart.normal.dot(normal) >= like_cosine)
                    .map(|chart| {
                        let distances = face.iter().map(|&vertex| {
                            chart.distance_metres(position(vertex), self.texel_metres)
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
        let within = chart.local(point, self.texel_metres).truncate();
        let texel = Vec2::new(chart.position[0] as f32, chart.position[1] as f32)
            + within.clamp(Vec2::ZERO, chart.size_texels());
        (texel / self.side as f32).to_array()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cook::far_proxy::faceted;

    const TEXEL_METRES: f32 = 0.1473;

    /// Upright quads by width, height and where each stands, turned about
    /// the vertical by `turn`, as a faceted shell.
    fn quads(sizes: &[([f32; 2], Vec3)], turn: f32) -> Geometry {
        let (mut positions, mut indices) = (Vec::new(), Vec::new());
        for ([width, height], foot) in sizes {
            let first = positions.len() as u32;
            for (across, up) in [(-0.5, 0.0), (0.5, 0.0), (0.5, 1.0), (-0.5, 1.0)] {
                let corner = Vec3::new(across * width, up * height, 0.0);
                positions.push((*foot + glam::Quat::from_rotation_y(turn) * corner).to_array());
            }
            indices.extend([0, 1, 2, 0, 2, 3].map(|corner| first + corner));
        }
        faceted(&positions, &indices)
    }

    const SIZES: [([f32; 2], Vec3); 5] = [
        ([4.0, 3.0], Vec3::new(0.0, 0.0, 0.0)),
        ([2.0, 9.0], Vec3::new(10.0, 0.0, -5.0)),
        ([30.0, 0.3], Vec3::new(20.0, 3.0, 5.0)),
        ([1.0, 1.0], Vec3::new(30.0, 0.0, 9.0)),
        ([7.0, 2.5], Vec3::new(40.0, 0.0, 2.0)),
    ];

    /// The chart that holds `uv`, gutter excluded.
    fn chart_at(atlas: &Atlas, uv: [f32; 2]) -> Option<usize> {
        let texel = Vec2::from(uv) * atlas.side as f32;
        atlas.charts.iter().position(|chart| {
            let first = Vec2::new(chart.position[0] as f32, chart.position[1] as f32);
            let within = texel - first;
            within.cmpge(Vec2::splat(-1e-3)).all() && within.cmple(chart.size_texels() + 1e-3).all()
        })
    }

    #[test]
    fn each_face_has_a_chart_of_its_own_on_the_grid_with_its_gutter() {
        let mut shell = quads(&SIZES, 0.3);
        let atlas = Atlas::chart(&mut shell, TEXEL_METRES).unwrap().unwrap();

        assert_eq!(atlas.side, 256);
        let rectangles: Vec<_> = (atlas.charts.iter())
            .map(|chart| {
                let first = chart.position.map(|texel| texel - GUTTER_TEXELS);
                let size = chart.size.map(padded);
                assert!(
                    first
                        .iter()
                        .chain(&size)
                        .all(|texels| texels % GRID_TEXELS == 0)
                );
                assert!((0..2).all(|axis| first[axis] + size[axis] <= atlas.side));
                (first, size)
            })
            .collect();
        for (index, (first, size)) in rectangles.iter().enumerate() {
            for (other_first, other_size) in &rectangles[index + 1..] {
                let apart = (0..2).any(|axis| {
                    first[axis] + size[axis] <= other_first[axis]
                        || other_first[axis] + other_size[axis] <= first[axis]
                });
                assert!(apart, "two charts overlap");
            }
        }
        for (quad, ([width, height], _)) in SIZES.iter().enumerate() {
            let charts = shell.uvs[quad * 4..][..4].iter();
            let charts: Vec<_> = charts.map(|uv| chart_at(&atlas, *uv)).collect();
            assert!(charts[0].is_some() && charts.iter().all(|chart| *chart == charts[0]));
            let chart = &atlas.charts[charts[0].unwrap()];
            let texels = [width.max(*height), width.min(*height)]
                .map(|metres| (metres / TEXEL_METRES).ceil() as u32);
            assert_eq!(chart.size, texels);
            let span = Vec2::from(shell.uvs[quad * 4 + 2]) - Vec2::from(shell.uvs[quad * 4]);
            let metres = span.abs() * atlas.side as f32 * TEXEL_METRES;
            assert!((metres.max_element() - width.max(*height)).abs() < 1e-3);
            assert!((metres.min_element() - width.min(*height)).abs() < 1e-3);
        }
    }

    #[test]
    fn a_coarser_lods_face_reads_the_nearest_chart_facing_its_way() {
        let mut shell = quads(&SIZES, 0.3);
        let atlas = Atlas::chart(&mut shell, TEXEL_METRES).unwrap().unwrap();
        let mut moved = SIZES;
        moved[0].1.y -= 0.3;
        moved[4].0 = [9.0, 2.5];
        let mut lod = quads(&moved, 0.5);

        atlas.map(&mut lod);

        for quad in 0..SIZES.len() {
            let own = chart_at(&atlas, shell.uvs[quad * 4]);
            for uv in &lod.uvs[quad * 4..][..4] {
                assert_eq!(chart_at(&atlas, *uv), own, "quad {quad}");
            }
        }
        let lowered = Vec2::from(lod.uvs[2]) - Vec2::from(lod.uvs[0]);
        let own = Vec2::from(shell.uvs[2]) - Vec2::from(shell.uvs[0]);
        assert!(
            lowered.abs().cmplt(own.abs()).any(),
            "the lowered quad was not held inside"
        );
    }
}
