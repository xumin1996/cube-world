use avian3d::{parry::shape, prelude::*};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use smooth_bevy_cameras::{LookTransformPlugin};

pub mod player;
pub mod region;

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
        .add_systems(Startup, (player::setup))
        .add_systems(
            Update,
            (
                player::handle_keyboard_controls,
                player::handle_mouse_motion,
                region::region_update,
            ),
        )
        .run();
}
