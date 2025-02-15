#![doc = include_str!("../README.md")]
// This crate uses unsafe, but attempts to minimize its usage. All functions
// that utilize unsafe must explicitly enable it.
#![deny(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::hash::{self, BuildHasher, Hash};
use std::ops::{Add, AddAssign, Deref, DerefMut};
use std::sync::Arc;

use ahash::AHasher;
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "cosmic-text")]
pub use cosmic_text;
pub use figures;
use figures::units::UPx;
use figures::{Fraction, FromComponents, IntoComponents, Point, Rect, Size};
use sealed::ShapeSource as _;
use wgpu::util::DeviceExt;

use crate::buffer::Buffer;
use crate::pipeline::{Uniforms, Vertex};
use crate::sealed::ClipRect;

/// Application and Windowing Support.
#[cfg(feature = "app")]
pub mod app;
mod atlas;
mod buffer;
mod pipeline;
mod pod;
/// An easy-to-use batching renderer.
pub mod render;
mod sealed;
/// Types for drawing paths and shapes.
pub mod shapes;
/// Types for text rendering.
#[cfg(feature = "cosmic-text")]
pub mod text;

pub use atlas::{CollectedTexture, TextureCollection};
pub use pipeline::{PreparedGraphic, ShaderScalable};

/// A 2d graphics instance.
///
/// This type contains the GPU state for a single instance of Kludgine. To
/// render graphics correctly, it must know the size and scale of the surface
/// being rendered to. These values are provided in the constructor, but can be
/// updated using [`resize()`](Self::resize).
///
/// To draw using Kludgine, create a [`Frame`] using
/// [`next_frame()`](Self::next_frame). [`wgpu`] has lifetime requirements on
/// the [`wgpu::RenderPass`] which causes each item being rendered to be
/// attached to the lifetime of the render pass. This means that no temporary
/// variables can be used to render.
///
/// Instead, graphics must be prepared before rendering, and stored somewhere
/// during the remainder of the [`RenderingGraphics`]. To prepare graphics to be
/// rendered, call [`Frame::prepare()`] to receive a [`Graphics`] instance that
/// can be used in various Kludgine APIs such as
/// [`Shape::prepare`](shapes::Shape::prepare).
#[derive(Debug)]
pub struct Kludgine {
    default_bindings: wgpu::BindGroup,
    pipeline: wgpu::RenderPipeline,
    _shader: wgpu::ShaderModule,
    binding_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: Buffer<Uniforms>,
    size: Size<UPx>,
    scale: Fraction,
    #[cfg(feature = "cosmic-text")]
    text: text::TextSystem,
}

impl Kludgine {
    /// Returns a new instance of Kludgine with the provided parameters.
    #[must_use]
    #[cfg_attr(not(feature = "cosmic-text"), allow(unused_variables))]
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        initial_size: Size<UPx>,
        scale: f32,
    ) -> Self {
        let scale = Fraction::from(scale);
        let uniforms = Buffer::new(
            &[Uniforms::new(initial_size, scale)],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            device,
        );

        let binding_layout = pipeline::bind_group_layout(device);

        let pipeline_layout = pipeline::layout(device, &binding_layout);

        let empty_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let default_bindings = pipeline::bind_group(
            device,
            &binding_layout,
            &uniforms.wgpu,
            &empty_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            &sampler,
        );

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline = pipeline::new(device, &pipeline_layout, &shader, format);

        Self {
            #[cfg(feature = "cosmic-text")]
            text: text::TextSystem::new(&ProtoGraphics {
                device,
                queue,
                binding_layout: &binding_layout,
                sampler: &sampler,
                uniforms: &uniforms.wgpu,
            }),

            default_bindings,
            pipeline,
            _shader: shader,
            sampler,
            size: initial_size,
            scale,

            uniforms,
            binding_layout,
        }
    }

    /// Updates the size and scale of this Kludgine instance.
    ///
    /// This function updates data stored in the GPU that affects how graphics
    /// are rendered. It should be called before calling `next_frame()` if the
    /// size or scale of the underlying surface has changed.
    pub fn resize(&mut self, new_size: Size<UPx>, new_scale: f32, queue: &wgpu::Queue) {
        let new_scale = Fraction::from(new_scale);
        if self.size != new_size || self.scale != new_scale {
            self.size = new_size;
            self.scale = new_scale;
            self.uniforms
                .update(0, &[Uniforms::new(self.size, self.scale)], queue);
        }

        #[cfg(feature = "cosmic-text")]
        self.text.scale_changed(self.scale);
    }

    /// Begins rendering a new frame.
    pub fn next_frame(&mut self) -> Frame<'_> {
        #[cfg(feature = "cosmic-text")]
        self.text.new_frame();
        Frame {
            kludgine: self,
            commands: None,
        }
    }

    /// Returns the currently configured size to render.
    pub const fn size(&self) -> Size<UPx> {
        self.size
    }

    /// Returns the current scaling factor for the display this instance is
    /// rendering to.
    pub const fn scale(&self) -> Fraction {
        self.scale
    }
}

