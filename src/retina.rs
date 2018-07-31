pub struct Point{
    pub x: i16,
    pub y: i16,
}

impl Point{
    pub fn new(x: i16, y:i16) -> Point{
        Point{x, y}
    }

    pub fn from_usize(x: usize, y: usize) -> Point{
        Point{x: x as i16, y: y as i16}
    }
}

impl Clone for Point{
    fn clone(&self) -> Point{
        Point{
            x: self.x,
            y: self.y
        }
    }
}

pub trait EdgePoints{
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn widthi(&self) -> i16;
    fn heighti(&self) -> i16;
    fn at(&mut self, x: i16, y: i16) -> &mut bool;
    fn atu(&mut self, x: usize, y: usize) -> &mut bool;
}

impl EdgePoints for Vec<Vec<bool>>{
    fn at(&mut self, x: i16, y: i16) -> &mut bool{
        &mut self[x as usize][y as usize]
    }
    fn atu(&mut self, x: usize, y: usize) -> &mut bool{
        &mut self[x][y]
    }
    fn width(&self) -> usize{
        self.len()
    }
    fn height(&self) -> usize{
        if self.len()>0{
            self[0].len()
        }else{
            0
        }
    }
    fn widthi(&self) -> i16{
        self.len() as i16
    }
    fn heighti(&self) -> i16{
        if self.len()>0{
            self[0].len() as i16
        }else{
            0
        }
    }
}

/*
    算法: 基于视网膜原理的边缘检测
    JiaYe 2018年1月

    视网膜水平细胞和双极细胞的功能如下:
    双极细胞 -- 亮光兴奋，弱光抑制。
    水平细胞 -- 亮光抑制，弱光兴奋，和双极细胞正好相反。

    算法：
    1.把每个像素点当作一个双极细胞，其右边和下边的像素点看作水平细胞，将像素点的亮度作为细胞输入。
    2.给定一个阈值，双极细胞和水平细胞根据阈值判断输入自身的是亮光还是弱光。
    3.计算将三个细胞的输出之和(双极细胞取两次)，如果没有抵消那么代表检测到一个边缘，否则没有检测到边缘。
    
    举例说明:
    
    B H B H B H
    H b h B H B
    B h B H B H
    H B H B H B

    上图中，字母代表图片的像素，B代表双极细胞, H代表水平细胞。
    小写b点代表当前像素点，那么当前像素点的输出等于4个细胞输出值之和除以4:
    pixel(1,1) = Sum(outB+outH+outB+outH)/4 (左下两个h点各取一次, b点取两次)))
    
    B和H的输出，根据亮度计算,如果像素亮度超过阈值，B输出255，H输出-255，没有超过阈值，二者都输出0。
*/


/// RGB888格式图像边缘检测
///
/// # Params
///
/// - `width`: 图像宽度.
/// - `src`: 图像数据.
/// - `out`: 输出，数组长度和原图像一致
/// - `threshold`: 阈值 0~255
/// - `out_color`: 输出颜色
pub fn edge_detect_draw(width:u32, src:&Vec<u8>, out:&mut Vec<u8>, thresholds:Vec<u8>, out_color: &[u8; 3]){
    edge_detect(width, src, thresholds, &mut |i|{
        out[i] = out_color[0];
        out[i+1] = out_color[1];
        out[i+2] = out_color[2];
    });
}

/// RGB888格式图像边缘检测
///
/// # Params
///
/// - `width`: 图像宽度.
/// - `height`: 图像宽度.
/// - `src`: 图像数据.
/// - `thresholds`: 阈值 0~255
/// 使用 result[x][y] 来获取点
pub fn edge_detect_points(width:u32, height:u32, src:&Vec<u8>, thresholds:Vec<u8>) -> Vec<Vec<bool>>{

    let mut edges = vec![vec![false; height as usize]; width as usize];

    edge_detect(width, src, thresholds, &mut |i|{
        let x = i/3%width as usize;
        let y = i/3/width as usize;
        edges[x][y] = true;
    });

    edges
}

/// RGB888格式图像边缘检测
///
/// # Params
///
/// - `width`: 图像宽度.
/// - `src`: 图像数据.
/// - `threshold`: 阈值 0~255
/// - `callback`: 检测到点的回调函数
pub fn edge_detect<F: FnMut(usize)>(width:u32, src:&Vec<u8>, thresholds:Vec<u8>, callback: &mut F){
    let bytepp = 3; //RGB888
    let size = src.len();
    let src = src.as_slice();

    for threshold in thresholds{
        let mut i = 0;
        while i<size{
            let (b1,b2,b3) = (i, i+1, i+2);
            let hrid = i+bytepp;
            let hbid = i+bytepp*width as usize;
            let b_out = calc_bipolar_cell(src[b1], src[b2], src[b3], threshold as f32);
            
            if hrid<size && hbid < size{
                let hr_out = calc_horizontal_cell(src[hrid], src[hrid+1], src[hrid+2], threshold as f32);
                let hb_out = calc_horizontal_cell(src[hbid], src[hbid+1], src[hbid+2], threshold as f32);

                if b_out*2.0+hr_out+hb_out != 0.0{
                    callback(i);
                }
            }
            i += bytepp;
        }
    }
}

