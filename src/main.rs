extern crate sdl2;
extern crate image;
extern crate rand;

use rand::Rng;

mod retina;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use image::ImageBuffer;
use image::Rgb;
use sdl2::rect::Point;
use std::time::{Duration, Instant};

// 边缘检测的窗口测试
// 使用鼠标滚轮或者方向键调整阈值

/*
1.首先将图片色块化(相近的颜色统一为一种颜色/颜色相似度算法)
2.边缘检测(视网膜算法)
3.检测边缘线(八邻域边缘跟踪与区域生长算法)
https://blog.csdn.net/sinat_31425585/article/details/78558849
http://www.imageprocessingplace.com/downloads_V3/root_downloads/tutorials/contour_tracing_Abeer_George_Ghuneim/square.html
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
    let img = image::open("sfz.png").unwrap().to_rgb();

    let (width, height) = (img.width(), img.height());
    println!("width={},height={}", width, height);

    let buf = img.into_raw();
    //let mut out = vec![0; buf.len()];

    // retina::facet(width, height, 2, &buf, &mut out);

    // let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, out).unwrap();
    // img.save("test.png").unwrap();


    

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("边缘检测", width as u32, height as u32)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    //提取边缘
    // let mut buffer = vec![0; buf.len()];
    // edge_detect(width, height, 24, &buf, &mut buffer, threshold, &[255, 0, 0, 255]);
    // let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();
    // img.save("test.png").unwrap();

    let mut threshold1 = 46;
    let mut threshold2 = 90;
    let mut threshold3 = 204;

    //draw_edge(&buf, width, height, threshold, &mut canvas);
    draw_contours(&buf, width, height, vec![125, 255], &mut canvas);

    'mainloop: loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit{..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
                        break 'mainloop,

                    Event::KeyDown {keycode: Option::Some(Keycode::Down), ..} => {
                        if threshold1 > 1{
                            threshold1 -= 1;
                        }
                        draw_contours(&buf, width, height, vec![threshold1, threshold2, threshold3], &mut canvas);
                    }
                    Event::KeyDown {keycode: Option::Some(Keycode::Up), ..} => {
                        if threshold1 < 255{
                            threshold1 += 1;
                        }
                        draw_contours(&buf, width, height, vec![threshold1, threshold2, threshold3], &mut canvas);
                    }


                    Event::KeyDown {keycode: Option::Some(Keycode::Left), ..} =>{
                        if threshold3 > 1{
                            threshold3 -= 1;
                        }
                        draw_contours(&buf, width, height, vec![threshold1, threshold2, threshold3], &mut canvas);
                    }

                    Event::KeyDown {keycode: Option::Some(Keycode::Right), ..} =>{
                        if threshold3 < 255{
                            threshold3 += 1;
                        }
                        draw_contours(&buf, width, height, vec![threshold1, threshold2, threshold3], &mut canvas);
                    }

                    Event::MouseWheel {y, ..} =>{
                        match y{
                            -1 => threshold2 -= 1,
                            1 => threshold2 += 1,
                            _ => ()
                        };
                        draw_contours(&buf, width, height, vec![threshold1, threshold2, threshold3], &mut canvas);
                    }
                    _ => {}
                }
            }
    }
}

//画轮廓
fn draw_contours(bitmap:&Vec<u8>, width:u32, height:u32, thresholds:Vec<u8>, canvas:&mut WindowCanvas){
    let thresholds = vec![thresholds.first().unwrap().clone()];
    println!("thresholds={:?}", thresholds);
    let start_time = Instant::now();
    let edges = retina::edge_detect(width, height, bitmap, thresholds);
    println!("边缘检测耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let contours = retina::edge_track(edges);
    println!("边缘跟踪耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let vectors = retina::contours_vectorize(&contours, 80, 10.0);
    println!("向量化耗时:{}ms", duration_to_milis(&start_time.elapsed()));

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let mut rng = rand::thread_rng();

    //画线
    //let points:Vec<Point> = points.iter().map(|point|{ Point::new(point.x as i32, point.y as i32) }).collect();
    //canvas.draw_lines(points.get(0..50).unwrap()).unwrap();

    for lines in vectors{
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