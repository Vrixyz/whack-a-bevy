use bevy::{prelude::*, window::PrimaryWindow};

/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct MyWorldCoords(Vec2);

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn setup(mut commands: Commands) {
    // Make sure to add the marker component when you set up your camera
    commands.spawn((Camera2dBundle::default(), MainCamera));
}

pub fn cursor_to_world(
    // query to get the window (so we can read the current cursor position)
    window: &Window,
    // query to get camera transform
    (camera, camera_transform): (&Camera, &GlobalTransform),
) -> Option<Vec2> {
    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
}