/// A frame that can be rendered.
///
/// # Panics
///
/// After [`Frame::render()`] has been invoked, this type will panic if dropped
/// before either [`Frame::submit()`] or [`Frame::abort()`] are invoked. This
/// panic is designed to prevent accidentally forgetting to submit a frame to the GPU.q
pub struct Frame<'gfx> {
    kludgine: &'gfx mut Kludgine,
    commands: Option<wgpu::CommandEncoder>,
}

impl Frame<'_> {
    /// Creates a [`Graphics`] context for this frame that can be used to
    /// prepare graphics for rendering:
    ///
    /// - [`Shape::prepare`](shapes::Shape::prepare)
    /// - [`Texture::prepare`]
    /// - [`Texture::prepare_partial`]
    /// - [`CollectedTexture::prepare`]
    /// - [`Drawing::new_frame`](render::Drawing::new_frame)
    ///
    /// The returned graphics provides access to the various types to update
    /// their representation on the GPU so that they can be rendered later.
    pub fn prepare<'gfx>(
        &'gfx mut self,
        device: &'gfx wgpu::Device,
        queue: &'gfx wgpu::Queue,
    ) -> Graphics<'gfx> {
        Graphics::new(self.kludgine, device, queue)
    }

    /// Creates a [`RenderingGraphics`] context for this frame which is used to
    /// render previously prepared graphics:
    ///
    /// - [`PreparedGraphic`]
    /// - [`PreparedText`](text::PreparedText)
    /// - [`Drawing`](render::Drawing)
    #[must_use]
    pub fn render<'gfx, 'pass>(
        &'pass mut self,
        pass: &wgpu::RenderPassDescriptor<'pass, '_>,
        device: &'gfx wgpu::Device,
        queue: &'gfx wgpu::Queue,
    ) -> RenderingGraphics<'gfx, 'pass> {
        if self.commands.is_none() {
            self.commands =
                Some(device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default()));
        }
        RenderingGraphics::new(
            self.commands
                .as_mut()
                .expect("initialized above")
                .begin_render_pass(pass),
            self.kludgine,
            device,
            queue,
        )
    }

    /// Creates a [`RenderingGraphics`] that renders into `texture` for this
    /// frame. The returned context can be used to render previously prepared
    /// graphics:
    ///
    /// - [`PreparedGraphic`]
    /// - [`PreparedText`](text::PreparedText)
    /// - [`Drawing`](render::Drawing)
    pub fn render_into<'gfx, 'pass>(
        &'pass mut self,
        texture: &'pass Texture,
        load_op: wgpu::LoadOp<Color>,
        device: &'gfx wgpu::Device,
        queue: &'gfx wgpu::Queue,
    ) -> RenderingGraphics<'gfx, 'pass> {
        self.render(
            &wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match load_op {
                            wgpu::LoadOp::Clear(color) => wgpu::LoadOp::Clear(color.into()),
                            wgpu::LoadOp::Load => wgpu::LoadOp::Load,
                        },
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            },
            device,
            queue,
        )
    }

    /// Submits all of the commands for this frame to the GPU.
    ///
    /// This function does not block for the operations to finish. The returned
    /// [`wgpu::SubmissionIndex`] can be used to block until completion if
    /// desired.
    #[allow(clippy::must_use_candidate)]
    pub fn submit(mut self, queue: &wgpu::Queue) -> Option<wgpu::SubmissionIndex> {
        let commands = self.commands.take()?;
        Some(queue.submit([commands.finish()]))
    }

    /// Aborts rendering this frame.
    ///
    /// If [`Frame::render()`] has been invoked, this function must be used
    /// instead of dropping the frame. This type implements a panic-on-drop to
    /// prevent forgetting to submit the frame to the GPU, and this function
    /// prevents the panic from happening.
    pub fn abort(mut self) {
        // Clear out the commands, preventing drop from panicking.
        self.commands.take();
    }
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        assert!(
            self.commands.is_none(),
            "Frame dropped without calling finish() or abort()"
        );
    }
}

trait WgpuDeviceAndQueue {
    fn device(&self) -> &wgpu::Device;
    fn queue(&self) -> &wgpu::Queue;
    fn binding_layout(&self) -> &wgpu::BindGroupLayout;
    fn uniforms(&self) -> &wgpu::Buffer;
    fn sampler(&self) -> &wgpu::Sampler;
}

struct ProtoGraphics<'gfx> {
    device: &'gfx wgpu::Device,
    queue: &'gfx wgpu::Queue,
    binding_layout: &'gfx wgpu::BindGroupLayout,
    sampler: &'gfx wgpu::Sampler,
    uniforms: &'gfx wgpu::Buffer,
}

impl WgpuDeviceAndQueue for ProtoGraphics<'_> {
    fn device(&self) -> &wgpu::Device {
        self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        self.queue
    }

    fn binding_layout(&self) -> &wgpu::BindGroupLayout {
        self.binding_layout
    }

    fn uniforms(&self) -> &wgpu::Buffer {
        self.uniforms
    }

    fn sampler(&self) -> &wgpu::Sampler {
        self.sampler
    }
}

