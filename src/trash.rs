    //提取边缘
    // let mut buffer = vec![0; buf.len()];
    // edge_detect(width, height, 24, &buf, &mut buffer, threshold, &[255, 0, 0, 255]);
    // let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width as u32, height as u32, buffer).unwrap();
    // img.save("test.png").unwrap();

    //let mut out = vec![0; buf.len()];

    // retina::facet(width, height, 2, &buf, &mut out);

    // let img:ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, out).unwrap();
    // img.save("test.png").unwrap();

        //画线
    //let points:Vec<Point> = points.iter().map(|point|{ Point::new(point.x as i32, point.y as i32) }).collect();
    //canvas.draw_lines(points.get(0..50).unwrap()).unwrap();

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


// 8邻域
const NEIGHBORS:[Point; 8] = [ Point{ x:0, y:1 }, Point{ x:1, y:1}, Point{x:1, y:0}, Point{x:1, y:-1}, 
                             Point{x:0, y:-1}, Point{x:-1, y:-1}, Point{x:-1, y:0}, Point{x:-1, y:1} ];

/// 区域生长算法
/// 此算法检测出的边缘点不是连续的!
pub fn track_edge_grow(mut edges:Vec<Vec<bool>>)->Vec<Vec<Point>>{
    let mut seeds:Vec<Point> = vec![];
    let mut contours: Vec<Vec<Point>> = vec![];
    for x in 0..edges.height(){
		for y in 0..edges.width(){    
	//for x in 0..edges.width(){
//		for y in 0..edges.height(){
			//如果当前点为轮廓点
			if *edges.atu(x, y){
                let mut contour: Vec<Point> = vec![];
				// 当前点清零
                *edges.atu(x, y) = false;
 
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