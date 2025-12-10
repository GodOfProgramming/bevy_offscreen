#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub mod sync;

use bevy::{prelude::*, window::PrimaryWindow};
use std::marker::PhantomData;
pub use wgpu_types;

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

#[derive(Event)]
pub struct OffscreenResizeEvent<C, W = PrimaryWindow>
where
    C: Component,
    W: Component,
{
    pub new_size: UVec2,
    _pd: PhantomData<(C, W)>,
}

impl<C, W> OffscreenResizeEvent<C, W>
where
    C: Component,
    W: Component,
{
    fn new(new_size: UVec2) -> Self {
        Self {
            new_size,
            _pd: Default::default(),
        }
    }
}

pub fn get_viewport_size(camera: Option<&Camera>, window: &Window) -> UVec2 {
    camera
        .and_then(|c| c.viewport.as_ref().map(|vp| vp.physical_size))
        .unwrap_or(window.physical_size())
}
