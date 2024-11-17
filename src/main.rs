use avian3d::{parry::shape, prelude::*};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::mouse::MouseMotion;
use bevy::math::VectorSpace;
use bevy::prelude::*;
use bevy::render::{
    mesh::{Indices, VertexAttributeValues},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use noise::{NoiseFn, Perlin};
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, LookTransformPlugin, Smoother};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            LookTransformPlugin,
            TnuaControllerPlugin::default(),
            TnuaAvian3dPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, (apply_controls, handle_mouse_motion))
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct CamereLookAt {
    look_at: Vec3,
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 地形
    // let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cube_material = materials.add(Color::WHITE);
    let mut cube_positions: Vec<Transform> = Vec::new();
    let mut collider_cube_positions: Vec<Transform> = Vec::new();
    let region_number: i32 = 16;

    let perlin = Perlin::new(1);
    for x in 1..1000 {
        for z in 1..1000 {
            let height = perlin.get([x as f64 / 10.0, z as f64 / 10.0]);
            cube_positions.push(Transform::from_xyz(
                x as f32,
                height as f32 * 2.0f32,
                z as f32,
            ));
            if (x < region_number * 16 && z < region_number * 16) {
                collider_cube_positions.push(Transform::from_xyz(
                    x as f32,
                    height as f32 * 2.0f32,
                    z as f32,
                ));
            }
        }
    }

    // 合并输出mesh
    commands.spawn(PbrBundle {
        mesh: meshes.add(create_cube_mesh(cube_positions)),
        material: cube_material.clone(),
        ..default()
    });

    // 地形
    let collider_cube_mesh = create_cube_mesh(collider_cube_positions);
    commands.spawn((
        RigidBody::Static,
        Collider::trimesh_from_mesh(&collider_cube_mesh).unwrap(),
    ));

    // 地板
    // commands.spawn((RigidBody::Static, Collider::cuboid(2000.0, 1.0, 2000.0)));

    // 角色
    commands.spawn((
        TnuaControllerBundle::default(),
        RigidBody::Dynamic,
        Collider::cuboid(0.2, 0.2, 0.2),
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.8, 0.8, 0.8)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_xyz(5.0, 15.0, 5.0),
            ..default()
        },
        Player,
    ));

    // 点光源
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 10.0, 4.0),
        ..default()
    });

    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.5, 4.5, 9.0)
    //         .looking_at(Vec3::new(5.0, 0.0, 5.0), Vec3::Y),
    //     ..default()
    // });

    let eye = Vec3::new(-2.5, 4.5, 9.0);
    let look_at = Vec3::new(5.0, 0.0, 5.0);
    commands
        .spawn(LookTransformBundle {
            transform: LookTransform::new(eye, look_at, Vec3::Y),
            smoother: Smoother::new(0.9),
        })
        .insert(Camera3dBundle::default());

    commands.insert_resource(CamereLookAt {
        look_at: Vec3::new(0.0, 0.0, 0.0) - Vec3::new(10.0, 6.0, 10.0),
    });
}

fn map_rigid_body(mut commands: Commands, mut maps: Query<&mut TnuaController>) {}

fn handle_mouse_motion(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut camera_look_at: ResMut<CamereLookAt>,
) {
    let displacement = mouse_motion_events
        .read()
        .fold(0., |acc, mouse_motion| acc + mouse_motion.delta.x);

    // 旋转
    let mut camera_transform = Transform::from_translation(camera_look_at.look_at);
    camera_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(-displacement / 500.));
    camera_look_at.look_at = Vec3::new(
        camera_transform.translation.x,
        camera_transform.translation.y,
        camera_transform.translation.z,
    );
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_look_at: Res<CamereLookAt>,
    mut query: Query<&mut TnuaController>,
    mut player_position_query: Query<&mut Transform, With<Player>>,
    mut lookTransformQuery: Query<&mut LookTransform>,
) {
    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };

    let look_direction = camera_look_at.look_at.normalize();
    let rotation_quaternion = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
    let look_direction_rotation = rotation_quaternion.mul_vec3(look_direction);

    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += look_direction;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= look_direction;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= look_direction_rotation;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += look_direction_rotation;
    }
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 15.0,
        float_height: 1.5,
        ..Default::default()
    });

    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 1.5,
            ..Default::default()
        });
    }

    // 更新摄像机位置
    let Ok(mut lt) = lookTransformQuery.get_single_mut() else {
        return;
    };
    let player_position: &Transform = player_position_query.get_single().unwrap();
    lt.eye = player_position.translation - camera_look_at.look_at;
    lt.target = player_position.translation;
}

