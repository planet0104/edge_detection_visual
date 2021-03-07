use image::{ImageBuffer, Rgb, RgbImage};
use rand::Rng;
mod retina;
use std::time::{Duration, Instant};
use minifb::{Key, Window, WindowOptions};
use imageproc::drawing::*;
use blit::*;

pub fn main() {
    const MASK_COLOR: u32 = 0xFF00FF;

    let img = image::open("lena.jpg").unwrap().to_rgb8();

    let (width, height) = (img.width(), img.height());
    println!("width={},height={}", width, height);

    let buf = img.into_raw();

    let mut img_rgb:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // draw_contours(&buf, width, height, vec![50], &mut img_rgb);
    draw_contours(&buf, width, height, vec![40, 130], &mut img_rgb);

    let mut buffer: Vec<u32> = vec![0; (width * height) as usize];

    let mut window = Window::new(
        "边缘检测 - ESC to exit",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        img_rgb.blit(&mut buffer, width as usize, (0, 0), Color::from_u32(MASK_COLOR));

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .unwrap();
    }
}

//画轮廓
fn draw_contours(bitmap:&Vec<u8>, width:u32, height:u32, thresholds:Vec<u8>, canvas:&mut RgbImage){
    println!("thresholds={:?}", thresholds);
    let start_time = Instant::now();
    let edges = retina::edge_detect(width, height, bitmap, thresholds);
    println!("边缘检测耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let contours = retina::edge_track(edges);
    println!("边缘跟踪耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let vectors = retina::contours_vectorize(&contours, 3, 2.0);
    println!("向量化耗时:{}ms", duration_to_milis(&start_time.elapsed()));

    for lines in vectors{
        let mut rng = rand::thread_rng();
        let r:u8 = rng.gen_range(100..255);
        let g:u8 = rng.gen_range(100..255);
        let b:u8 = rng.gen_range(100..255);
        for i in 0..lines.len(){
            if i+1<lines.len(){
                draw_line_segment_mut(canvas, (lines[i].x as f32, lines[i].y as f32), (lines[i+1].x as f32, lines[i+1].y as f32), Rgb([r, g, b]));
            }
        }
    }
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}