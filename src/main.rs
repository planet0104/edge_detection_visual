use rand::Rng;

mod retina;
use std::time::{Duration, Instant};
use svg::node::element::Line;
use svg::Document;

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

const WIDTH:f64 = 640.0;
const HEIGHT:f64 = 480.0;

use mengine::{Window, Image, Settings, Graphics, AssetsType, Assets, State};
use std::io::Result;

struct App {
    vectors_image: Option<Image>
}

impl State for App {
    fn new(window: &mut Window) -> Self {
        let image = image::open("luo1.png").unwrap().to_rgb();
        let (width, height) = (image.width(), image.height());
        let vectors = draw_contours(&image.into_raw(), width, height, vec![40, 130]);
        window.load_svg("vectors", vectors);
        App {
            vectors_image: None
        }
    }

    fn on_assets_load(
        &mut self,
        path: &str,
        _: AssetsType,
        assets: Result<Assets>,
        _window: &mut Window,
    ) {
        if path == "vectors" {
            if let Ok(assets) = assets {
                self.vectors_image = Some(assets.as_image().unwrap());
            }
        }
    }

    fn draw(&mut self, g: &mut Graphics, _window: &mut Window) {
        g.fill_rect(&[0, 0, 0, 255], 0., 0., WIDTH, HEIGHT);
        if let Some(image) = self.vectors_image.as_ref(){
            g.draw_image_at(None, image, 0.0, 0.0);
        }
    }

    fn update(&mut self, _window: &mut Window) {}
}

fn main() {
    mengine::run::<App>(
        "边缘检测",
        WIDTH,
        HEIGHT,
        Settings {
            show_ups_fps: true,
            background_color: Some([255, 255, 255, 255]),
            draw_center: false,
            ..Default::default()
        },
    );
}

//画轮廓
fn draw_contours(bitmap:&Vec<u8>, width:u32, height:u32, thresholds:Vec<u8>) -> String{
    println!("thresholds={:?}", thresholds);
    let start_time = Instant::now();
    let edges = retina::edge_detect(width, height, bitmap, thresholds);
    println!("边缘检测耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let contours = retina::edge_track(edges);
    println!("边缘跟踪耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    let start_time = Instant::now();
    let vectors = retina::contours_vectorize(&contours, 5, 5.0);
    println!("向量化耗时:{}ms", duration_to_milis(&start_time.elapsed()));
    
    let start_time = Instant::now();

    let mut document = Document::new()
            .set("viewBox", (0, 0, WIDTH, HEIGHT))
            .set("width", WIDTH)
            .set("height", HEIGHT);
    
    let mut line_count = 0;
    for lines in &vectors{
        let mut rng = rand::thread_rng();
        let color:&[u8; 3] = &[rng.gen_range(100, 255), rng.gen_range(100, 255), rng.gen_range(100, 255)];
        for i in 0..lines.len(){
            if i+1<lines.len(){
                line_count += 1;
                document = svg_draw_line(document, (lines[i].x as i32, lines[i].y as i32), (lines[i+1].x as i32, lines[i+1].y as i32), color, 0.1);
            }
        }
    }
    println!("绘图耗时:{}ms 线条数量:{}", duration_to_milis(&start_time.elapsed()), line_count);
    document.to_string()
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}

fn svg_draw_line(
    document: Document,
    start: (i32, i32),
    end: (i32, i32),
    color: &[u8; 3],
    stroke_width: f32,
) -> Document {
    //<line x1="0" y1="0" x2="300" y2="300" style="stroke:rgb(99,99,99);stroke-width:2"/>
    document.add(
        Line::new()
            .set("x1", start.0)
            .set("y1", start.1)
            .set("x2", end.0)
            .set("y2", end.1)
            .set(
                "style",
                format!(
                    "stroke:rgb({},{},{});stroke-width:{}",
                    color[0], color[1], color[2], stroke_width
                ),
            ),
    )
}