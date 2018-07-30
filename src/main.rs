extern crate sdl2;
extern crate image;

mod retina;

use retina::edge_detect;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
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


/**
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
    let img = image::open("tubingen.png").unwrap().to_rgb();

    let (width, height) = (img.width(), img.height());
    println!("width={},height={}", width, height);

    let buf = img.into_raw();
    let mut out = vec![0; buf.len()];

    retina::facet(width, height, 2, &buf, &mut out);

    let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, out).unwrap();
    img.save("test.png").unwrap();


    

    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    
    // let window = video_subsystem.window("边缘检测", width as u32, height as u32)
    //   .position_centered()
    //   .build()
    //   .unwrap();

    // let mut canvas = window.into_canvas().build().unwrap();
    // let mut threshold = 127.5;

    //提取边缘
    // let mut buffer = vec![0; buf.len()];
    // edge_detect(width, height, 24, &buf, &mut buffer, threshold, &[255, 0, 0, 255]);
    // let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();
    // img.save("test.png").unwrap();

    // draw_edge(&buf, width, height, threshold, &mut canvas);

    // 'mainloop: loop {
    //         for event in sdl_context.event_pump().unwrap().poll_iter() {
    //             match event {
    //                 Event::Quit{..} |
    //                 Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
    //                     break 'mainloop,

    //                 Event::KeyDown {keycode: Option::Some(Keycode::Down), ..} |
    //                 Event::KeyDown {keycode: Option::Some(Keycode::Left), ..} =>{
    //                     threshold -= 1.0;
    //                     draw_edge(&buf, width, height, threshold, &mut canvas);
    //                 }

    //                 Event::KeyDown {keycode: Option::Some(Keycode::Up), ..} |
    //                 Event::KeyDown {keycode: Option::Some(Keycode::Right), ..} =>{
    //                     threshold += 1.0;
    //                     draw_edge(&buf, width, height, threshold, &mut canvas);
    //                 }

    //                 Event::MouseWheel {y, ..} =>{
    //                     threshold += 
    //                     match y{
    //                         -1 => -1.0,
    //                         1 => 1.0,
    //                         _ => 0.0
    //                     };
    //                     draw_edge(&buf, width, height, threshold, &mut canvas);
    //                 }
    //                 _ => {}
    //             }
    //         }
    // }
}

fn draw_edge(bitmap:&Vec<u8>, width:u32, height:u32, mut threshold:f32, canvas:&mut WindowCanvas){
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
    edge_detect(width, height, bitmap, &mut buffer, threshold, &[255, 0, 0]);
    println!("耗时:{}ms", duration_to_milis(&start_time.elapsed()));

    let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, buffer).unwrap();

    //清空窗口
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    //绘制到窗口
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for y in 0..height{
        for x in 0..width{
            let pixel = img.get_pixel(x, y);
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