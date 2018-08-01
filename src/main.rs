extern crate sdl2;
extern crate image;
extern crate rand;

use rand::Rng;

mod retina;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use std::time::{Duration, Instant};

// 一、照片变卡通
// 二、照片变矢量图

/**

碰撞使图像扩大和腐蚀使图像缩小
https://blog.csdn.net/qq_33200959/article/details/76072639

开操作使图像轮廓变得光滑，断开狭窄的间断和消除细的突出物
闭操作使轮廓线变光滑，消弥狭窄的间断和长细的鸿沟，消除小的孔洞，并填补轮廓线中的断裂。


 
    http://autotrace.sourceforge.net/ bitmap转svg
    http://potrace.sourceforge.net/ bitmap转svg

    http://jhlabs.com/ip/filters/PosterizeFilter.html 减少颜色通道
    http://jhlabs.com/ip/filters/SmearFilter.html 像素涂抹
    http://jhlabs.com/ip/filters/ReduceNoiseFilter.html 8邻降噪

 */

/**
 
 PS实验1： 
    选择 滤镜->像素化->彩块化, 可以将图片转换成彩色快，这些彩块其实就可以转换成矢量图。
    多次进行彩块化以后，色块变大，这时候再去检测图像边缘，不同阈值时边缘波动小。

 PS实验2:
    选择 图像->色调分离，可以将图像颜色减少。同时图像边缘更清晰。
    色调分离以后，在进行 滤镜->(像素化->彩块化)，或者滤镜库的涂抹效果，也可以清晰图像边缘。

    测试2:
            1、首先在不同的阈值下进行线条检测
            2、每次检测完成，将较短的线条删除

 */
pub fn main() {
    //tubingen
    let img = image::open("lena.jpg").unwrap().to_rgb();

    let (width, height) = (img.width(), img.height());
    println!("width={},height={}", width, height);

    let buf = img.into_raw();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("边缘检测", width, height)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    draw_contours(&buf, width, height, vec![40, 130], &mut canvas);

    'mainloop: loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit{..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
                        break 'mainloop,
                    _ => {}
                }
            }
    }
}

//画轮廓
fn draw_contours(bitmap:&Vec<u8>, width:u32, height:u32, thresholds:Vec<u8>, canvas:&mut WindowCanvas){
    println!("thresholds={:?}", thresholds);
    let start_time = Instant::now();
    let edges = retina::edge_detect(width, height, bitmap, thresholds);
    println!("边缘检测耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let contours = retina::edge_track(edges);
    println!("边缘跟踪耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let vectors = retina::contours_vectorize(&contours, 5, 3.0);
    println!("向量化耗时:{}ms", duration_to_milis(&start_time.elapsed()));

    for lines in vectors{
        let mut rng = rand::thread_rng();
        canvas.set_draw_color(Color::RGB(rng.gen_range(100, 255), rng.gen_range(100, 255), rng.gen_range(100, 255)));
        for i in 0..lines.len(){
            if i+1<lines.len(){
                canvas.draw_line(Point::new(lines[i].x as i32, lines[i].y as i32), Point::new(lines[i+1].x as i32, lines[i+1].y as i32)).unwrap();
            }
        }
    }
    canvas.present();
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}