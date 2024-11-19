use avian3d::{parry::shape, prelude::*};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
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
                player::apply_controls,
                player::handle_mouse_motion,
                region::region_update,
            ),
        )
        .run();
}
