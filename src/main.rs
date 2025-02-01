use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use smooth_bevy_cameras::LookTransformPlugin;
use bevy_demo::{customMaterial, cubePlain, region, player};
use bevy_rapier3d::prelude::*;


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),
            LookTransformPlugin,
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
        .run();
}
