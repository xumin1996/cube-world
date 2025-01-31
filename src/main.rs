use avian3d::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use smooth_bevy_cameras::LookTransformPlugin;
use bevy_demo::{customMaterial, cubePlain, region, player};


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build(),
            PhysicsPlugins::default(),
            LookTransformPlugin,
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
            MaterialPlugin::<customMaterial::CustomMaterial>::default(),
        ))
        .add_systems(
            Startup,
            (
                player::setup,
                // cubePlain::setup,
                region::startup,
            ),
        )
        .add_systems(
            Update,
            (
                player::handle_keyboard_controls,
                player::handle_mouse_motion,
                player::handle_camera,
                player::handle_light,
                // cubePlain::handle_keyboard_controls,
                // cubePlain::handle_mouse_motion,
                // cubePlain::handle_camera,
                region::region_update,
            ),
        )
        .insert_resource(Gravity(Vec3::NEG_Y * 9.8))
        .run();
}
