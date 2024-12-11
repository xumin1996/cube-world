use bevy::math::Vec3;

struct Triangle {
    points: Vec<Vec3>,
    normal: Vec<Vec3>,
    uv: Vec<Vec3>, // 最后一位是0
}

impl Triangle {
    pub const fn from(points: Vec<Vec3>, normal: Vec<Vec3>, uv: Vec<Vec3>) -> Triangle {
        Triangle {
            points: points,
            normal: normal,
            uv: uv,
        }
    }

    pub fn patch(&self, subdivisions: u32) -> Vec<Triangle> {
        if subdivisions == 0 || subdivisions == 1 {
            return vec![Triangle::from(
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
                traiangles.push(Triangle::from(new_points, new_normal, new_uv));
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
        Triangle::from(
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
            vec![[0.0, 1.0], [0.0, 0.0]],
        );
    }
}