impl WgpuDeviceAndQueue for Graphics<'_> {
    fn device(&self) -> &wgpu::Device {
        self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        self.queue
    }

    fn binding_layout(&self) -> &wgpu::BindGroupLayout {
        &self.kludgine.binding_layout
    }

    fn uniforms(&self) -> &wgpu::Buffer {
        &self.kludgine.uniforms.wgpu
    }

    fn sampler(&self) -> &wgpu::Sampler {
        &self.kludgine.sampler
    }
}

#[derive(Debug)]
struct ClipStack {
    current: ClipRect,
    previous_clips: Vec<ClipRect>,
}

impl ClipStack {
    pub fn new(size: Size<UPx>) -> Self {
        Self {
            current: size.into(),
            previous_clips: Vec::new(),
        }
    }

    pub fn push_clip(&mut self, clip: Rect<UPx>) {
        let previous_clip = self.current;
        self.current = previous_clip.clip_to(clip);
        self.previous_clips.push(previous_clip);
    }

    pub fn pop_clip(&mut self) {
        self.current = self.previous_clips.pop().expect("unpaired pop_clip");
    }
}

/// A context used to prepare graphics to render.
///
/// This type is used in these APIs:
///
/// - [`Shape::prepare`](shapes::Shape::prepare)
/// - [`Texture::prepare`]
/// - [`Texture::prepare_partial`]
/// - [`CollectedTexture::prepare`]
/// - [`Drawing::new_frame`](render::Drawing::new_frame)
#[derive(Debug)]
pub struct Graphics<'gfx> {
    kludgine: &'gfx mut Kludgine,
    device: &'gfx wgpu::Device,
    queue: &'gfx wgpu::Queue, // Need this eventually to be able to have dynamic shape collections
    clip: ClipStack,
}

impl<'gfx> Graphics<'gfx> {
    /// Returns a new instance.
    pub fn new(
        kludgine: &'gfx mut Kludgine,
        device: &'gfx wgpu::Device,
        queue: &'gfx wgpu::Queue,
    ) -> Self {
        Self {
            clip: ClipStack::new(kludgine.size),
            kludgine,
            device,
            queue,
        }
    }

    /// Returns a reference to the underlying [`wgpu::Device`].
    #[must_use]
    pub const fn device(&self) -> &'gfx wgpu::Device {
        self.device
    }

    /// Returns a reference to the underlying [`wgpu::Queue`].
    #[must_use]
    pub const fn queue(&self) -> &'gfx wgpu::Queue {
        self.queue
    }

    /// Returns a mutable reference to the [`cosmic_text::FontSystem`] used when
    /// rendering text.
    #[cfg(feature = "cosmic-text")]
    pub fn font_system(&mut self) -> &mut cosmic_text::FontSystem {
        self.kludgine.font_system()
    }

    /// Returns the current clipped size of the context.
    ///
    /// If this context has not been clipped, the value returned will be
    /// equivalent to [`Kludgine::size`].
    #[must_use]
    pub const fn size(&self) -> Size<UPx> {
        self.clip.current.0.size
    }
}

impl AsRef<wgpu::Device> for Graphics<'_> {
    fn as_ref(&self) -> &wgpu::Device {
        self.device()
    }
}

impl AsRef<wgpu::Queue> for Graphics<'_> {
    fn as_ref(&self) -> &wgpu::Queue {
        self.queue()
    }
}

impl Deref for Graphics<'_> {
    type Target = Kludgine;

    fn deref(&self) -> &Self::Target {
        self.kludgine
    }
}

impl DerefMut for Graphics<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.kludgine
    }
}

impl Clipped for Graphics<'_> {
    fn push_clip(&mut self, clip: Rect<UPx>) {
        self.clip.push_clip(clip);
    }

    fn pop_clip(&mut self) {
        self.clip.pop_clip();
    }
}

impl sealed::Clipped for Graphics<'_> {}

/// A graphics context used to render previously prepared graphics.
///
/// This type is used to render these types:
///
/// - [`PreparedGraphic`]
/// - [`PreparedText`](text::PreparedText)
/// - [`Drawing`](render::Drawing)
pub struct RenderingGraphics<'gfx, 'pass> {
    pass: wgpu::RenderPass<'pass>,
    kludgine: &'pass Kludgine,
    device: &'gfx wgpu::Device,
    queue: &'gfx wgpu::Queue,
    clip: ClipStack,
    pipeline_is_active: bool,
}

impl<'gfx, 'pass> RenderingGraphics<'gfx, 'pass> {
    fn new(
        pass: wgpu::RenderPass<'pass>,
        kludgine: &'pass Kludgine,
        device: &'gfx wgpu::Device,
        queue: &'gfx wgpu::Queue,
    ) -> Self {
        Self {
            pass,
            clip: ClipStack::new(kludgine.size),
            kludgine,
            device,
            queue,
            pipeline_is_active: false,
        }
    }

