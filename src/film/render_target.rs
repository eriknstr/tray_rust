//! Defines the render target for tray, where our image will be written too
//! during rendering
//! TODO: Reconstruction filters

use std::vec::Vec;
use std::{iter, cmp, f32};
use std::sync::Mutex;

use film::Colorf;
use film::filter::Filter;
use sampler::Region;

const FILTER_TABLE_SIZE: usize = 16;

/// A struct containing results of an image sample where a ray was fired through
/// continuous pixel coordinates [x, y] and color `color` was computed
pub struct ImageSample {
    pub x: f32,
    pub y: f32,
    pub color: Colorf,
}

impl ImageSample {
    pub fn new(x: f32, y: f32, color: Colorf) -> ImageSample {
        ImageSample { x: x, y: y, color: color }
    }
}

/// RenderTarget is a RGBF render target to write our image too while rendering
pub struct RenderTarget {
    width: usize,
    height: usize,
    pixels: Vec<Colorf>,
    pixels_locked: Vec<Mutex<Vec<Colorf>>>,
    lock_size: (i32, i32),
    filter: Box<Filter + Send + Sync>,
    filter_table: Vec<f32>,
    filter_pixel_width: (i32, i32),
}

impl RenderTarget {
    /// Create a render target with `width * height` pixels
    pub fn new(image_dim: (usize, usize), lock_size: (usize, usize), filter: Box<Filter + Send + Sync>) -> RenderTarget {
        if image_dim.0 % lock_size.0 != 0 || image_dim.1 % lock_size.1 != 0 {
            panic!("Image with dimension {:?} not evenly divided by blocks of {:?}", image_dim, lock_size);
        }
        let width = image_dim.0;
        let height = image_dim.1;
        let filter_pixel_width = (f32::floor(filter.width() / 0.5) as i32, f32::floor(filter.height() / 0.5) as i32);
        let mut filter_table: Vec<f32> = iter::repeat(0.0).take(FILTER_TABLE_SIZE * FILTER_TABLE_SIZE)
            .collect();
        for y in 0..FILTER_TABLE_SIZE {
            let fy = (y as f32 + 0.5) * filter.height() / FILTER_TABLE_SIZE as f32;
            for x in 0..FILTER_TABLE_SIZE {
                let fx = (x as f32 + 0.5) * filter.width() / FILTER_TABLE_SIZE as f32;
                filter_table[y * FILTER_TABLE_SIZE + x] = filter.weight(fx, fy);
            }
        }

        let x_blocks = width / lock_size.0;
        let y_blocks = height / lock_size.1;
        let mut pixels_locked = Vec::with_capacity(x_blocks * y_blocks);
        for _ in 0..x_blocks * y_blocks {
            pixels_locked.push(Mutex::new(iter::repeat(Colorf::broadcast(0.0)).take(lock_size.0 * lock_size.1).collect()));
        }

        RenderTarget { width: width, height: height,
            pixels: iter::repeat(Colorf::broadcast(0.0)).take(width * height).collect(),
            pixels_locked: pixels_locked,
            lock_size: (lock_size.0 as i32, lock_size.1 as i32),
            filter: filter,
            filter_table: filter_table,
            filter_pixel_width: filter_pixel_width,
        }
    }
    /// Write all the image samples to the render target
    pub fn write(&self, samples: &Vec<ImageSample>, region: &Region) {
        // Determine which blocks we touch with our set of samples
        let x_range = (cmp::max(region.start.0 as i32 - self.filter_pixel_width.0, 0),
                       cmp::min(region.end.0 as i32 + self.filter_pixel_width.0, self.width as i32 - 1));
        let y_range = (cmp::max(region.start.1 as i32 - self.filter_pixel_width.1, 0),
                       cmp::min(region.end.1 as i32 + self.filter_pixel_width.1, self.height as i32 - 1));

        if x_range.1 - x_range.0 < 0 || y_range.1 - y_range.0 < 0 {
                return;
        }
        let block_x_range = (x_range.0 / self.lock_size.0, x_range.1 / self.lock_size.0);
        let block_y_range = (y_range.0 / self.lock_size.1, y_range.1 / self.lock_size.1);
        // Temporary storage for filtered samples so we can compute the filtered results for
        // the block we're writing too without having to get the lock
        let mut filtered_samples: Vec<_> = iter::repeat(Colorf::broadcast(0.0))
            .take((self.lock_size.0 * self.lock_size.1) as usize).collect();

        let blocks_per_row = self.width as i32 / self.lock_size.0;
        for y in block_y_range.0..block_y_range.1 + 1 {
            for x in block_x_range.0..block_x_range.1 + 1 {
                let block_x_start = x * self.lock_size.0;
                let block_y_start = y * self.lock_size.1;

                let x_write_range = (cmp::max(x_range.0, block_x_start),
                                     cmp::min(x_range.1 + 1, block_x_start + self.lock_size.0));
                let y_write_range = (cmp::max(y_range.0, block_y_start),
                                     cmp::min(y_range.1 + 1, block_y_start + self.lock_size.1));

                let block_samples = samples.iter().filter(|s| {
                    s.x >= (x_write_range.0 - self.filter_pixel_width.0) as f32
                        && s.x < (x_write_range.1 + self.filter_pixel_width.0) as f32
                        && s.y >= (y_write_range.0 - self.filter_pixel_width.1) as f32
                        && s.y < (y_write_range.1 + self.filter_pixel_width.1) as f32
                });

                for c in &mut filtered_samples {
                    *c = Colorf::broadcast(0.0);
                }

                // Compute the filtered samples for the block
                for c in block_samples {
                    let img_x = c.x - 0.5;
                    let img_y = c.y - 0.5;
                    for iy in y_write_range.0..y_write_range.1 {
                        let fy = f32::abs(iy as f32 - img_y) * self.filter.inv_height() * FILTER_TABLE_SIZE as f32;
                        let fy_idx = cmp::min(fy as usize, FILTER_TABLE_SIZE - 1);
                        for ix in x_write_range.0..x_write_range.1 {
                            let fx = f32::abs(ix as f32 - img_x) * self.filter.inv_width() * FILTER_TABLE_SIZE as f32;
                            let fx_idx = cmp::min(fx as usize, FILTER_TABLE_SIZE - 1);
                            let weight = self.filter_table[fy_idx * FILTER_TABLE_SIZE + fx_idx];
                            let px = ((iy - block_y_start) * self.lock_size.0 + ix - block_x_start) as usize;
                            // TODO: Can't currently overload the += operator
                            // Coming soon though, see RFC #953 https://github.com/rust-lang/rfcs/pull/953
                            filtered_samples[px].r += weight * c.color.r;
                            filtered_samples[px].g += weight * c.color.g;
                            filtered_samples[px].b += weight * c.color.b;
                            // TODO: Remove this it's just for debugging adaptive sampler
                            filtered_samples[px].a += weight;
                        }
                    }
                }

                // Acquire lock for the block and write the filtered samples
                let block_idx = (y * blocks_per_row + x) as usize;
                let mut pixels = self.pixels_locked[block_idx].lock().unwrap();
                for iy in y_write_range.0..y_write_range.1 {
                    for ix in x_write_range.0..x_write_range.1 {
                        let px = ((iy - block_y_start) * self.lock_size.0 + ix - block_x_start) as usize;
                        let c = &filtered_samples[px];
                        pixels[px].r += c.r;
                        pixels[px].g += c.g;
                        pixels[px].b += c.b;
                        pixels[px].a += c.a;
                    }
                }
            }
        }
    }
    /// Clear the render target to black
    pub fn clear(&mut self) {
        for p in self.pixels.iter_mut() {
            *p = Colorf::broadcast(0.0);
        }
    }
    /// Get the dimensions of the render target
    pub fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
    /// Convert the floating point color buffer to 24bpp sRGB for output to an image
    pub fn get_render(&self) -> Vec<u8> {
        let mut render: Vec<u8> = iter::repeat(0u8).take(self.width * self.height * 3).collect();
        let x_blocks = self.width / self.lock_size.0 as usize;
        let y_blocks = self.height / self.lock_size.1 as usize;
        for by in 0..y_blocks {
            for bx in 0..x_blocks {
                let block_x_start = bx * self.lock_size.0 as usize;
                let block_y_start = by * self.lock_size.1 as usize;
                let block_idx = (by * x_blocks + bx) as usize;
                let pixels = self.pixels_locked[block_idx].lock().unwrap();
                for y in 0..self.lock_size.1 as usize {
                    for x in 0..self.lock_size.0 as usize {
                        let c = &pixels[y * self.lock_size.0 as usize + x];
                        if c.a > 0.0 {
                            let cn = (*c / c.a).clamp().to_srgb();
                            let px = (y + block_y_start) * self.width * 3 + (x + block_x_start) * 3;
                            for i in 0..3 {
                                render[px + i] = (cn[i] * 255.0) as u8;
                            }
                        }
                    }
                }
            }
        }
        render
    }
}

