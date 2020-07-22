use super::texture::*;
use crate::action::*;

use gl;

use std::ops::{Deref};

///
/// An OpenGL render target
///
pub struct RenderTarget {
    /// The frame buffer for this render target
    frame_buffer: gl::types::GLuint,

    /// The render buffer for this render target
    render_buffer: Option<gl::types::GLuint>,

    /// The texture attached to the framebuffer (if we're tracking it)
    texture: Option<Texture>,

    /// Set to true if this should drop its frame buffer when done
    drop_frame_buffer: bool
}

impl RenderTarget {
    ///
    /// Creates a new OpenGL render target
    ///
    pub fn new(width: u16, height: u16, render_type: RenderTargetType) -> RenderTarget {
        // Create the backing texture

        unsafe {
            // Find the currently bound frame buffer (so we can rebind it)
            let mut old_frame_buffer = 0;
            gl::GetIntegerv(gl::DRAW_FRAMEBUFFER_BINDING, &mut old_frame_buffer);

            // Create the frame buffer
            let mut frame_buffer =0;
            gl::GenFramebuffers(1, &mut frame_buffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, frame_buffer);

            // Generate the texture or the render buffer for the render target
            let texture;
            let render_buffer;

            match render_type {
                RenderTargetType::Standard => {
                    // Use a backing texture for the rendering
                    let mut backing_texture = Texture::new();
                    backing_texture.create_empty(width, height);

                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, *backing_texture, 0);

                    // This type of render target uses a backing texture
                    texture         = Some(backing_texture);
                    render_buffer   = None;
                }

                RenderTargetType::Multisampled => {
                    // Uses a render buffer for the backing layer 
                    let mut backing_renderbuffer = 0;
                    gl::GenRenderbuffers(1, &mut backing_renderbuffer);

                    let mut old_renderbuffer = 0;
                    gl::GetIntegerv(gl::RENDERBUFFER_BINDING, &mut old_renderbuffer);

                    // Define as a MSAA renderbuffer
                    gl::BindRenderbuffer(gl::RENDERBUFFER, backing_renderbuffer);
                    gl::RenderbufferStorageMultisample(gl::RENDERBUFFER, 4, gl::RGBA8, width as gl::types::GLsizei, height as gl::types::GLsizei);
                    gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::RENDERBUFFER, backing_renderbuffer);

                    gl::BindRenderbuffer(gl::RENDERBUFFER, old_renderbuffer as u32);

                    // This type of render target uses a render buffer
                    texture         = None;
                    render_buffer   = Some(backing_renderbuffer);
                }
            };

            // Bind back to the original framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, old_frame_buffer as u32);

            // Create the render target
            RenderTarget {
                frame_buffer:       frame_buffer,
                texture:            texture,
                render_buffer:      render_buffer,
                drop_frame_buffer:  true
            }
        }
    }

    ///
    /// Returns the texture associated with this render target
    ///
    pub fn texture(&self) -> Option<Texture> {
        self.texture.clone()
    }
}

impl Drop for RenderTarget {
    fn drop(&mut self) {
        unsafe {
            if self.drop_frame_buffer {
                gl::DeleteFramebuffers(1, &self.frame_buffer);
            }

            if let Some(render_buffer) = self.render_buffer {
                gl::DeleteRenderbuffers(1, &render_buffer);
            }
        }
    }
}

///
/// Deref returns the frame buffer ID
///
impl Deref for RenderTarget {
    type Target = gl::types::GLuint;

    fn deref(&self) -> &gl::types::GLuint {
        &self.frame_buffer
    }
}
