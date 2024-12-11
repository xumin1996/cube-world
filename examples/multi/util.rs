use bevy::math::Vec3;
use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;

#[derive(Debug)]
pub struct Triangle {
    points: Vec<Vec3>,
    normal: Vec<Vec3>,
    uv: Vec<Vec3>, // 最后一位是0
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
        // points
        let points_option: Option<&VertexAttributeValues> = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
        let points;
        if let Option::Some(VertexAttributeValues::Float32x3(vs)) = points_option {
            points = vs.clone().iter().map(
                |v| Vec3::new(v[0], v[1], v[2])
            )
            .collect()
        } else {
            points = vec![];
        }

        // normals
        let normals_option: Option<&VertexAttributeValues> = mesh.attribute(Mesh::ATTRIBUTE_NORMAL);
        let normals;
        if let Option::Some(VertexAttributeValues::Float32x3(vs)) = normals_option {
            normals = vs.clone().iter().map(
                |v| Vec3::new(v[0], v[1], v[2])
            )
            .collect()
        } else {
            normals = vec![];
        }

        // normal
        let uv0s_option: Option<&VertexAttributeValues> = mesh.attribute(Mesh::ATTRIBUTE_UV_0);
        let uv0s;
        if let Option::Some(VertexAttributeValues::Float32x3(vs)) = uv0s_option {
            uv0s = vs.clone().iter().map(
                |v| Vec3::new(v[0], v[1], v[2])
            )
            .collect()
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
        if subdivisions == 0 || subdivisions == 1 {
            return vec![Triangle::new(
                self.points.clone(),
                self.normal.clone(),
                self.uv.clone(),
            )];
        }

        let points_tiny: Vec<Vec3> = vec![
            Vec3::new(0.0, 0.0, 0.0),
            (*self.points.get(1).unwrap() - *self.points.get(0).unwrap()) / subdivisions as f32,
            (*self.points.get(2).unwrap() - *self.points.get(0).unwrap()) / subdivisions as f32,
        ];
        let normal_tiny: Vec<Vec3> = vec![
            Vec3::new(0.0, 0.0, 0.0),
            (*self.normal.get(1).unwrap() - *self.normal.get(0).unwrap()) / subdivisions as f32,
            (*self.normal.get(2).unwrap() - *self.normal.get(0).unwrap()) / subdivisions as f32,
        ];
        let uv_tiny: Vec<Vec3> = vec![
            Vec3::new(0.0, 0.0, 0.0),
            (*self.uv.get(1).unwrap() - *self.uv.get(0).unwrap()) / subdivisions as f32,
            (*self.uv.get(2).unwrap() - *self.uv.get(0).unwrap()) / subdivisions as f32,
        ];
        let points_v1: Vec3 = *self.points.get(1).unwrap() - *self.points.get(0).unwrap();
        let points_v2: Vec3 = *self.points.get(2).unwrap() - *self.points.get(0).unwrap();
        let normal_v1: Vec3 = *self.normal.get(1).unwrap() - *self.normal.get(0).unwrap();
        let normal_v2: Vec3 = *self.normal.get(2).unwrap() - *self.normal.get(0).unwrap();
        let uv_v1: Vec3 = *self.uv.get(1).unwrap() - *self.uv.get(0).unwrap();
        let uv_v2: Vec3 = *self.uv.get(2).unwrap() - *self.uv.get(0).unwrap();
        let mut traiangles: Vec<Triangle> = Vec::new();
        for row in 0..subdivisions {
            let point_row = points_v1 * row as f32;
            let normal_row = normal_v1 * row as f32;
            let uv_row = uv_v1 * row as f32;

            for column in 0..subdivisions - row {
                let point_column = points_v2 * column as f32;
                let normal_column = normal_v2 * column as f32;
                let uv_column = uv_v2 * column as f32;

                let point_start = point_row + point_column;
                let normal_start = normal_row + normal_column;
                let uv_start = uv_row + uv_column;

                // points
                let new_points: Vec<Vec3> = points_tiny.iter().map(|p| *p + point_start).collect();

                // normal
                let new_normal: Vec<Vec3> = normal_tiny.iter().map(|n| *n + normal_start).collect();

                // uv
                let new_uv: Vec<Vec3> = uv_tiny.iter().map(|uv| *uv + uv_start).collect();
                traiangles.push(Triangle::new(new_points, new_normal, new_uv));
            }
        }
        return traiangles;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        Triangle::new(
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
            ],
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
            vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(0.0, 0.0, 0.0),
            ],
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
        println!("tris {:?}", tris);
    }
}