/// 双极细胞 -- 亮光兴奋，弱光抑制
fn calc_bipolar_cell(r: u8, g:u8, b:u8, threshold: f32) -> f32{
    if 0.299*r as f32 + 0.587*g as f32 + 0.114*b as f32 >= threshold{
        1.0
    }else{
        -1.0
    }
}

///水平细胞 -- 亮光抑制，弱光兴奋
fn calc_horizontal_cell(r: u8, g:u8, b:u8, threshold: f32) -> f32{
    if 0.299*r as f32 + 0.587*g as f32 + 0.114*b as f32 >= threshold{
        -1.0
    }else{
        1.0
    }
}

pub fn vectorize(contours: &Vec<Vec<Point>>, min_distance:f32) -> Vec<Vec<Point>>{
    let mut lines = vec![];

    for contour in contours{
        let mut points = vec![contour[0].clone()];
        for i in 1..contour.len(){
            let pi = points.len()-1;
            let dist = (points[pi].x - contour[i].x)*(points[pi].x - contour[i].x)+(points[pi].y - contour[i].y)*(points[pi].y - contour[i].y);
            if (dist as f32).sqrt() >= min_distance{
                points.push(contour[i].clone());
            }
        }

        lines.push(points);
    }

    lines
}

//相近颜色转换为同一颜色
pub fn facet(width:u32, height:u32, block_size: u32, src:&Vec<u8>, out:&mut Vec<u8>){
    // let count_x = width/block_size;
    // let count_y = height/block_size;

    // for x in 0..count_x+1{
    //     for y in 0..count_y+1{
    //         facet_rect2(width, height, src, out, &[x*block_size, y*block_size, block_size, block_size], 5000, 2);
    //     }
    // }

    
    facet_rect(width, height, src, out, &[0, 0, width, height], 900);

    // let block_size = 4;
    // let count_x = width/block_size;
    // let count_y = height/block_size;

    // for x in 0..count_x+1{
    //     for y in 0..count_y+1{
    //         facet_rect2(width, height, src, out, &[x*block_size, y*block_size, block_size, block_size], 4000, 3);
    //     }
    // }
}

//将区域中的颜色数量减少N

// 参数说明
// D 颜色相似的最小距离
// N 区域中剩余最多颜色数

// 第一步将第一个像素放入 species,
// 检查第二个像素, 查看是否和species中的某个差距小于D，小于将其放入对应的species_map，否则也将其放入species
// 区域中所有像素都检查一遍
// 
// 循环species_map, 计算平均色。
// 
// species_map前N个，保留平均色
// species_map剩余的，计算其平均色与species_map前N个的距离，哪个距离近，就用前N个中哪个颜色替换。
fn facet_rect2(width:u32, _height:u32, src:&Vec<u8>, out:&mut Vec<u8>, rect: &[u32; 4], d: u32, n: usize){
    let i = (rect[1]*width*3+rect[0]*3) as usize;
    if i>=src.len(){
        return;
    }
    let mut species = vec![(src[i], src[i+1], src[i+2])];
    let mut species_map = vec![];
    let mut colors_map = vec![];

    for x in rect[0]..rect[0]+rect[2]{
        for y in rect[1]..rect[1]+rect[3]{
            let i = (y*width*3+x*3) as usize;

            if i>=src.len(){
                continue;
            }
            
            let (r,g,b) = (src[i],  src[i+1],  src[i+2]);

            let mut si = -1;

            for sp in 0..species.len(){
                let dist = color_diff(r, g, b, species[sp].0, species[sp].1, species[sp].2);
                if dist<d{
                    si = sp as i32;
                    break;
                }
            }

            if si == -1{
                si = species.len() as i32;
                species.push((r, g, b));
            }

            let si = si as usize;

            if species_map.len()<(si+1){
                species_map.push(vec![]);
            }

            species_map[si].push((r, g, b, i));
        }
    }

    //计算平均色
    for pixels in &species_map{
        let mut tr = 0;
        let mut tg = 0;
        let mut tb = 0;
        for pixel in pixels{
            tr += pixel.0 as u32;
            tg += pixel.1 as u32;
            tb += pixel.2 as u32;
        }
        let len = pixels.len() as u32;
        tr = tr/len;
        tg = tg/len;
        tb = tb/len;
        colors_map.push((tr as u8, tg as u8, tb as u8));
    }

    //计算平均色和前N个颜色的距离
    if colors_map.len()>n{
        for i in n..colors_map.len(){
            let mut fit_index = 0;
            let mut min_dist = ::std::u32::MAX;
            for g in 0..n{
                let dist = color_diff(colors_map[g].0, colors_map[g].1, colors_map[g].2,colors_map[i].0, colors_map[i].1, colors_map[i].2);
                if dist<min_dist{
                    min_dist = dist;
                    fit_index = g;
                }
            }
            colors_map[i].0 = colors_map[fit_index].0;
            colors_map[i].1 = colors_map[fit_index].1;
            colors_map[i].2 = colors_map[fit_index].2;
        }
    }

    let mut mapi = 0;
    for pixels in &species_map{
        for pixel in pixels{
            let i = pixel.3 as usize;
            out[i] = colors_map[mapi].0; 
            out[i+1] = colors_map[mapi].1; 
            out[i+2] = colors_map[mapi].2; 
        }
        mapi += 1;
    }
}

