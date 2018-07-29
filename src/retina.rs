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


/// 边缘检测
///
/// # Params
///
/// - `width`: 图像宽度.
/// - `height`: 图像宽度.
/// - `bpp`: 像素深度 24或32
/// - `src`: 图像数据.
/// - `out`: 输出，数组长度和原图像一致
/// - `threshold`: 阈值 0~255
/// - `out_color`: 输出颜色
pub fn edge_detect(width:usize, _height:usize, bpp: usize, src:&Vec<u8>, out:&mut Vec<u8>, threshold:f32, out_color: &[u8; 4]){
    let bytepp = bpp/8;
    let size = src.len();
    let src = src.as_slice();
    let out = out.as_mut_slice();

    let mut i = 0;

    while i<size{
        let (b1,b2,b3) = (i, i+1, i+2);
        let hrid = i+bytepp;
        let hbid = i+bytepp*width;
        let b_out = calc_bipolar_cell(src[b1], src[b2], src[b3], threshold);
        
        if hrid<size && hbid < size{
            let hr_out = calc_horizontal_cell(src[hrid], src[hrid+1], src[hrid+2], threshold);
            let hb_out = calc_horizontal_cell(src[hbid], src[hbid+1], src[hbid+2], threshold);

            if b_out*2.0+hr_out+hb_out != 0.0{
                out[b1] = out_color[0];
                out[b2] = out_color[1];
                out[b3] = out_color[2];
                if bytepp>3{
                    out[b3+1] = out_color[3];
                }
            }
        }
        i += bytepp;
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