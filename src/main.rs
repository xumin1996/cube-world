use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::scene::ron::de;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy_demo::{cubePlain, customMaterial, player, region};
use bevy_rapier3d::prelude::*;
use smooth_bevy_cameras::LookTransformPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build(),
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin::default(),  // rapier碰撞调试
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
                player::del_bullet,
                // cubePlain::handle_keyboard_controls,
                // cubePlain::handle_mouse_motion,
                // cubePlain::handle_camera,
                region::region_update,
                grab_mouse,
            ),
        )
        .run();
}

fn grab_mouse(
    mut window: Single<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}