extern crate sdl2;
extern crate image;

mod retina;

use retina::edge_detect;

use std::fs::File;
use std::path;
use std::io;
use image::GenericImage;
use std::env;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::mouse::MouseWheelDirection;
use image::RgbaImage;
use image::ColorType;
use image::ImageBuffer;
use image::Rgb;
use std::time::{Duration, Instant};

// 边缘检测的窗口测试
// 使用鼠标滚轮或者方向键调整阈值

/*
1.首先将图片色块化(相近的颜色统一为一种颜色/颜色相似度算法)
2.边缘检测(视网膜算法)
3.检测边缘线(八邻域边缘跟踪与区域生长算法)
https://blog.csdn.net/sinat_31425585/article/details/78558849
5.画线，合并多边形
图片资源:
https://weheartit.com/
*/

pub fn main() {

    let img = image::open("tubingen.png").unwrap().to_rgb();

    let (width, height) = (img.width() as usize, img.height() as usize);
    println!("width={},height={}", width, height);

    let buf = img.into_raw();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("边缘检测", width as u32, height as u32)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut threshold = 127.5;

    //提取边缘
    // let mut buffer = vec![0; buf.len()];
    // edge_detect(width, height, 24, &buf, &mut buffer, threshold, &[255, 0, 0, 255]);
    // let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();
    // img.save("test.png").unwrap();

    let bpp = 24;

    draw_edge(&buf, width, height, bpp, threshold, &mut canvas);

    'mainloop: loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit{..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
                        break 'mainloop,

                    Event::KeyDown {keycode: Option::Some(Keycode::Down), ..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Left), ..} =>{
                        threshold -= 1.0;
                        draw_edge(&buf, width, height, bpp, threshold, &mut canvas);
                    }

                    Event::KeyDown {keycode: Option::Some(Keycode::Up), ..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Right), ..} =>{
                        threshold += 1.0;
                        draw_edge(&buf, width, height, bpp, threshold, &mut canvas);
                    }

                    Event::MouseWheel {y, ..} =>{
                        threshold += 
                        match y{
                            -1 => -1.0,
                            1 => 1.0,
                            _ => 0.0
                        };
                        draw_edge(&buf, width, height, bpp, threshold, &mut canvas);
                    }
                    _ => {}
                }
            }
    }
}

fn draw_edge(bitmap:&Vec<u8>, width:usize, height:usize, bpp:usize, mut threshold:f32, canvas:&mut WindowCanvas){
    if threshold>255.0{
        threshold = 255.0;
    }
    if threshold<1.0{
        threshold = 1.0;
    }
    println!("阈值:{}", threshold);
    //提取边缘
    let start_time = Instant::now();
    let mut buffer = vec![0; bitmap.len()];
    edge_detect(width, height, bpp, bitmap, &mut buffer, threshold, &[255, 0, 0, 255]);
    println!("耗时:{}ms", duration_to_milis(&start_time.elapsed()));

    let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();

    //清空窗口
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    //绘制到窗口
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for y in 0..height{
        for x in 0..width{
            let pixel = img.get_pixel(x as u32, y as u32);
            if pixel[0] == 255{
                canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            }
        }
    }
    canvas.present();
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}