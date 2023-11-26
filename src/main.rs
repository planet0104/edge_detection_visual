use image::RgbaImage;
use image::buffer::ConvertBuffer;
use image::{ImageBuffer, Rgb, RgbImage, DynamicImage};
use slint::LogicalSize;
use slint::SharedPixelBuffer;
mod retina;
use std::time::{Duration, Instant};
use imageproc::drawing::*;
use imageproc::hough::*;
use anyhow::Result;

slint::slint!{
    export component HelloWorld {
        in property <image> img;
        Image {
            width: 100%;
            height: 100%;
            source: img;
        }
    }
}

pub fn main() -> Result<()> {
    const MASK_COLOR: u32 = 0xFF00FF;

    let img_name = "book2.jpg";

    let img = image::open(img_name).unwrap().to_rgb8();

    let (width, height) = (img.width(), img.height());
    println!("width={},height={}", width, height);

    let buf = img.into_raw();

    let mut img_rgb:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);

    // draw_contours(&buf, width, height, vec![50], &mut img_rgb);
    draw_contours(&buf, width, height, vec![50], &mut img_rgb);


    let start_time = Instant::now();

    let img = DynamicImage::ImageRgb8(img_rgb.clone()).to_luma8();
    // Detect lines using Hough transform
    let options = LineDetectionOptions {
        vote_threshold: 40,
        suppression_radius: 60,
    };
    let lines_raw: Vec<PolarLine> = detect_lines(&img, options);
    let mut lines:Vec<((f32, f32), (f32, f32))> = vec![];
    let min = 10.0;
    //如果两条线顶点距离小于min，只取1条
    for line in &lines_raw{
        let l2 = intersection_points(*line, img.width(), img.height()).unwrap();
        let mut find = false;
        for l1 in &lines{
            let d1 = calc_distance(&l1.0, &l2.0);
            let d2 = calc_distance(&l1.0, &l2.1);
            let d3 = calc_distance(&l1.1, &l2.0);
            let d4 = calc_distance(&l1.1, &l2.1);
            if d1<min || d2<min || d3<min || d4<min{
                find = true;
            }
        }
        if !find{
            lines.push(l2);
        }
    }
    //按照角度排序
    lines.sort_by(|l1, l2|{
        let angle1 = ((calc_angle(&l1.0, &l1.1).to_degrees() * 100.) as i32).abs();
        let angle2 = ((calc_angle(&l2.0, &l2.1).to_degrees() * 100.) as i32).abs();
        angle1.cmp(&angle2)
    });

    let mut img_lines = image::open(img_name).unwrap().to_rgb8();

    for line_point in &lines{
        draw_line_segment_mut(&mut img_lines, line_point.0, line_point.1, Rgb([255, 255, 0]));
    }

    let horizontal_lines = vec![ lines.pop().unwrap(), lines.pop().unwrap() ];
    let vertical_lines = vec![ lines.pop().unwrap(), lines.pop().unwrap() ];

    // println!("直线检测耗时:{}ms", duration_to_milis(&start_time.elapsed()));

    //计算交点
    //取出两条横线
    let c1 = line_ntersection(&horizontal_lines[0].0,&horizontal_lines[0].1, &vertical_lines[0].0, &vertical_lines[0].1);
    let c2 = line_ntersection(&horizontal_lines[0].0,&horizontal_lines[0].1, &vertical_lines[1].0, &vertical_lines[1].1);
    let c3 = line_ntersection(&horizontal_lines[1].0,&horizontal_lines[1].1, &vertical_lines[0].0, &vertical_lines[0].1);
    let c4 = line_ntersection(&horizontal_lines[1].0,&horizontal_lines[1].1, &vertical_lines[1].0, &vertical_lines[1].1);

    draw_hollow_ellipse_mut(&mut img_lines, (c1.0 as i32, c1.1 as i32), 5, 5, Rgb([255, 255, 0]));
    draw_hollow_ellipse_mut(&mut img_lines, (c2.0 as i32, c2.1 as i32), 5, 5, Rgb([255, 255, 0]));
    draw_hollow_ellipse_mut(&mut img_lines, (c3.0 as i32, c3.1 as i32), 5, 5, Rgb([255, 255, 0]));
    draw_hollow_ellipse_mut(&mut img_lines, (c4.0 as i32, c4.1 as i32), 5, 5, Rgb([255, 255, 0]));

    println!("{:?}", c1);
    println!("{:?}", c2);
    println!("{:?}", c3);
    println!("{:?}", c4);

    // 右上下，左上下，上左右，下左右，右下上，左下上
    // let l1p = &lines[5];
    // let line_point = intersection_points(*l1p, img.width(), img.height()).unwrap();
    // draw_filled_ellipse_mut(&mut img_rgb, (line_point.0.0 as i32, line_point.0.1 as i32), 5, 5, Rgb([255, 255, 0]));
    // draw_filled_ellipse_mut(&mut img_rgb, (line_point.1.0 as i32, line_point.1.1 as i32), 5, 5, Rgb([255, 255, 255]));

    // let mut img_lines = image::open(img_name).unwrap().to_rgb8();
    // draw_polar_lines_mut(&mut img_lines, &lines, Rgb([255, 255, 0]));
    // img_lines.save("out3.png").unwrap();
    // lines_image.save("lines.png").unwrap();
    // for line in lines{
    //     // println!("{:?}", corner);
    //     draw_hollow_ellipse_mut(&mut img_rgb, (corner.x as i32, corner.y as i32), 5, 5, Rgb([255, 255, 0]));
    // }

    let app = HelloWorld::new()?;
    app.window().set_size(LogicalSize::new(width as f32, height as f32));
    let rgb_image:RgbaImage = img_lines.convert();
    let buf = SharedPixelBuffer::clone_from_slice(&rgb_image, rgb_image.width(), rgb_image.height());
    app.set_img(slint::Image::from_rgba8(buf));
    let _ = app.run()?;
    Ok(())
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
        // let mut rng = rand::thread_rng();
        // let r:u8 = rng.gen_range(100..255);
        // let g:u8 = rng.gen_range(100..255);
        // let b:u8 = rng.gen_range(100..255);
        // let color = Rgb([r, g, b]);
        let color = Rgb([255, 255, 255]);
        for i in 0..lines.len(){
            if i+1<lines.len(){
                draw_line_segment_mut(canvas, (lines[i].x as f32, lines[i].y as f32), (lines[i+1].x as f32, lines[i+1].y as f32), color);
            }
        }
    }
}

