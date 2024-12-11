use avian3d::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use smooth_bevy_cameras::LookTransformPlugin;

pub mod player;
pub mod region;
pub mod util;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "A Cool Title".into(),
                    resolution: (300., 300.).into(),
                    resizable: true,
                    decorations: true,
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
            LookTransformPlugin,
            TnuaControllerPlugin::default(),
            TnuaAvian3dPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, (player::setup, region::startup))
        .add_systems(
            Update,
            (
                player::handle_keyboard_controls,
                player::handle_mouse_motion,
                player::handle_camera,
                player::handle_light,
                region::region_update,
            ),
        )
        .run();
}
