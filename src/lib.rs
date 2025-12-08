#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

use bevy::{
    camera::{
        ImageRenderTarget, RenderTarget,
        visibility::{Layer, RenderLayers},
    },
    prelude::*,
    window::PrimaryWindow,
};
use std::marker::PhantomData;
pub use wgpu_types;
use wgpu_types::{Extent3d, TextureFormat};

/// - C: Marker component for the camera that will render the output
pub struct OffscreenRenderingPlugin<C, W = PrimaryWindow>
where
    C: Component,
    W: Component,
{
    layer: Layer,
    _pd: PhantomData<(C, W)>,
}

impl<C, W> Default for OffscreenRenderingPlugin<C, W>
where
    C: Component,
    W: Component,
{
    fn default() -> Self {
        Self {
            layer: Default::default(),
            _pd: Default::default(),
        }
    }
}

impl<C, W> OffscreenRenderingPlugin<C, W>
where
    C: Component,
    W: Component,
{
    pub fn new(layer: Layer) -> Self {
        Self {
            layer,
            _pd: Default::default(),
        }
    }

    pub fn with_layer(mut self, layer: Layer) -> Self {
        self.layer = layer;
        self
    }

    fn sync_offscreen_cameras(
        mut commands: Commands,
        main_camera: Single<(&Camera, &Transform), (With<C>, Without<OffscreenCamera<C, W>>)>,
        window: Single<&Window, With<W>>,
        mut q_offscreen: Query<&mut Camera, With<OffscreenCamera<C, W>>>,
        mut images: ResMut<Assets<Image>>,
    ) {
        let (rendering_camera, transform) = *main_camera;

        let render_size = get_viewport_size(Some(rendering_camera), &window);

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
            commands.trigger(OffscreenResizeEvent::<C, W>::new(render_size, *transform));
        }
    }
}

impl<C, W> Plugin for OffscreenRenderingPlugin<C, W>
where
    C: Component,
    W: Component,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(OffscreenRenderingSettings::<C, W>::new(self.layer))
            .add_observer(SpawnNewOffscreenCamera::<C, W>::handle)
            .add_systems(PreUpdate, Self::sync_offscreen_cameras);
    }
}

#[derive(Component)]
#[require(Camera)]
pub struct OffscreenCamera<C, W = PrimaryWindow> {
    _pd: PhantomData<(C, W)>,
}

impl<C, W> Default for OffscreenCamera<C, W>
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

#[derive(Resource, Clone)]
struct OffscreenRenderingSettings<C, W>
where
    C: Component,
    W: Component,
{
    layer: Layer,
    _pd: PhantomData<(C, W)>,
}

impl<C, W> OffscreenRenderingSettings<C, W>
where
    C: Component,
    W: Component,
{
    fn new(layer: Layer) -> Self {
        Self {
            layer,
            _pd: Default::default(),
        }
    }
}

#[derive(Event)]
pub struct OffscreenResizeEvent<C, W = PrimaryWindow>
where
    C: Component,
    W: Component,
{
    pub new_size: UVec2,
    pub transform: Transform,
    _pd: PhantomData<(C, W)>,
}

impl<C, W> OffscreenResizeEvent<C, W>
where
    C: Component,
    W: Component,
{
    fn new(new_size: UVec2, transform: Transform) -> Self {
        Self {
            new_size,
            transform,
            _pd: Default::default(),
        }
    }
}

#[derive(EntityEvent)]
pub struct SpawnNewOffscreenCamera<C, W = PrimaryWindow>
where
    C: Component,
    W: Component,
{
    /// The camera that will render the offscreen data
    entity: Entity,

    camera_type: CameraType,

    texture_format: TextureFormat,

    _pd: PhantomData<(C, W)>,
}

impl<C, W> SpawnNewOffscreenCamera<C, W>
where
    C: Component,
    W: Component,
{
    pub fn new(entity: Entity, camera_type: CameraType, texture_format: TextureFormat) -> Self {
        Self {
            entity,
            camera_type,
            texture_format,
            _pd: Default::default(),
        }
    }

    fn handle(
        event: On<Self>,
        mut commands: Commands,
        q_cameras: Query<&Camera>,
        mut images: ResMut<Assets<Image>>,
        window: Single<&Window, With<W>>,
        settings: Res<OffscreenRenderingSettings<C, W>>,
    ) {
        let rendering_camera = q_cameras.get(event.event_target()).ok();

        new_offscreen_camera::<C, W>(
            &mut commands,
            &mut images,
            event.event_target(),
            rendering_camera,
            &window,
            settings.layer,
            event.camera_type.clone(),
            event.texture_format,
        );
    }
}

#[derive(Clone)]
pub enum CameraType {
    Camera2d(Camera2d),
    Camera3d(Camera3d),
}

pub fn new_offscreen_camera<C, W>(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    rendering_camera_entity: Entity,
    rendering_camera: Option<&Camera>,
    rendering_window: &Window,
    layer: Layer,
    camera_type: CameraType,
    texture_format: TextureFormat,
) -> (Entity, Handle<Image>)
where
    C: Component,
    W: Component,
{
    let image_size = get_viewport_size(rendering_camera, rendering_window);

    let image = Image::new_target_texture(image_size.x, image_size.y, texture_format);

    let image_handle = images.add(image);

    let offscreen_entity_id = match camera_type {
        CameraType::Camera2d(cam_2d) => commands
            .spawn((
                cam_2d,
                offscreen_camera_bundle::<C, W>(image_handle.clone(), layer),
            ))
            .id(),
        CameraType::Camera3d(cam_3d) => commands
            .spawn((
                cam_3d,
                offscreen_camera_bundle::<C, W>(image_handle.clone(), layer),
            ))
            .id(),
    };

    commands
        .entity(rendering_camera_entity)
        .add_child(offscreen_entity_id);

    (offscreen_entity_id, image_handle)
}

pub fn get_viewport_size(camera: Option<&Camera>, window: &Window) -> UVec2 {
    camera
        .and_then(|c| c.viewport.as_ref().map(|vp| vp.physical_size))
        .unwrap_or(window.physical_size())
}

fn offscreen_camera_bundle<C, W>(image_handle: Handle<Image>, layer: Layer) -> impl Bundle
where
    C: Component,
    W: Component,
{
    (
        OffscreenCamera::<C, W>::default(),
        RenderLayers::layer(layer),
        Camera {
            target: RenderTarget::Image(ImageRenderTarget::from(image_handle)),
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
    )
}