fn create_cube_mesh(cube_positions: Vec<Transform>) -> Mesh {
    let mut attribute_position: Vec<[f32; 3]> = Vec::new();
    let mut attribute_uv_0: Vec<[f32; 2]> = Vec::new();
    let mut attribute_normal: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for cube_position in cube_positions.iter() {
        let pre_attribute_position_num = attribute_position.len();
        let mut item_position = vec![
            // top (facing towards +y)
            [-0.5, 0.5, -0.5], // vertex with index 0
            [0.5, 0.5, -0.5],  // vertex with index 1
            [0.5, 0.5, 0.5],   // etc. until 23
            [-0.5, 0.5, 0.5],
            // bottom   (-y)
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            // right    (+x)
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5], // This vertex is at the same position as vertex with index 2, but they'll have different UV and normal
            [0.5, 0.5, -0.5],
            // left     (-x)
            [-0.5, -0.5, -0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [-0.5, 0.5, -0.5],
            // back     (+z)
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, -0.5, 0.5],
            // forward  (-z)
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [0.5, 0.5, -0.5],
            [0.5, -0.5, -0.5],
        ];
        for mut position_item in item_position.iter_mut() {
            position_item[0] += cube_position.translation.x;
            position_item[1] += cube_position.translation.y;
            position_item[2] += cube_position.translation.z;
        }

        attribute_position.extend(item_position);
        attribute_uv_0.extend(vec![
            // Assigning the UV coords for the top side.
            [0.0, 0.2],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 0.2],
            // Assigning the UV coords for the bottom side.
            [0.0, 0.45],
            [0.0, 0.25],
            [1.0, 0.25],
            [1.0, 0.45],
            // Assigning the UV coords for the right side.
            [1.0, 0.45],
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            // Assigning the UV coords for the left side.
            [1.0, 0.45],
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            // Assigning the UV coords for the back side.
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            [1.0, 0.45],
            // Assigning the UV coords for the forward side.
            [0.0, 0.45],
            [0.0, 0.2],
            [1.0, 0.2],
            [1.0, 0.45],
        ]);
        attribute_normal.extend(vec![
            // Normals for the top side (towards +y)
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            // Normals for the bottom side (towards -y)
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, -1.0, 0.0],
            // Normals for the right side (towards +x)
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            // Normals for the left side (towards -x)
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            // Normals for the back side (towards +z)
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
            // Normals for the forward side (towards -z)
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
            [0.0, 0.0, -1.0],
        ]);

        let indices_temp: Vec<u32> = vec![
            0, 3, 1, 1, 3, 2, // triangles making up the top (+y) facing side.
            4, 5, 7, 5, 6, 7, // bottom (-y)
            8, 11, 9, 9, 11, 10, // right (+x)
            12, 13, 15, 13, 14, 15, // left (-x)
            16, 19, 17, 17, 19, 18, // back (+z)
            20, 21, 23, 21, 22, 23, // forward (-z)
        ];
        let indices_maped: Vec<u32> = indices_temp
            .iter()
            .map(|item| item + pre_attribute_position_num as u32)
            .collect();
        indices.extend(indices_maped);
    }

    // Keep the mesh data accessible in future frames to be able to mutate it in toggle_texture.
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    // Each array is an [x, y, z] coordinate in local space.
    // The camera coordinate space is right-handed x-right, y-up, z-back. This means "forward" is -Z.
    // Meshes always rotate around their local [0, 0, 0] when a rotation is applied to their Transform.
    // By centering our mesh around the origin, rotating the mesh preserves its center of mass.
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, attribute_position)
    // Set-up UV coordinates to point to the upper (V < 0.5), "dirt+grass" part of the texture.
    // Take a look at the custom image (assets/textures/array_texture.png)
    // so the UV coords will make more sense
    // Note: (0.0, 0.0) = Top-Left in UV mapping, (1.0, 1.0) = Bottom-Right in UV mapping
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, attribute_uv_0)
    // For meshes with flat shading, normals are orthogonal (pointing out) from the direction of
    // the surface.
    // Normals are required for correct lighting calculations.
    // Each array represents a normalized vector, which length should be equal to 1.0.
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, attribute_normal)
    // Create the triangles out of the 24 vertices we created.
    // To construct a square, we need 2 triangles, therefore 12 triangles in total.
    // To construct a triangle, we need the indices of its 3 defined vertices, adding them one
    // by one, in a counter-clockwise order (relative to the position of the viewer, the order
    // should appear counter-clockwise from the front of the triangle, in this case from outside the cube).
    // Read more about how to correctly build a mesh manually in the Bevy documentation of a Mesh,
    // further examples and the implementation of the built-in shapes.
    .with_inserted_indices(Indices::U32(indices))
}