pub fn duration_to_milis(duration: &Duration) -> f64 {
    duration.as_secs() as f64 * 1000.0 + duration.subsec_nanos() as f64 / 1_000_000.0
}

/// Returns the intersection points of a `PolarLine` with an image of given width and height,
/// or `None` if the line and image bounding box are disjoint. The x value of an intersection
/// point lies within the closed interval [0, image_width] and the y value within the closed
/// interval [0, image_height].
fn intersection_points(
    line: PolarLine,
    image_width: u32,
    image_height: u32,
) -> Option<((f32, f32), (f32, f32))> {
    let r = line.r;
    let m = line.angle_in_degrees;
    let w = image_width as f32;
    let h = image_height as f32;

    // Vertical line
    if m == 0 {
        return if r >= 0.0 && r <= w {
            Some(((r, 0.0), (r, h)))
        } else {
            None
        };
    }

    // Horizontal line
    if m == 90 {
        return if r >= 0.0 && r <= h {
            Some(((0.0, r), (w, r)))
        } else {
            None
        };
    }

    let theta = (m as f32).to_radians();
    let (sin, cos) = theta.sin_cos();

    let right_y = cos.mul_add(-w, r) / sin;
    let left_y = r / sin;
    let bottom_x = sin.mul_add(-h, r) / cos;
    let top_x = r / cos;

    let mut start = None;

    if right_y >= 0.0 && right_y <= h {
        let right_intersect = (w, right_y);
        if let Some(s) = start {
            return Some((s, right_intersect));
        }
        start = Some(right_intersect);
    }

    if left_y >= 0.0 && left_y <= h {
        let left_intersect = (0.0, left_y);
        if let Some(s) = start {
            return Some((s, left_intersect));
        }
        start = Some(left_intersect);
    }

    if bottom_x >= 0.0 && bottom_x <= w {
        let bottom_intersect = (bottom_x, h);
        if let Some(s) = start {
            return Some((s, bottom_intersect));
        }
        start = Some(bottom_intersect);
    }

    if top_x >= 0.0 && top_x <= w {
        let top_intersect = (top_x, 0.0);
        if let Some(s) = start {
            return Some((s, top_intersect));
        }
    }

    None
}


