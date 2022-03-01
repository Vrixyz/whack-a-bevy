use bevy::{math::Vec4Swizzles, prelude::*};

pub(crate) fn cursor_to_world(
    // need to get window dimensions
    wnds: &Res<Windows>,
    // query to get camera transform
    q_camera: &Transform,
) -> Result<Vec3, ()> {
    // get the primary window
    let wnd = wnds.get_primary().unwrap();

    // check if the cursor is in the primary window
    if let Some(pos) = wnd.cursor_position() {
        // get the size of the window
        let size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // the default orthographic projection is in pixels from the center;
        // just undo the translation
        let p = pos - size / 2.0;

        // assuming there is exactly one main camera entity, so this is OK
        let camera_transform = q_camera;

        // apply the camera transform
        let pos_wld = camera_transform.compute_matrix() * p.extend(0.0).extend(1.0);
        return Ok(pos_wld.xyz());
    }
    Err(())
}