    /// Returns a reference to the underlying [`wgpu::Device`].
    #[must_use]
    pub const fn device(&self) -> &'gfx wgpu::Device {
        self.device
    }

    /// Returns a reference to the underlying [`wgpu::Queue`].
    #[must_use]
    pub const fn queue(&self) -> &'gfx wgpu::Queue {
        self.queue
    }

    fn active_pipeline_if_needed(&mut self) -> bool {
        if self.pipeline_is_active {
            false
        } else {
            self.pipeline_is_active = true;
            self.pass.set_pipeline(&self.kludgine.pipeline);
            true
        }
    }

    /// Returns a [`ClipGuard`] that causes all drawing operations to be offset
    /// and clipped to `clip` until it is dropped.
    ///
    /// This function causes the [`RenderingGraphics`] to act as if the origin
    /// of the context is `clip.origin`, and the size of the context is
    /// `clip.size`. This means that rendering at 0,0 will actually render at
    /// the effective clip rect's origin.
    ///
    /// `clip` is relative to the current clip rect and cannot extend the
    /// current clipping rectangle.
    pub fn clipped_to(&mut self, clip: Rect<UPx>) -> ClipGuard<'_, Self> {
        self.push_clip(clip);
        ClipGuard { clipped: self }
    }

    /// Returns the current size of the graphics area being rendered to.
    ///
    /// If the graphics has been clipped, this returns the current width of the
    /// clipped area.
    #[must_use]
    pub const fn size(&self) -> Size<UPx> {
        self.clip.current.0.size
    }

    /// Returns the current scaling factor of the display being rendered to.
    #[must_use]
    pub const fn scale(&self) -> Fraction {
        self.kludgine.scale()
    }
}

/// A graphics context that has been clipped.
pub trait Clipped: Sized + sealed::Clipped {
    /// Pushes a new clipping state to the clipping stack.
    ///
    /// This function causes this type to act as if the origin of the context is
    /// `clip.origin`, and the size of the context is `clip.size`. This means
    /// that rendering at 0,0 will actually render at the effective clip rect's
    /// origin.
    ///
    /// `clip` is relative to the current clip rect and cannot extend the
    /// current clipping rectangle.
    ///
    /// To restore the clipping rect to the state it was before this function
    /// was called, use [`Clipped::pop_clip()`].
    fn push_clip(&mut self, clip: Rect<UPx>);
    /// Restores the clipping rect to the previous state before the last call to
    /// [`Clipped::push_clip()`].
    ///
    /// # Panics
    ///
    /// This function will panic if it is called more times than
    /// [`Clipped::push_clip()`].
    fn pop_clip(&mut self);

    /// Returns a [`ClipGuard`] that causes all drawing operations to be offset
    /// and clipped to `clip` until it is dropped.
    ///
    /// This function causes this type to act as if the origin of the context is
    /// `clip.origin`, and the size of the context is `clip.size`. This means
    /// that rendering at 0,0 will actually render at the effective clip rect's
    /// origin.
    ///
    /// `clip` is relative to the current clip rect and cannot extend the
    /// current clipping rectangle.
    fn clipped_to(&mut self, clip: Rect<UPx>) -> ClipGuard<'_, Self> {
        self.push_clip(clip);
        ClipGuard { clipped: self }
    }
}

impl Clipped for RenderingGraphics<'_, '_> {
    fn pop_clip(&mut self) {
        self.clip.pop_clip();
        if self.clip.current.size.width > 0 && self.clip.current.size.height > 0 {
            self.pass.set_scissor_rect(
                self.clip.current.origin.x.0,
                self.clip.current.origin.y.0,
                self.clip.current.size.width.0,
                self.clip.current.size.height.0,
            );
        }
    }

    fn push_clip(&mut self, clip: Rect<UPx>) {
        self.clip.push_clip(clip);
        if self.clip.current.size.width > 0 && self.clip.current.size.height > 0 {
            self.pass.set_scissor_rect(
                self.clip.current.origin.x.0,
                self.clip.current.origin.y.0,
                self.clip.current.size.width.0,
                self.clip.current.size.height.0,
            );
        }
    }
}

impl sealed::Clipped for RenderingGraphics<'_, '_> {}

/// A clipped surface.
///
/// When dropped, the clipped type will have its clip rect restored to the
/// previously clipped rect. [`ClipGuard`]s can be nested.
///
/// This type implements [`Deref`]/[`DerefMut`] to provide access to the
/// underyling clipped type.
#[derive(Debug)]
pub struct ClipGuard<'clip, T>
where
    T: Clipped,
{
    clipped: &'clip mut T,
}

impl<T> Drop for ClipGuard<'_, T>
where
    T: Clipped,
{
    fn drop(&mut self) {
        self.clipped.pop_clip();
    }
}

impl<T> Deref for ClipGuard<'_, T>
where
    T: Clipped,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.clipped
    }
}

impl<T> DerefMut for ClipGuard<'_, T>
where
    T: Clipped,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.clipped
    }
}

/// A red, green, blue, and alpha color value stored in 32-bits.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Pod, Zeroable)]
#[repr(C)]
pub struct Color(u32);