fn calc_angle(pt1:&(f32, f32), pt2:&(f32, f32)) -> f32{
	let dx = pt1.0 - pt2.0;
	let dy = pt1.1 - pt2.1;
	if dx == 0.{
		if dy < 0.{
			return std::f32::consts::PI / 2.0;
		}
		else if dy > 0. {
			return -std::f32::consts::PI / 2.0;
		} else {
			return 0.0;
		}
	}
	(dy / dx).atan()
}


/**
 * Calculate angle between two lines with two given points
 *
 * @param A1 First point first line
 * @param A2 Second point first line
 * @param B1 First point second line
 * @param B2 Second point second line
 * @return Angle between two lines in degrees
 */
fn calc_line_angle(a1: &(f32, f32), a2: &(f32, f32), b1: &(f32, f32), b2: &(f32, f32)) -> f32{
    let angle1 = (a2.1 - a1.1).atan2(a1.0 - a2.0);
    let angle2 = (b2.1 - b1.1).atan2(b1.0 - b2.0);
    let mut calculated_angle = (angle1 - angle2) * 180. / std::f32::consts::PI;
    if calculated_angle < 0. { calculated_angle += 360. };
    calculated_angle
}

fn calc_distance(p1: &(f32, f32), p2: &(f32, f32)) -> f32{
    // let x1 = p1.0;
    // let y1 = p1.1;
    // let x2 = p2.0;
    // let y2 = p2.1;
    ((p2.0-p1.0)*(p2.0-p1.0)+(p2.1-p1.1)*(p2.1-p1.1)).sqrt()
}

//计算两条直线的交点
fn get_cross(line1:&((f32, f32), (f32, f32)), line2:&((f32, f32), (f32, f32))) -> (f32, f32){
    let mut point = (0., 0.);
    //y = a * x + b;
    let a1 = (line1.0.1 - line1.1.1) / (line1.0.0 - line1.1.0);
    let b1 = line1.0.1 - a1 * (line1.0.0);

    let a2 = (line2.0.1 - line2.1.1) / (line2.0.0 - line2.1.0);
    let b2 = line2.0.1 - a2 * (line2.0.0);

    point.0 = b1 - b2 / a2 - a1;
    point.1 = a1 * point.0 + b1;
    point
}

// 计算直线交点
fn line_ntersection(a:&(f32, f32), b:&(f32, f32), c:&(f32, f32), d:&(f32, f32)) -> (f32, f32) { 
    // Line AB represented as a1x + b1y = c1 
    let a1 = b.1 - a.1;
    let b1 = a.0 - b.0; 
    let c1 = a1*(a.0) + b1*(a.1); 
  
    // Line CD represented as a2x + b2y = c2 
    let a2 = d.1 - c.1; 
    let b2 = c.0 - d.0; 
    let c2 = a2*(c.0)+ b2*(c.1); 
  
    let determinant = a1*b2 - a2*b1; 
  
    if determinant == 0.{
        // The lines are parallel. This is simplified 
        // by returning a pair of FLT_MAX 
        (0., 0.)
    } else { 
        let x = (b2*c1 - b1*c2)/determinant; 
        let y = (a1*c2 - a2*c1)/determinant; 
        (x, y)
    } 
} 