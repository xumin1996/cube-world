use avian3d::parry::na::coordinates::X;
use bevy::input::mouse::MouseMotion;
use bevy::render::mesh::VertexAttributeValues;
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};
use bevy_obj::ObjPlugin;
use core::f32;
use rand::Rng;
use util::Triangle;

pub mod util;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, ObjPlugin))
        .add_systems(Startup, startup)
        .add_systems(Update, handle_mouse_motion)
        .run();
}

pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 平面
    let plane = Plane3d::default().mesh().size(1f32, 1f32).subdivisions(1);
    commands.spawn((
        Mesh3d(meshes.add(plane)),
        MeshMaterial3d(materials.add(Color::WHITE)),
    ));

    // 正方体
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    let mut rng = rand::thread_rng();
    let grass_obj: Handle<Mesh> = asset_server.load::<Mesh>("models/grass.obj");
    util::Triangle::from_mesh(&plane.build())
        .patch(10)
        .into_iter()
        .for_each(|item| {
            let center = item
                .points
                .iter()
                .map(|vsi| Vec3::new(vsi[0], vsi[1], vsi[2]))
                .reduce(|a, b| a + b)
                .map(|ti| ti / 3f32);

            if let Some(center_point) = center {
                let rotate = rng.gen_range(0.0..f32::consts::TAU);
                if let Option::Some(mesh) = meshes.get_mut(&grass_obj) {
                    let tt = Triangle::from_mesh(mesh);
                    println!("tt {:?}", tt);
                    commands.spawn((
                        Mesh3d(meshes.add(tt.build())),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_xyz(center_point.x, center_point.y, center_point.z)
                            .with_scale(Vec3::new(0.01, 0.01, 0.01))
                            .with_rotation(
                                Quat::from_rotation_y(rotate)
                                    * Quat::from_rotation_x(f32::consts::FRAC_PI_2),
                            ),
                    ));
                    panic!("okokok");
                } else {
                    panic!("no");
                }
                // commands.spawn((
                //     Mesh3d(grass_obj.clone()),
                //     MeshMaterial3d(materials.add(
                //         StandardMaterial {
                //             base_color: Color::WHITE,
                //             unlit: true,
                //            ..default()
                //         }
                //     )),
                //     Transform::from_xyz(center_point.x, center_point.y, center_point.z)
                //     .with_scale(Vec3::new(0.01, 0.01, 0.01))
                //     .with_rotation(Quat::from_rotation_y(rotate)*Quat::from_rotation_x(f32::consts::FRAC_PI_2) ),
                // ));
            }
        });

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(1.5, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn handle_mouse_motion(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_transform: Query<&mut Transform, With<Camera>>,
) {
    let displacement = mouse_motion_events
        .read()
        .fold(0., |acc, mouse_motion| acc + mouse_motion.delta.x);

    // 旋转
    camera_transform
        .single_mut()
        .rotate_around(Vec3::ZERO, Quat::from_rotation_y(-displacement / 700.));
}
