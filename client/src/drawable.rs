use piston_window::graphics::math::Matrix2d;
use piston_window::graphics::{ellipse, rectangle};
use piston_window::wgpu_graphics::WgpuGraphics;

use common::Position;
use crate::shapes::{RenderKind, Shape};

/// Rendu d'une forme avec piston — séparé de `Shape` pour que la collision (GJK) ne dépende jamais du rendu.
pub trait Drawable {
    fn draw(&self, center: Position, color: [f32; 4], transform: Matrix2d, graphics: &mut WgpuGraphics<'_>);
}

/// Fourni automatiquement pour toute forme à partir de `half_extents`/`render_kind`.
impl<T: Shape + ?Sized> Drawable for T {
    fn draw(&self, center: Position, color: [f32; 4], transform: Matrix2d, graphics: &mut WgpuGraphics<'_>) {
        let (half_width, half_height) = self.half_extents();
        let bounds = [
            center.x() - half_width,
            center.y() - half_height,
            half_width * 2.0,
            half_height * 2.0,
        ];
        match self.render_kind() {
            RenderKind::Rect => rectangle(color, bounds, transform, graphics),
            RenderKind::Ellipse => ellipse(color, bounds, transform, graphics),
        }
    }
}