impl Color {
    /// Returns a new color with the provided components.
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self((red as u32) << 24 | (green as u32) << 16 | (blue as u32) << 8 | alpha as u32)
    }

    /// Returns a new color by converting each component from its `0.0..=1.0`
    /// range into a `0..=255` range.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // truncation desired
    #[allow(clippy::cast_sign_loss)] // sign loss is truncated
    pub fn new_f32(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self::new(
            (red.max(0.) * 255.).round() as u8,
            (green.max(0.) * 255.).round() as u8,
            (blue.max(0.) * 255.).round() as u8,
            (alpha.max(0.) * 255.).round() as u8,
        )
    }

    /// Returns the red component of this color, range 0-255.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // truncation desired
    pub const fn red(&self) -> u8 {
        (self.0 >> 24) as u8
    }

    /// Returns the red component of this color, range 0.0-1.0.
    #[must_use]
    pub fn red_f32(&self) -> f32 {
        f32::from(self.red()) / 255.
    }

    /// Returns the green component of this color, range 0-255.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // truncation desired
    pub const fn green(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// Returns the green component of this color, range 0.0-1.0.
    #[must_use]
    pub fn green_f32(&self) -> f32 {
        f32::from(self.green()) / 255.
    }

    /// Returns the blue component of this color, range 0-255.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // truncation desired
    pub const fn blue(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    /// Returns the blue component of this color, range 0.0-1.0.
    #[must_use]
    pub fn blue_f32(&self) -> f32 {
        f32::from(self.blue()) / 255.
    }

    /// Returns the alpha component of this color, range 0-255. A value of 255
    /// is completely opaque.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // truncation desired
    pub const fn alpha(&self) -> u8 {
        self.0 as u8
    }

    /// Returns the alpha component of this color, range 0.0-1.0. A value of 1.0
    /// is completely opaque.
    #[must_use]
    pub fn alpha_f32(&self) -> f32 {
        f32::from(self.alpha()) / 255.
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "#{:08x}", self.0)
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        Self {
            r: f64::from(color.red_f32()),
            g: f64::from(color.green_f32()),
            b: f64::from(color.blue_f32()),
            a: f64::from(color.alpha_f32()),
        }
    }
}

#[cfg(feature = "cosmic-text")]
impl From<cosmic_text::Color> for Color {
    fn from(value: cosmic_text::Color) -> Self {
        Self::new(value.r(), value.g(), value.b(), value.a())
    }
}

#[cfg(feature = "cosmic-text")]
impl From<Color> for cosmic_text::Color {
    fn from(value: Color) -> Self {
        Self::rgba(value.red(), value.green(), value.blue(), value.alpha())
    }
}

#[test]
fn color_debug() {
    assert_eq!(format!("{:?}", Color::new(1, 2, 3, 4)), "#01020304");
}

