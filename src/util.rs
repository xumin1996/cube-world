use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::{
    mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology,
};
use bevy::tasks::futures_lite::io::split;
use std::ops::{Add, Mul};

#[derive(Debug)]
pub struct Triangle {
    pub points: Vec<Vec3>,
    pub normal: Vec<Vec3>,
    pub uv: Vec<Vec3>, // 最后一位是0
}

impl Triangle {
    pub fn new(points: Vec<Vec3>, normal: Vec<Vec3>, uv: Vec<Vec3>) -> Triangle {
        Triangle {
            points: points,
            normal: normal,
            uv: uv,
        }
    }

    pub fn from_mesh(mesh: &Mesh) -> Triangle {
        let indices = mesh.indices().unwrap();

        // points
        let points_option: Option<&VertexAttributeValues> =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION);
        let points;
        if let Option::Some(VertexAttributeValues::Float32x3(vs)) = points_option {
            points = indices
                .iter()
                .map(|it| {
                    if let Option::Some(v) = vs.get(it) {
                        let r = Vec3::new(v[0], v[1], v[2]);
                        r
                    } else {
                        Vec3::ZERO
                    }
                })
                .collect();
        } else {
            points = vec![];
        }

        // normals
        let normals_option: Option<&VertexAttributeValues> = mesh.attribute(Mesh::ATTRIBUTE_NORMAL);
        let normals;
        if let Option::Some(VertexAttributeValues::Float32x3(vs)) = normals_option {
            normals = indices
                .iter()
                .map(|it| {
                    if let Option::Some(v) = vs.get(it) {
                        Vec3::new(v[0], v[1], v[2])
                    } else {
                        Vec3::ZERO
                    }
                })
                .collect();
        } else {
            normals = vec![];
        }

        // uv
        let uv0s_option: Option<&VertexAttributeValues> = mesh.attribute(Mesh::ATTRIBUTE_UV_0);
        let uv0s;
        if let Option::Some(VertexAttributeValues::Float32x2(vs)) = uv0s_option {
            uv0s = indices
                .iter()
                .map(|it: usize| {
                    if let Option::Some(v) = vs.get(it) {
                        Vec3::new(v[0], v[1], 0.0)
                    } else {
                        Vec3::ZERO
                    }
                })
                .collect();
        } else {
            uv0s = vec![];
        }

        Triangle {
            points: points,
            normal: normals,
            uv: uv0s,
        }
    }

    pub fn patch(&self, subdivisions: u32) -> Vec<Triangle> {
        if subdivisions == 0 {
            return vec![Triangle::new(
                self.points.clone(),
                self.normal.clone(),
                self.uv.clone(),
            )];
        }

        // 分割三角形
        let mut points: Vec<Vec<Vec3>> = self
            .points
            .chunks(3)
            .flat_map(|vec3s| {
                let r: Vec<Vec<Vec3>> =
                    Triangle::divide(&vec![vec3s[0], vec3s[1], vec3s[2]], subdivisions);
                return r;
            })
            .collect();

        let mut normals: Vec<Vec<Vec3>> = self
            .normal
            .chunks(3)
            .flat_map(|vec3s| {
                let r: Vec<Vec<Vec3>> =
                    Triangle::divide(&vec![vec3s[0], vec3s[1], vec3s[2]], subdivisions);
                return r;
            })
            .collect();

        let mut uv0s: Vec<Vec<Vec3>> = self
            .uv
            .chunks(3)
            .flat_map(|vec3s| {
                let r: Vec<Vec<Vec3>> =
                    Triangle::divide(&vec![vec3s[0], vec3s[1], vec3s[2]], subdivisions);
                return r;
            })
            .collect();

        // 组合顶点生成mesh
        let mut traiangles: Vec<Triangle> = Vec::new();
        for _ in 0..points.len() {
            let point = points.pop();
            let normal = normals.pop();
            let uv0: Option<Vec<Vec3>> = uv0s.pop();
            traiangles.push(Triangle::new(
                point.unwrap_or(vec![]),
                normal.unwrap_or(vec![]),
                uv0.unwrap_or(vec![]),
            ));
        }

        return traiangles;
    }

    fn divide(tri: &Vec<Vec3>, num: u32) -> Vec<Vec<Vec3>> {
        let scale = num as f32;
        // point
        let point_0 = *tri.get(0).unwrap_or(&Vec3::ZERO);
        let point_1 = *tri.get(1).unwrap_or(&Vec3::ZERO);
        let point_2 = *tri.get(2).unwrap_or(&Vec3::ZERO);
        // line
        let v1_v0_tiny = (point_1 - point_0) / scale;
        let v2_v0_tiny = (point_2 - point_0) / scale;
        let v1_v2_tiny = (point_1 - point_2) / scale;
        // 三角
        let points_tiny_down: Vec<Vec3> = vec![Vec3::ZERO, v1_v0_tiny, v2_v0_tiny];
        let points_tiny_up: Vec<Vec3> = vec![Vec3::ZERO, v1_v2_tiny, v1_v0_tiny];

        let mut vec3r: Vec<Vec<Vec3>> = Vec::new();
        for row in 0..num {
            let point_row = v1_v0_tiny * row as f32;

            // 上面的三角形
            for column in 1..num - row {
                let point_column = v2_v0_tiny * column as f32;
                let point_start = point_row + point_column + point_0;

                let new_vec3s = points_tiny_up.iter().map(|p| *p + point_start).collect();
                vec3r.push(new_vec3s);
            }

            // 下面的三角形
            for column in 0..num - row {
                let point_column = v2_v0_tiny * column as f32;
                let point_start = point_row + point_column + point_0;

                let new_vec3s = points_tiny_down.iter().map(|p| *p + point_start).collect();
                vec3r.push(new_vec3s);
            }
        }

        return vec3r;
    }

    pub fn build(self) -> Mesh {
        let points_len = self.points.len() as u32;
        let indices: Vec<u32> = (0..points_len).collect();

        let points_array: Vec<[f32; 3]> = self
            .points
            .into_iter()
            .map(|v3| [v3.x, v3.y, v3.z])
            .collect();
        let normal_array: Vec<[f32; 3]> = self
            .normal
            .into_iter()
            .map(|v3| [v3.x, v3.y, v3.z])
            .collect();
        let uv0_array: Vec<[f32; 2]> = self.uv.into_iter().map(|v3| [v3.x, v3.y]).collect();

        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, points_array)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normal_array)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv0_array)
        .with_inserted_indices(Indices::U32(indices));
    }
}

impl Add<Triangle> for Triangle {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        let mut new_points = Vec::new();
        let mut new_normals = Vec::new();
        let mut new_uv0s = Vec::new();

        new_points.extend(rhs.points);
        new_normals.extend(rhs.normal);
        new_uv0s.extend(rhs.uv);

        Triangle {
            points: new_points,
            normal: new_normals,
            uv: new_uv0s,
        }
    }
}

impl Mul<Transform> for Triangle {
    type Output = Triangle;
    #[inline]
    fn mul(self, trans: Transform) -> Triangle {
        let points = self.points.iter().map(
            |it: &Vec3| {
                trans.transform_point(it)
            }
        )
        .collect();

        Triangle {
            points: points,
            normal: self.normal,
            uv: self.uv,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        Triangle::new(
            vec![Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
            vec![
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            vec![
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            ],
        );
    }

    #[test]
    fn test_patch() {
        let tri = Triangle::new(
            vec![Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
            vec![
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            vec![
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
            ],
        );
        let tris = tri.patch(2);
    }
}
