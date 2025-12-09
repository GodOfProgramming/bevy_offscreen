use bevy::{camera::RenderTarget, prelude::*, window::PrimaryWindow};
use std::marker::PhantomData;
use wgpu_types::Extent3d;

use crate::{OffscreenCamera, OffscreenResizeEvent, get_viewport_size};

/// - C: Marker component for the camera that will render the output
pub struct OffscreenCameraSyncPlugin<C, W = PrimaryWindow>
where
    C: Component,
    W: Component,
{
    _pd: PhantomData<(C, W)>,
}

impl<C, W> Default for OffscreenCameraSyncPlugin<C, W>
where
    C: Component,
    W: Component,
{
    fn default() -> Self {
        Self {
            _pd: Default::default(),
        }
    }
}

impl<C, W> OffscreenCameraSyncPlugin<C, W>
where
    C: Component,
    W: Component,
{
    pub fn new() -> Self {
        Self {
            _pd: Default::default(),
        }
    }

    fn sync_offscreen_cameras(
        mut commands: Commands,
        rendering_camera: Single<&Camera, (With<C>, Without<OffscreenCamera<C, W>>)>,
        window: Single<&Window, With<W>>,
        mut q_offscreen: Query<&mut Camera, With<OffscreenCamera<C, W>>>,
        mut images: ResMut<Assets<Image>>,
    ) {
        let render_size = get_viewport_size(Some(&rendering_camera), &window);

        let mut was_resized = false;

        for mut offscreen in &mut q_offscreen {
            if let RenderTarget::Image(image_target) = &mut offscreen.target
                && let Some(image) = images.get_mut(image_target.handle.id())
                && image.size() != render_size
            {
                was_resized = true;
                image.resize(Extent3d {
                    width: render_size.x,
                    height: render_size.y,
                    depth_or_array_layers: 1,
                });
            }
        }

        if was_resized {
            commands.trigger(OffscreenResizeEvent::<C, W>::new(render_size));
        }
    }
}

impl<C, W> Plugin for OffscreenCameraSyncPlugin<C, W>
where
    C: Component,
    W: Component,
{
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, Self::sync_offscreen_cameras);
    }
}
