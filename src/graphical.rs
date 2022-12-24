use core::{
    cmp,
    fmt::{Display, Write},
    sync::atomic::{AtomicUsize, Ordering},
};

use bootloader_api::info::{FrameBuffer, PixelFormat};
use noto_sans_mono_bitmap::{BitmapHeight, FontWeight};

use crate::logger::Color;

const CORNERS_SIZE_RATION: f32 = 0.05;
const PADDING: usize = 14;


static mut FRAMEBUFFER: Option<FrameBuffer> = None;

pub fn init_framebuffer(framebuffer: Option<FrameBuffer>) {
    let mut framebuffer = if let Some(framebuffer) = framebuffer {
        framebuffer
    } else {
        return;
    };

    framebuffer.buffer_mut().fill(0);

    draw_test_colors(&mut framebuffer);

    unsafe {
        FRAMEBUFFER = Some(framebuffer);
    }
}

fn corners_size(framebuffer: &FrameBuffer) -> usize {
    let width = ((framebuffer.info().width as f32) * CORNERS_SIZE_RATION) as usize;
    let height = ((framebuffer.info().height as f32) * CORNERS_SIZE_RATION) as usize;
    cmp::min(width, height)
}

fn draw_test_colors(framebuffer: &mut FrameBuffer) {
    let size = corners_size(framebuffer);

    for x in 0..size {
        for y in 0..size {
            let width = framebuffer.info().width;
            let height = framebuffer.info().height;

            set_pixel(framebuffer, x, y, Color::Red, 255);
            set_pixel(framebuffer, width - x - 1, y, Color::Green, 255);
            set_pixel(framebuffer, x, height - y - 1, Color::Blue, 255);
            set_pixel(
                framebuffer,
                width - x - 1,
                height - y - 1,
                Color::Yellow,
                255,
            );
        }
    }
}

static LINE_IDX: AtomicUsize = AtomicUsize::new(0);
static COL_IDX: AtomicUsize = AtomicUsize::new(0);
pub fn write(display: impl Display, color: Color) {
    let framebuffer = if let Some(framebuffer) = unsafe { FRAMEBUFFER.as_mut() } {
        framebuffer
    } else {
        return;
    };


    let corners_size = corners_size(framebuffer);
    let width = framebuffer.info().width;
    let height = framebuffer.info().height;

    let mut formatter = Formatter {
        framebuffer,
        x: PADDING + corners_size + COL_IDX.load(Ordering::SeqCst) + 14,
        y: PADDING + LINE_IDX.load(Ordering::SeqCst),
        width,
        height,
        corners_size,
        color,
    };

    write!(formatter, "{display}").unwrap();
}

struct Formatter<'a> {
    framebuffer: &'a mut FrameBuffer,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    corners_size: usize,
    color: Color,
}

impl Write for Formatter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            let mut skip: bool = false;
            COL_IDX.fetch_add(14, Ordering::SeqCst);
            if c == '\n' {
                COL_IDX.store(0, Ordering::SeqCst);
                self.x = PADDING + self.corners_size + COL_IDX.load(Ordering::SeqCst);
                self.y = PADDING + LINE_IDX.fetch_add(14, Ordering::SeqCst);
                skip = true;
            } else if c == '\r' {
                COL_IDX.store(0, Ordering::SeqCst);
                self.x = PADDING + self.corners_size + COL_IDX.load(Ordering::SeqCst);
                skip = true;
            } else if self.x + 24 > self.width - self.corners_size {
                COL_IDX.store(0, Ordering::SeqCst);
                self.x = PADDING + self.corners_size + COL_IDX.load(Ordering::SeqCst);
                self.y = PADDING + LINE_IDX.fetch_add(14, Ordering::SeqCst);
            }

            // Scrolling mechanism:
            // It isn't perfect, but it works (for now)
            if self.y + 14 > self.height {
                LINE_IDX.fetch_sub(14, Ordering::SeqCst);
                self.y = PADDING + LINE_IDX.load(Ordering::SeqCst);
                let stride = self.framebuffer.info().stride;
                let bpp = self.framebuffer.info().bytes_per_pixel;
                let len = self.framebuffer.info().byte_len;
                let buffer = self.framebuffer.buffer_mut();
                buffer.copy_within(stride*bpp*(14)..len, 0);
                for p in len - stride * bpp * (14)..len {
                    // Slow
                    buffer[p] = 0;
                }

            }

            if !skip {
                self.write_char(c);
            }
        }
        Ok(())
    }
}

impl Formatter<'_> {
    fn write_char(&mut self, c: char) {
        let bitmap =
            noto_sans_mono_bitmap::get_bitmap(c, FontWeight::Regular, BitmapHeight::Size14)
                .unwrap_or_else(|| {
                    noto_sans_mono_bitmap::get_bitmap(
                        ' ',
                        FontWeight::Regular,
                        BitmapHeight::Size14,
                    )
                    .unwrap()
                });

        // Calculate the bounding box for log messages.
        let top = PADDING;
        let left = self.corners_size + PADDING;
        let bottom = self.height - PADDING - 1;
        let right = self.width - self.corners_size - PADDING - 1;

        for (bits, y) in bitmap.bitmap().iter().zip(self.y..) {
            for (&intensity, x) in bits.iter().zip(self.x..) {
                if y >= top && y <= bottom && x >= left && x <= right {
                    set_pixel(self.framebuffer, x, y, self.color, intensity);
                }
            }
        }
        self.x += bitmap.width();
    }
}

fn set_pixel(framebuffer: &mut FrameBuffer, x: usize, y: usize, color: Color, intensity: u8) {
    // Convert the color and intensity to bytes.
    let bytes = match framebuffer.info().pixel_format {
        PixelFormat::Rgb => [color.r(), color.g(), color.b(), 0],
        PixelFormat::Bgr => [color.b(), color.g(), color.r(), 0],
        PixelFormat::U8 => [color.greyscale(), 0, 0, 0],
        _ => {
            // Fall back to RGB.
            [color.r(), color.g(), color.b(), 0]
        }
    };
    let bytes = bytes.map(|pixel| (u16::from(pixel) * u16::from(intensity) / 255) as u8);
    let bytes = &bytes[..framebuffer.info().bytes_per_pixel];

    // Write the bytes to the framebuffer.
    let start = (framebuffer.info().stride * y + x) * framebuffer.info().bytes_per_pixel;
    framebuffer.buffer_mut()[start..][..bytes.len()].copy_from_slice(bytes);
}

impl Color {
    fn r(&self) -> u8 {
        match self {
            Color::White => 0xc0,
            Color::Red => 0xff,
            Color::Green => 0x00,
            Color::Blue => 0x00,
            Color::Yellow => 0xff,
        }
    }

    fn g(&self) -> u8 {
        match self {
            Color::White => 0xc0,
            Color::Red => 0x00,
            Color::Green => 0xff,
            Color::Blue => 0x00,
            Color::Yellow => 0xff,
        }
    }

    fn b(&self) -> u8 {
        match self {
            Color::White => 0xc0,
            Color::Red => 0x00,
            Color::Green => 0x00,
            Color::Blue => 0xff,
            Color::Yellow => 0x00,
        }
    }

    fn greyscale(&self) -> u8 {
        match self {
            Color::White => 0xc0,
            Color::Red => 0xc0,
            Color::Green => 0xc0,
            Color::Blue => 0xc0,
            Color::Yellow => 0xc0,
        }
    }
}