impl Color {
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const ALICEBLUE: Self = Self::new(240, 248, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const ANTIQUEWHITE: Self = Self::new(250, 235, 215, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const AQUA: Self = Self::new(0, 255, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const AQUAMARINE: Self = Self::new(127, 255, 212, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const AZURE: Self = Self::new(240, 255, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BEIGE: Self = Self::new(245, 245, 220, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BISQUE: Self = Self::new(255, 228, 196, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BLANCHEDALMOND: Self = Self::new(255, 235, 205, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BLUE: Self = Self::new(0, 0, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BLUEVIOLET: Self = Self::new(138, 43, 226, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BROWN: Self = Self::new(165, 42, 42, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const BURLYWOOD: Self = Self::new(222, 184, 135, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CADETBLUE: Self = Self::new(95, 158, 160, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CHARTREUSE: Self = Self::new(127, 255, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CHOCOLATE: Self = Self::new(210, 105, 30, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CLEAR_BLACK: Self = Self::new(0, 0, 0, 0);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CLEAR_WHITE: Self = Self::new(255, 255, 255, 0);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CORAL: Self = Self::new(255, 127, 80, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CORNFLOWERBLUE: Self = Self::new(100, 149, 237, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CORNSILK: Self = Self::new(255, 248, 220, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CRIMSON: Self = Self::new(220, 20, 60, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const CYAN: Self = Self::new(0, 255, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKBLUE: Self = Self::new(0, 0, 139, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKCYAN: Self = Self::new(0, 139, 139, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKGOLDENROD: Self = Self::new(184, 134, 11, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKGRAY: Self = Self::new(169, 169, 169, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKGREEN: Self = Self::new(0, 100, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKGREY: Self = Self::new(169, 169, 169, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKKHAKI: Self = Self::new(189, 183, 107, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKMAGENTA: Self = Self::new(139, 0, 139, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKOLIVEGREEN: Self = Self::new(85, 107, 47, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKORANGE: Self = Self::new(255, 140, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKORCHID: Self = Self::new(153, 50, 204, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKRED: Self = Self::new(139, 0, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKSALMON: Self = Self::new(233, 150, 122, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKSEAGREEN: Self = Self::new(143, 188, 143, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKSLATEBLUE: Self = Self::new(72, 61, 139, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKSLATEGRAY: Self = Self::new(47, 79, 79, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKSLATEGREY: Self = Self::new(47, 79, 79, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKTURQUOISE: Self = Self::new(0, 206, 209, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DARKVIOLET: Self = Self::new(148, 0, 211, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DEEPPINK: Self = Self::new(255, 20, 147, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DEEPSKYBLUE: Self = Self::new(0, 191, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DIMGRAY: Self = Self::new(105, 105, 105, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DIMGREY: Self = Self::new(105, 105, 105, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const DODGERBLUE: Self = Self::new(30, 144, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const FIREBRICK: Self = Self::new(178, 34, 34, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const FLORALWHITE: Self = Self::new(255, 250, 240, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const FORESTGREEN: Self = Self::new(34, 139, 34, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const FUCHSIA: Self = Self::new(255, 0, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GAINSBORO: Self = Self::new(220, 220, 220, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GHOSTWHITE: Self = Self::new(248, 248, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GOLD: Self = Self::new(255, 215, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GOLDENROD: Self = Self::new(218, 165, 32, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GRAY: Self = Self::new(128, 128, 128, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GREEN: Self = Self::new(0, 128, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GREENYELLOW: Self = Self::new(173, 255, 47, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const GREY: Self = Self::new(128, 128, 128, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const HONEYDEW: Self = Self::new(240, 255, 240, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const HOTPINK: Self = Self::new(255, 105, 180, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const INDIANRED: Self = Self::new(205, 92, 92, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const INDIGO: Self = Self::new(75, 0, 130, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const IVORY: Self = Self::new(255, 255, 240, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const KHAKI: Self = Self::new(240, 230, 140, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LAVENDER: Self = Self::new(230, 230, 250, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LAVENDERBLUSH: Self = Self::new(255, 240, 245, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LAWNGREEN: Self = Self::new(124, 252, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LEMONCHIFFON: Self = Self::new(255, 250, 205, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTBLUE: Self = Self::new(173, 216, 230, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTCORAL: Self = Self::new(240, 128, 128, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTCYAN: Self = Self::new(224, 255, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTGOLDENRODYELLOW: Self = Self::new(250, 250, 210, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTGRAY: Self = Self::new(211, 211, 211, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTGREEN: Self = Self::new(144, 238, 144, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTGREY: Self = Self::new(211, 211, 211, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTPINK: Self = Self::new(255, 182, 193, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTSALMON: Self = Self::new(255, 160, 122, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTSEAGREEN: Self = Self::new(32, 178, 170, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTSKYBLUE: Self = Self::new(135, 206, 250, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTSLATEGRAY: Self = Self::new(119, 136, 153, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTSLATEGREY: Self = Self::new(119, 136, 153, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTSTEELBLUE: Self = Self::new(176, 196, 222, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIGHTYELLOW: Self = Self::new(255, 255, 224, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIME: Self = Self::new(0, 255, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LIMEGREEN: Self = Self::new(50, 205, 50, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const LINEN: Self = Self::new(250, 240, 230, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MAGENTA: Self = Self::new(255, 0, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MAROON: Self = Self::new(128, 0, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMAQUAMARINE: Self = Self::new(102, 205, 170, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMBLUE: Self = Self::new(0, 0, 205, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMORCHID: Self = Self::new(186, 85, 211, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMPURPLE: Self = Self::new(147, 112, 219, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMSEAGREEN: Self = Self::new(60, 179, 113, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMSLATEBLUE: Self = Self::new(123, 104, 238, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMSPRINGGREEN: Self = Self::new(0, 250, 154, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMTURQUOISE: Self = Self::new(72, 209, 204, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MEDIUMVIOLETRED: Self = Self::new(199, 21, 133, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MIDNIGHTBLUE: Self = Self::new(25, 25, 112, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MINTCREAM: Self = Self::new(245, 255, 250, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MISTYROSE: Self = Self::new(255, 228, 225, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const MOCCASIN: Self = Self::new(255, 228, 181, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const NAVAJOWHITE: Self = Self::new(255, 222, 173, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const NAVY: Self = Self::new(0, 0, 128, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const OLDLACE: Self = Self::new(253, 245, 230, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const OLIVE: Self = Self::new(128, 128, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const OLIVEDRAB: Self = Self::new(107, 142, 35, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const ORANGE: Self = Self::new(255, 165, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const ORANGERED: Self = Self::new(255, 69, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const ORCHID: Self = Self::new(218, 112, 214, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PALEGOLDENROD: Self = Self::new(238, 232, 170, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PALEGREEN: Self = Self::new(152, 251, 152, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PALETURQUOISE: Self = Self::new(175, 238, 238, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PALEVIOLETRED: Self = Self::new(219, 112, 147, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PAPAYAWHIP: Self = Self::new(255, 239, 213, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PEACHPUFF: Self = Self::new(255, 218, 185, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PERU: Self = Self::new(205, 133, 63, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PINK: Self = Self::new(255, 192, 203, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PLUM: Self = Self::new(221, 160, 221, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const POWDERBLUE: Self = Self::new(176, 224, 230, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const PURPLE: Self = Self::new(128, 0, 128, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const REBECCAPURPLE: Self = Self::new(102, 51, 153, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const RED: Self = Self::new(255, 0, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const ROSYBROWN: Self = Self::new(188, 143, 143, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const ROYALBLUE: Self = Self::new(65, 105, 225, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SADDLEBROWN: Self = Self::new(139, 69, 19, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SALMON: Self = Self::new(250, 128, 114, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SANDYBROWN: Self = Self::new(244, 164, 96, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SEAGREEN: Self = Self::new(46, 139, 87, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SEASHELL: Self = Self::new(255, 245, 238, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SIENNA: Self = Self::new(160, 82, 45, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SILVER: Self = Self::new(192, 192, 192, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SKYBLUE: Self = Self::new(135, 206, 235, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SLATEBLUE: Self = Self::new(106, 90, 205, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SLATEGRAY: Self = Self::new(112, 128, 144, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SLATEGREY: Self = Self::new(112, 128, 144, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SNOW: Self = Self::new(255, 250, 250, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const SPRINGGREEN: Self = Self::new(0, 255, 127, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const STEELBLUE: Self = Self::new(70, 130, 180, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const TAN: Self = Self::new(210, 180, 140, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const TEAL: Self = Self::new(0, 128, 128, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const THISTLE: Self = Self::new(216, 191, 216, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const TOMATO: Self = Self::new(255, 99, 71, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const TURQUOISE: Self = Self::new(64, 224, 208, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const VIOLET: Self = Self::new(238, 130, 238, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const WHEAT: Self = Self::new(245, 222, 179, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const WHITESMOKE: Self = Self::new(245, 245, 245, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const YELLOW: Self = Self::new(255, 255, 0, 255);
    /// Equivalent to the [CSS color keywords](https://developer.mozilla.org/en-US/docs/Web/CSS/named-color) of the same name.
    pub const YELLOWGREEN: Self = Self::new(154, 205, 50, 255);
}

/// An image stored on the GPU.
#[derive(Debug)]
pub struct Texture {
    id: sealed::TextureId,
    wgpu: wgpu::Texture,
    view: wgpu::TextureView,
    bind_group: Arc<wgpu::BindGroup>,
}

impl Texture {
    pub(crate) fn new_generic(
        graphics: &impl WgpuDeviceAndQueue,
        size: Size<UPx>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let wgpu = graphics.device().create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: size.into(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        });
        let view = wgpu.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = Arc::new(pipeline::bind_group(
            graphics.device(),
            graphics.binding_layout(),
            graphics.uniforms(),
            &view,
            graphics.sampler(),
        ));
        Self {
            id: sealed::TextureId::new_unique_id(),
            wgpu,
            view,
            bind_group,
        }
    }

    /// Creates a new texture of the given size, format, and usages.
    #[must_use]
    pub fn new(
        graphics: &Graphics<'_>,
        size: Size<UPx>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
    ) -> Self {
        Self::new_generic(graphics, size, format, usage)
    }

    /// Returns a new texture of the given size, format, and usages. The texture
    /// is initialized with `data`. `data` must match `format`.
    #[must_use]
    pub fn new_with_data(
        graphics: &Graphics<'_>,
        size: Size<UPx>,
        format: wgpu::TextureFormat,
        usage: wgpu::TextureUsages,
        data: &[u8],
    ) -> Self {
        let wgpu = graphics.device().create_texture_with_data(
            graphics.queue(),
            &wgpu::TextureDescriptor {
                label: None,
                size: size.into(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            },
            data,
        );
        let view = wgpu.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = Arc::new(pipeline::bind_group(
            graphics.device(),
            graphics.binding_layout(),
            graphics.uniforms(),
            &view,
            graphics.sampler(),
        ));
        Self {
            id: sealed::TextureId::new_unique_id(),
            wgpu,
            view,
            bind_group,
        }
    }

    /// Creates a texture from `image`.
    #[must_use]
    #[cfg(feature = "image")]
    pub fn from_image(image: &image::DynamicImage, graphics: &Graphics<'_>) -> Self {
        // TODO is it better to force rgba8, or is it better to avoid the
        // conversion and allow multiple texture formats?
        let image = image.to_rgba8();
        Self::new_with_data(
            graphics,
            Size::new(image.width(), image.height()),
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::TEXTURE_BINDING,
            image.as_raw(),
        )
    }

    /// Prepares to render this texture with `size`. The returned graphic will
    /// be oriented around `origin`.
    #[must_use]
    pub fn prepare_sized<Unit>(
        &self,
        origin: Origin<Unit>,
        size: Size<Unit>,
        graphics: &Graphics<'_>,
    ) -> PreparedGraphic<Unit>
    where
        Unit: figures::Unit,
        i32: IntoComponents<Unit>,
        Vertex<Unit>: bytemuck::Pod,
    {
        let origin = match origin {
            Origin::TopLeft => Point::default(),
            Origin::Center => Point::default() - (Point::from_vec(size) / 2),
            Origin::Custom(point) => point,
        };
        self.prepare(Rect::new(origin, size), graphics)
    }

    /// Prepares to render this texture at the given location.
    #[must_use]
    pub fn prepare<Unit>(&self, dest: Rect<Unit>, graphics: &Graphics<'_>) -> PreparedGraphic<Unit>
    where
        Unit: figures::Unit,
        Vertex<Unit>: bytemuck::Pod,
    {
        self.prepare_partial(self.size().into(), dest, graphics)
    }

    /// Prepares the `source` area to be rendered at `dest`.
    #[must_use]
    pub fn prepare_partial<Unit>(
        &self,
        source: Rect<UPx>,
        dest: Rect<Unit>,
        graphics: &Graphics<'_>,
    ) -> PreparedGraphic<Unit>
    where
        Unit: figures::Unit,
        Vertex<Unit>: bytemuck::Pod,
    {
        TextureBlit::new(source, dest, Color::WHITE).prepare(Some(self), graphics)
    }

    /// The size of the texture.
    #[must_use]
    pub fn size(&self) -> Size<UPx> {
        Size::new(self.wgpu.width(), self.wgpu.height())
    }

    /// The format of the texture.
    #[must_use]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.wgpu.format()
    }
}

/// The origin of a prepared graphic.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Origin<Unit> {
    /// The graphic should be drawn so that the top-left of the graphic appears
    /// at the rendered location. When rotating the graphic, it will rotate
    /// around the top-left.
    TopLeft,
    /// The grapihc should be drawn so that the center of the graphic appears at
    /// the rendered location. When rotating the graphic, it will rotate around
    /// the center.
    Center,
    /// The graphic should be drawn so that the provided relative location
    /// appears at the rendered location. When rotating the graphic, it will
    /// rotate around this point.
    Custom(Point<Unit>),
}

/// A type that is rendered using a texture.
pub trait TextureSource: sealed::TextureSource {}

impl TextureSource for Texture {}

impl sealed::TextureSource for Texture {
    fn bind_group(&self) -> Arc<wgpu::BindGroup> {
        self.bind_group.clone()
    }

    fn id(&self) -> sealed::TextureId {
        self.id
    }

    fn is_mask(&self) -> bool {
        // TODO this should be a flag on the texture.
        self.wgpu.format() == wgpu::TextureFormat::R8Unorm
    }

    fn default_rect(&self) -> Rect<UPx> {
        self.size().into()
    }
}

#[derive(Default)]
struct DefaultHasher(AHasher);

impl BuildHasher for DefaultHasher {
    type Hasher = AHasher;

    fn build_hasher(&self) -> Self::Hasher {
        self.0.clone()
    }
}

#[derive(Default, Debug)]
struct VertexCollection<T> {
    vertices: Vec<Vertex<T>>,
    vertex_index_by_id: HashMap<VertexId, u16, DefaultHasher>,
}

impl<T> VertexCollection<T> {
    fn get_or_insert(&mut self, vertex: Vertex<T>) -> u16
    where
        T: Copy,
        Vertex<T>: Into<Vertex<i32>>,
    {
        *self
            .vertex_index_by_id
            .entry(VertexId(vertex.into()))
            .or_insert_with(|| {
                let index = self
                    .vertices
                    .len()
                    .try_into()
                    .expect("too many drawn verticies");
                self.vertices.push(vertex);
                index
            })
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
struct VertexId(Vertex<i32>);

impl hash::Hash for VertexId {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        bytemuck::bytes_of(&self.0).hash(state);
    }
}

/// A source of triangle data for a shape.
pub trait ShapeSource<Unit, const TEXTURED: bool>: sealed::ShapeSource<Unit> {}

impl<Unit> ShapeSource<Unit, true> for TextureBlit<Unit> where Unit: Add<Output = Unit> + Ord + Copy {}

impl<Unit> sealed::ShapeSource<Unit> for TextureBlit<Unit>
where
    Unit: Add<Output = Unit> + Ord + Copy,
{
    fn vertices(&self) -> &[Vertex<Unit>] {
        &self.verticies
    }

    fn indices(&self) -> &[u16] {
        &[1, 0, 2, 1, 2, 3]
    }
}

#[derive(Clone, Copy, Debug)]
struct TextureBlit<Unit> {
    verticies: [Vertex<Unit>; 4],
}

#[cfg_attr(not(feature = "cosmic-text"), allow(dead_code))]
impl<Unit> TextureBlit<Unit> {
    pub fn new(source: Rect<UPx>, dest: Rect<Unit>, color: Color) -> Self
    where
        Unit: Add<Output = Unit> + Ord + Copy,
    {
        let (dest_top_left, dest_bottom_right) = dest.extents();
        let (source_top_left, source_bottom_right) = source.extents();
        Self {
            verticies: [
                Vertex {
                    location: dest_top_left,
                    texture: source_top_left,
                    color,
                },
                Vertex {
                    location: Point::new(dest_bottom_right.x, dest_top_left.y),
                    texture: Point::new(source_bottom_right.x, source_top_left.y),
                    color,
                },
                Vertex {
                    location: Point::new(dest_top_left.x, dest_bottom_right.y),
                    texture: Point::new(source_top_left.x, source_bottom_right.y),
                    color,
                },
                Vertex {
                    location: dest_bottom_right,
                    texture: source_bottom_right,
                    color,
                },
            ],
        }
    }

    pub const fn top_left(&self) -> &Vertex<Unit> {
        &self.verticies[0]
    }

    // pub const fn top_right(&self) -> &Vertex<Unit> {
    //     &self.verticies[1]
    // }

    // pub const fn bottom_left(&self) -> &Vertex<Unit> {
    //     &self.verticies[2]
    // }

    pub const fn bottom_right(&self) -> &Vertex<Unit> {
        &self.verticies[3]
    }

    pub fn translate_by(&mut self, offset: Point<Unit>)
    where
        Unit: AddAssign + Copy,
    {
        for vertex in &mut self.verticies {
            vertex.location += offset;
        }
    }
}
