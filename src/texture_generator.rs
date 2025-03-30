use image::{ImageBuffer, Rgb};
use noise::{NoiseFn, Perlin};

pub fn generate_stone_texture() -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let size = 64;
    let noise = Perlin::new(42);
    let mut img = ImageBuffer::new(size, size);

    for x in 0..size {
        for y in 0..size {
            let value = noise.get([x as f64 / 8.0, y as f64 / 8.0]);
            let value = ((value + 1.0) * 127.5) as u8;
            img.put_pixel(x, y, Rgb([value, value, value]));
        }
    }

    img
} 