fn facet_rect(width:u32, _height:u32, src:&Vec<u8>, out:&mut Vec<u8>, rect: &[u32; 4], distance: u32){

    let i = (rect[1]*width*3+rect[0]*3) as usize;
    if i>=src.len(){
        return;
    }
    let mut species = vec![(src[i], src[i+1], src[i+2])];
    let mut species_map = vec![];

    for x in rect[0]..rect[0]+rect[2]{
        for y in rect[1]..rect[1]+rect[3]{
            let i = (y*width*3+x*3) as usize;

            if i>=src.len(){
                continue;
            }
            
            let (r,g,b) = (src[i],  src[i+1],  src[i+2]);

            let mut si = -1;

            for sp in 0..species.len(){
                let dist = color_diff(r, g, b, species[sp].0, species[sp].1, species[sp].2);
                if dist<distance{
                    si = sp as i32;
                    break;
                }
            }

            if si == -1{
                si = species.len() as i32;
                species.push((r, g, b));
            }

            let si = si as usize;

            if species_map.len()<(si+1){
                species_map.push(vec![]);
            }

            species_map[si].push((r, g, b, i));
        }
    }

    //计算平均色
    for pixels in &species_map{
        let mut tr = 0;
        let mut tg = 0;
        let mut tb = 0;
        for pixel in pixels{
            tr += pixel.0 as u32;
            tg += pixel.1 as u32;
            tb += pixel.2 as u32;
        }
        let len = pixels.len() as u32;
        tr = tr/len;
        tg = tg/len;
        tb = tb/len;
        for pixel in pixels{
            let i = pixel.3 as usize;
            out[i] = tr as u8; 
            out[i+1] = tg as u8; 
            out[i+2] = tb as u8; 
        }
    }
}

//简单计算颜色距离
//最大距离: 255*255*3=195075 二进制: 00000000_00000010_11111010_00000011
fn color_diff(r1: u8, g1:u8, b1: u8, r2: u8, g2:u8, b2: u8) -> u32{
    ((r2 as i32-r1 as i32)*(r2 as i32-r1 as i32) + (g2 as i32-g1 as i32)*(g2 as i32-g1 as i32) + (b2 as i32-b1 as i32)*(b2 as i32-b1 as i32)) as u32
}

// 8邻域
const NEIGHBORS:[Point; 8] = [ Point{ x:0, y:1 }, Point{ x:1, y:1}, Point{x:1, y:0}, Point{x:1, y:-1}, 
                             Point{x:0, y:-1}, Point{x:-1, y:-1}, Point{x:-1, y:0}, Point{x:-1, y:1} ];

pub fn track_edge(mut edges:Vec<Vec<bool>>)->Vec<Vec<Point>>{
    let mut seeds:Vec<Point> = vec![];
    let mut contours: Vec<Vec<Point>> = vec![];
    for x in 0..edges.height(){
		for y in 0..edges.width(){    
	//for x in 0..edges.width(){
//		for y in 0..edges.height(){
			//如果当前点为轮廓点
			if edges[x][y]{
                let mut contour: Vec<Point> = vec![];
				// 当前点清零
                edges[x][y] = false;
 
				// 存入种子点及轮廓
				seeds.push(Point::from_usize(x, y));
				contour.push(Point::from_usize(x, y));
 
				// 区域生长
				while seeds.len() > 0{
					// 遍历8邻域
					for k in 0..8{
						// 更新当前点坐标
						let new_x = seeds[0].x + NEIGHBORS[k].x;
						let new_y = seeds[0].y + NEIGHBORS[k].y;
 
						// 边界界定
						if (new_x >= 0)  && (new_x <= edges.widthi() - 1) &&
							(new_y >= 0) && (new_y <= edges.heighti() - 1){
							if *edges.at(new_x, new_y){
								// 当前点清零
                                *edges.at(new_x, new_y) = false;
 
								// 存入种子点及轮廓
								seeds.push(Point::new(new_x, new_y));
								contour.push(Point::new(new_x, new_y));
							}// end if
						}
					} // end for
 
					// 删除第一个元素
					seeds.remove(0);
 
				}// end while
 
				contours.push(contour);
 
			}// end if
		}
    }
    contours
}