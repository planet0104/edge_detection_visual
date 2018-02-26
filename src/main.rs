extern crate sdl2;
extern crate lodepng;

use lodepng::{RGB, Bitmap};
use std::env;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use sdl2::rect::Point;
use sdl2::render::WindowCanvas;
use sdl2::pixels::Color;
use sdl2::mouse::MouseWheelDirection;

// 边缘检测的窗口测试
// 使用鼠标滚轮或者方向键调整阈值

pub fn main() {
    let bitmap = lodepng::decode24_file("image.png").unwrap();
    let (width, height) = (bitmap.width, bitmap.height);
    println!("width={},height={}", width, height);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    
    let window = video_subsystem.window("边缘检测", width as u32, height as u32)
      .position_centered()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut threshold = 127.0;
    draw_edge(&bitmap, &mut threshold, &mut canvas);

    'mainloop: loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::Quit{..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Escape), ..} =>
                        break 'mainloop,

                    Event::KeyDown {keycode: Option::Some(Keycode::Down), ..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Left), ..} =>{
                        threshold -= 1.0;
                        draw_edge(&bitmap, &mut threshold, &mut canvas);
                    }

                    Event::KeyDown {keycode: Option::Some(Keycode::Up), ..} |
                    Event::KeyDown {keycode: Option::Some(Keycode::Right), ..} =>{
                        threshold += 1.0;
                        draw_edge(&bitmap, &mut threshold, &mut canvas);
                    }

                    Event::MouseWheel {y, ..} =>{
                        threshold += 
                        match y{
                            -1 => -1.0,
                            1 => 1.0,
                            _ => 0.0
                        };
                        draw_edge(&bitmap, &mut threshold, &mut canvas);
                    }
                    _ => {}
                }
            }
    }
}

fn draw_edge(bitmap:&Bitmap<RGB<u8>>, threshold:&mut f32, canvas:&mut WindowCanvas){
    if *threshold>255.0{
        *threshold = 255.0;
    }
    if *threshold<1.0{
        *threshold = 1.0;
    }
    println!("阈值:{}", threshold);
    //提取边缘
    let mut buffer:Vec<RGB<u8>> = vec![RGB::new(0, 0, 0); bitmap.width*bitmap.height];
    edge_detection(&bitmap.buffer, &mut buffer, bitmap.width, bitmap.height, *threshold);

    //清空窗口
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    //canvas.present();
    //绘制到窗口
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for y in 0..bitmap.height{
        for x in 0..bitmap.width{
            if buffer[y*bitmap.width+x].r == 255{
                canvas.draw_point(Point::new(x as i32, y as i32));
            }
        }
    }
    canvas.present();
}

//边缘检测
//buffer: 图像数据
//out_buffer: 输出到
//height: 图像高度
//width: 图像宽度
//threshold: 阈值0~255
//返回: 黑底百色边缘的图像数据
fn edge_detection(buffer:&Vec<RGB<u8>>, out_buffer:&mut Vec<RGB<u8>>, width:usize, height:usize, threshold:f32){
    let mut i = 0;

    for _row in 0..height{
        //第一列是否是双极细胞
        for _col in 0..width{
            //4个像素
            //双极细胞 给光ON，撤光OFF => 超过阈值:255
            //水平细胞 亮光抑制，弱光增强，和双极细胞正好相反 => 超过阈值:-255
            match((calc_pixel(buffer.get(i).unwrap_or(&buffer[i]), 255.0, threshold)
                        + calc_pixel(buffer.get(i+1).unwrap_or(&buffer[i]), -255.0, threshold)
                        +calc_pixel(buffer.get(i).unwrap_or(&buffer[i]), 255.0, threshold)
                        + calc_pixel(buffer.get(i+width).unwrap_or(&buffer[i]), -255.0, threshold)
                        )/4.0) as i32{
                0 => (),
                _ =>{
                    out_buffer[i].r = 255;
                    out_buffer[i].g = 255;
                    out_buffer[i].b = 255;
                }
            }
            i += 1;
        }
    }
}

//计算每个像素的输出
// p: 像素
// out: 超过阈值细胞的输出
// threshold: 阈值0~255
fn calc_pixel(pixel:&RGB<u8>, out:f32, threshold:f32)->f32{
    //二值化以后根据双极细胞、水平细胞返回输出值
    if 0.299*pixel.r as f32+0.587*pixel.g as f32+0.114*pixel.b as f32>threshold{
        out
    }else{
        0.0 //弱光都不返回
    }
}