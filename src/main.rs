use avian3d::prelude::*;
use bevy::prelude::*;
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
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, (move_camera_system, apply_controls))
        .run();
}

#[derive(Component)]
struct Player;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 地形
    let perlin = Perlin::new(1);
    for x in 1..15 {
        for z in 1..15 {
            let height = perlin.get([x as f64 / 10.0, z as f64 / 10.0]);
            commands.spawn((
                RigidBody::Static,
                Collider::cuboid(1.0, 1.0, 1.0),
                PbrBundle {
                    mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    material: materials.add(Color::WHITE),
                    transform: Transform::from_xyz(x as f32, height as f32 * 2.0f32, z as f32),
                    ..default()
                },
            ));
        }
    }

    // 角色
    commands
        .spawn((
            TnuaControllerBundle::default(),
            RigidBody::Dynamic,
            Collider::cuboid(0.2, 0.2, 0.2),
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
                material: materials.add(Color::WHITE),
                transform: Transform::from_xyz(5.0, 5.0, 5.0),
                ..default()
            },
        ))
        .insert(Player);

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
}

fn move_camera_system(
    mut cameras: Query<&mut LookTransform>,
    player_transform: Query<&mut Transform, With<Player>>,
) {
    for mut camera in cameras.iter_mut() {
        camera.target = player_transform.single().translation;
    }
}


fn apply_controls(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut TnuaController>) {
    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction -= Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= Vec3::X;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }
    controller.basis(TnuaBuiltinWalk {
        // The `desired_velocity` determines how the character will move.
        desired_velocity: direction.normalize_or_zero() * 10.0,
        // The `float_height` must be greater (even if by little) from the distance between the
        // character's center and the lowest point of its collider.
        float_height: 1.0,
        // `TnuaBuiltinWalk` has many other fields for customizing the movement - but they have
        // sensible defaults. Refer to the `TnuaBuiltinWalk`'s documentation to learn what they do.
        ..Default::default()
    });

    // Feed the jump action every frame as long as the player holds the jump button. If the player
    // stops holding the jump button, simply stop feeding the action.
    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 1.5,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }
}
