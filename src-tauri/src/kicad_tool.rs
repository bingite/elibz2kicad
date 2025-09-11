//! kicad_tool - 用于处理KiCad文件格式的工具模块
//!
//! 该模块提供创建kicad_mod文件和修改kicad_sym文件的功能框架

use chrono::{DateTime, Local};
use uuid::Uuid;
use log::{info}; // 添加日志库引用

/// 用于暂存kicad_mod文件数据的结构体
#[derive(Debug, Clone)]
pub struct KicadModData {
    /// 文件头信息
    header: String,
    /// 版本
    version: String,
    /// 生成者
    generator: String,
    /// 所在层
    layer: String,
    /// 最后编辑时间
    tedit: String,
    /// 描述
    description: String,
    /// 标签
    tags: String,
    /// 元件类型
    attr: String,
    /// 文本元素
    text_elements: Vec<String>,
    /// 图形元素
    graphic_elements: Vec<String>,
    /// 焊盘信息
    pads: Vec<String>,
    // 3D模型数据
    // other_data: String,
}


impl KicadModData {
    /// 创建一个新的KicadModData实例
    ///
    /// # 参数
    ///
    /// * `module_name` - 模块名称
    ///
    /// # 返回值
    ///
    /// * `KicadModData` - 新创建的实例
    pub fn new(module_name: &str) -> Self {

        let now: DateTime<Local> = Local::now();
        // 获取 YYYYMMDD 格式
        //let date_str = now.format("%Y%m%d").to_string();
        // 获取 16 位十六进制时间戳（秒）
        let hex_tedit = format!("{:016x}", now.timestamp() as u64);
        // 生成UUIDv4，应该唯一，防止冲突
        // Uuid::new_v4()
        // 文件头
        let header = format!("(footprint \"{}\" ", 
                            module_name);
        // 文本属性
        let mut text_elements = Vec::new();
        // 参考标识
        text_elements.push(format!("  (fp_text reference \"REF**\" (at 0 -5) (layer \"F.SilkS\")\n    (effects (font (size 1 1) (thickness 0.15)))\n    (tstamp {}))", Uuid::new_v4()));
        // 值
        text_elements.push(format!("  (fp_text value \"{}\" (at 0 5) (layer \"F.Fab\")\n    (effects (font (size 1 1) (thickness 0.15)))\n    (tstamp {}))", module_name, Uuid::new_v4()));

        info!("创建新的KicadModData实例: {}", module_name); // 添加日志

        KicadModData {
            header,
            version: format!("(version {})", "20211014"),
            generator: format!("(generator {})", "pcbnew"),
            layer: format!("(layer \"{}\")", "F.Cu"),
            tedit: format!("(tedit {})", hex_tedit),
            description: format!("(descr \"{}\")", ""),  
            tags: format!("(tags \"{}\")", ""),
            attr: format!("(attr {})", "smd"),
            text_elements,
            graphic_elements: Vec::new(),
            pads: Vec::new(),
            // other_data: String::new(),
        }
    }

    
    /// 添加图形元素:直线
    ///
    /// # 参数
    ///
    /// * `element` - 图形元素字符串
    pub fn add_graphic_element_line(&mut self, start_x: f64, start_y: f64, end_x: f64, end_y: f64, layer: String, width: f64) {

        info!("添加直线元素: 从({:.6},{:.6})到({:.6},{:.6})，层{}，线宽{:.6}", mil_to_mm(start_x), mil_to_mm(start_y), mil_to_mm(end_x), mil_to_mm(end_y), layer, mil_to_mm(width)); // 添加日志
        self.graphic_elements.push(format!("  (fp_line (start {:.6} {:.6}) (end {:.6} {:.6}) (layer \"{}\") (width {:.6}) (tstamp {}))", 
            mil_to_mm(start_x), mil_to_mm(start_y), mil_to_mm(end_x), mil_to_mm(end_y), layer, mil_to_mm(width), Uuid::new_v4()));
    }

    
    /// 添加图形元素：圆
    /// 
    /// # 参数
    ///
    /// * `element` - 图形元素字符串
    /// 示例：(fp_circle (center x y) (end x2 y2) (layer "Layer") (width w) (fill none|solid) ...)
    pub fn add_graphic_element_circle(&mut self, center_x: f64, center_y: f64, end_x: f64, end_y: f64, layer: String, width: f64, fill: bool) {
        info!("添加圆形元素: 圆心({:.6},{:.6})，终点({:.6},{:.6})，层{}，线宽{:.6}，填充{}", mil_to_mm(center_x), mil_to_mm(center_y), mil_to_mm(end_x), mil_to_mm(end_y), layer, mil_to_mm(width), if fill { "solid" } else { "none" }); // 添加日志
        self.graphic_elements.push(format!("  (fp_circle (center {:.6} {:.6}) (end {:.6} {:.6}) (layer {}) (width {:.6}) (fill {}) (tstamp {}))", 
            mil_to_mm(center_x), - mil_to_mm(center_y), mil_to_mm(end_x), - mil_to_mm(end_y), layer, mil_to_mm(width), if fill { "solid" } else { "none" }, Uuid::new_v4()));
    }


    /// 添加图形元素:多边形
    /// 
    /// # 参数
    ///
    /// * `element` - 图形元素字符串
    /// 示例：(fp_poly (pts (xy -16.1036 4.953) (xy -16.8656 4.191) ...) (layer "F.SilkS") (width 0.12) (fill solid) (tstamp {}))
    pub fn add_graphic_element_polygon(&mut self, pts: &Vec<f64>, layer: String, width: f64, fill: bool) {
        // 如果多边形在F.SilkS层，需要使用add_graphic_element_line来将多边形分成一段段的直线
        if layer == "F.SilkS" && !fill {
            // 将多边形分解为线段
            for i in 0..((pts.len()/2)-1) {
                let start_x = pts[i*2];
                let start_y = pts[i*2+1];
                let end_x = pts[(i+1)*2];
                let end_y = pts[(i+1)*2+1];   
                self.add_graphic_element_line(start_x, -start_y, end_x, -end_y, layer.clone(), width);
            }
        } else {
            let mut str_pts = "  (fp_poly (pts ".to_string();
            // 获取pts长度
        
            for i in 0..pts.len()/2 {
                str_pts.push_str(&format!("   (xy {:.6} {:.6})", mil_to_mm(pts[i*2]), - mil_to_mm(pts[i*2+1])));
            }
            str_pts.push_str(&format!("  ) (layer \"{}\") (width {:.6}) (fill {}) (tstamp {}))", layer, mil_to_mm(width), if fill { "solid" } else { "none" }, Uuid::new_v4()));
            info!("添加多边形元素: {};", str_pts); 
            self.graphic_elements.push(str_pts);
        }
    }


    /// 添加图形元素：圆弧
    /// 
    /// # 参数
    ///
    /// * `element` - 图形元素字符串
    /// 示例：(fp_arc (start x1 y1) (mid x2 y2) (end x3 y3) (layer "LayerName") (width w) (tstamp ...))
    pub fn add_graphic_element_arc(&mut self, start_x: f64, start_y: f64, angle: f64, end_x: f64, end_y: f64, layer: String, width: f64) {
        let (start_x_3,start_y_3,mid_x, mid_y,end_x_3,end_y_3) = calculate_arc_midpoint(start_x, -start_y, end_x, -end_y, -angle);
        info!("添加圆弧元素: 起点({:.6},{:.6})，中点({:.6},{:.6})，终点({:.6},{:.6})，层{}，线宽{:.6}", mil_to_mm(start_x_3), mil_to_mm(start_y_3), mil_to_mm(mid_x), mil_to_mm(mid_y), mil_to_mm(end_x_3), mil_to_mm(end_y_3), layer, mil_to_mm(width)); // 添加日志
        self.graphic_elements.push(format!("  (fp_arc (start {:.6} {:.6}) (mid {:.6} {:.6}) (end {:.6} {:.6}) (layer {}) (width {:.6})(tstamp {}))", 
            mil_to_mm(start_x_3), mil_to_mm(start_y_3), mil_to_mm(mid_x), mil_to_mm(mid_y), mil_to_mm(end_x_3), mil_to_mm(end_y_3), layer, mil_to_mm(width), Uuid::new_v4()));    
        
    }

    /// 添加圆形挖槽
    /// 
    /// # 参数
    ///
    /// * `center_x` - 圆心的X坐标
    /// * `center_y` - 圆心的Y坐标
    /// * `radius` - 半径
    /// * `layer` - 层
    pub fn add_graphic_element_circle_hole(&mut self, center_x: f64, center_y: f64, radius: f64) {
        info!("添加圆形挖槽元素: 圆心({:.6},{:.6})，半径{:.6}，层{}", mil_to_mm(center_x), mil_to_mm(center_y), mil_to_mm(radius), "F&B.Cu *.Mask"); 
        self.graphic_elements.push(format!("  (pad \"\" np_thru_hole circle (at {:.6} {:.6}) (size {:.6} {:.6}) (drill {:.6}) (layers {}) (tstamp {}))", 
            mil_to_mm(center_x), - mil_to_mm(center_y), mil_to_mm(radius), mil_to_mm(radius), 2.0*mil_to_mm(radius), "F&B.Cu *.Mask", Uuid::new_v4()));  
    }

    /// 添加圆形贴片焊盘
    /// 
    /// # 参数
    /// * `pad_name` - 焊盘名称
    /// * `center_x` - 圆心的X坐标
    /// * `center_y` - 圆心的Y坐标
    /// * `radius` - 半径
    /// * `solder_mask_margin` - 阻焊距离
    /// * `solder_paste_margin` - 锡膏距离
    /// 
    pub fn add_pad_circle(&mut self, pad_name: &str, center_x: f64, center_y: f64, radius: f64, solder_mask_margin: f64, solder_paste_margin: f64) {
        info!("添加圆形贴片焊盘元素: 圆心({:.6},{:.6})，半径{:.6}，层{}，阻焊距离{}，锡膏距离{}", mil_to_mm(center_x), mil_to_mm(center_y), mil_to_mm(radius), "F.Cu", solder_mask_margin, solder_paste_margin); 
        self.pads.push(format!("  (pad {} smd circle (at {:.6} {:.6}) (size {:.6} {:.6}) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\") (solder_mask_margin {:.6}) (solder_paste_margin {:.6}) (tstamp {}))", 
            pad_name, mil_to_mm(center_x), - mil_to_mm(center_y), mil_to_mm(radius), mil_to_mm(radius), mil_to_mm(solder_mask_margin), mil_to_mm(solder_paste_margin), Uuid::new_v4()));  
    }

    /// 添加矩形贴片焊盘
    /// 
    /// # 参数
    /// * `pad_name` - 焊盘名称
    /// * `center_x` - 圆心的X坐标
    /// * `center_y` - 圆心的Y坐标
    /// * `angle` - 角度
    /// * `width` - 宽度
    /// * `height` - 高度
    /// * `solder_mask_margin` - 阻焊距离
    /// * `solder_paste_margin` - 锡膏距离
    pub fn add_pad_rect(&mut self, pad_name: &str, center_x: f64, center_y: f64, angle: f64, width: f64, height: f64, solder_mask_margin: f64, solder_paste_margin: f64) {
        info!("添加矩形贴片焊盘元素: 圆心({:.6},{:.6})，角度{}，宽度{:.6}，高度{:.6}，层{}，阻焊距离{}，锡膏距离{}", mil_to_mm(center_x), mil_to_mm(center_y), angle, mil_to_mm(width), mil_to_mm(height), "F.Cu", solder_mask_margin, solder_paste_margin); 
        self.pads.push(format!("  (pad {} smd rect (at {:.6} {:.6} {}) (size {:.6} {:.6}) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\") (solder_mask_margin {:.6}) (solder_paste_margin {:.6}) (tstamp {}))", 
            pad_name, mil_to_mm(center_x), - mil_to_mm(center_y), angle, mil_to_mm(width), mil_to_mm(height), mil_to_mm(solder_mask_margin), mil_to_mm(solder_paste_margin), Uuid::new_v4()));  
    }

    /// 添加椭圆形贴片焊盘
    /// 
    /// # 参数
    /// * `pad_name` - 焊盘名称
    /// * `center_x` - 圆心的X坐标
    /// * `center_y` - 圆心的Y坐标
    /// * `width` - 宽度
    /// * `height` - 高度
    /// * `angle` - 角度
    /// * `solder_mask_margin` - 阻焊距离
    /// * `solder_paste_margin` - 锡膏距离
    pub fn add_pad_ellipse(&mut self, pad_name: &str, center_x: f64, center_y: f64, width: f64, height: f64, angle: f64, solder_mask_margin: f64, solder_paste_margin: f64) {
        info!("添加椭圆形贴片焊盘元素: 圆心({:.6},{:.6})，宽度{:.6}，高度{:.6}，层{}，角度{}，阻焊距离{}，锡膏距离{}", mil_to_mm(center_x), mil_to_mm(center_y), mil_to_mm(width), mil_to_mm(height), "F.Cu", angle, solder_mask_margin, solder_paste_margin); 
        self.pads.push(format!("  (pad {} smd oval (at {:.6} {:.6} {}) (size {:.6} {:.6}) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\") (solder_mask_margin {:.6}) (solder_paste_margin {:.6}) (tstamp {}))", 
            pad_name, mil_to_mm(center_x), - mil_to_mm(center_y), angle, mil_to_mm(width), mil_to_mm(height), mil_to_mm(solder_mask_margin), mil_to_mm(solder_paste_margin), Uuid::new_v4()));  
    }

    /// 添加多边形焊盘
    /// 
    /// # 参数
    /// * `pad_name` - 焊盘名称
    /// * `center_x` - 中心的X坐标
    /// * `center_y` - 中心的Y坐标
    /// * `pts` - 点的坐标，格式为[x1, y1, x2, y2, ...]
    /// * `solder_mask_margin` - 阻焊距离
    /// * `solder_paste_margin` - 锡膏距离
    pub fn add_pad_poly(&mut self, pad_name: &str, center_x: f64, center_y: f64, pts: &[f64], solder_mask_margin: f64, solder_paste_margin: f64) {
        let mut str_pts = String::new();
        str_pts.push_str(&format!("  (pad {} smd custom (at {:.6} {:.6}) (size 0.0001 0.0001) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\")\n    (options (clearance outline) (anchor circle))\n    (primitives\n      (gr_poly (pts", 
            pad_name, mil_to_mm(center_x), - mil_to_mm(center_y)));
        
        let relative_pts = absolute_to_relative(pts, center_x, center_y);
        for i in 0..relative_pts.len() / 2 {
            str_pts.push_str(&format!(" (xy {:.6} {:.6})", mil_to_mm(relative_pts[i * 2]), mil_to_mm(relative_pts[i * 2 + 1])));
        }
        str_pts.push_str(&format!("\n      ) (width 0) (fill yes))\n  )(solder_mask_margin {:.6}) (solder_paste_margin {:.6})(tstamp {}))", mil_to_mm(solder_mask_margin), mil_to_mm(solder_paste_margin), Uuid::new_v4()));
        info!("添加多边形焊盘元素: 中心({:.6},{:.6})，点{}，层{}，阻焊距离{}，锡膏距离{}", mil_to_mm(center_x), mil_to_mm(center_y), str_pts, "F.Cu", solder_mask_margin, solder_paste_margin);
        self.pads.push(str_pts);
    }


    /// 添加通孔圆形焊盘
    /// 
    /// # 参数
    /// * `pad_name` - 焊盘名称
    /// * `center_x` - 圆心的X坐标
    /// * `center_y` - 圆心的Y坐标
    /// * `radius` - 半径
    /// * `drill` - 钻孔直径
    pub fn add_pad_hole(&mut self, pad_name: &str, center_x: f64, center_y: f64, radius: f64, drill:f64) {
        info!("添加通孔圆形焊盘元素: 圆心({:.6},{:.6})，半径{:.6}，层{}，钻孔直径{}", mil_to_mm(center_x), mil_to_mm(center_y), mil_to_mm(radius), "F.Cu", drill); 
        self.pads.push(format!("  (pad {} thru_hole circle (at {:.6} {:.6}) (size {:.6} {:.6}) (drill {:.6}) (layers *.Cu *.Mask) (solder_mask_margin 0.051) (tstamp {}))", 
            pad_name, mil_to_mm(center_x), - mil_to_mm(center_y), mil_to_mm(radius), mil_to_mm(radius), mil_to_mm(drill), Uuid::new_v4()));  
    }

    /// 添加通孔椭圆焊盘
    /// 
    /// # 参数
    /// * `pad_name` - 焊盘名称
    /// * `center_x` - 中心的X坐标
    /// * `center_y` - 中心的Y坐标
    /// * `angle` - 角度
    /// * `width` - 宽度
    /// * `height` - 高度
    /// * `drill_x` - 钻孔直径X
    /// * `drill_y` - 钻孔直径Y
    pub fn add_pad_hole_oval(&mut self, pad_name: &str, center_x: f64, center_y: f64, angle: f64, width: f64, height: f64,  drill_x:f64, drill_y:f64) {
        info!("添加通孔矩形焊盘元素: 中心({:.6},{:.6})，宽度{:.6}，高度{:.6}，层{}，角度{}，钻孔直径{}x{}", mil_to_mm(center_x), mil_to_mm(center_y), mil_to_mm(width), mil_to_mm(height), "F.Cu", angle, drill_x, drill_y); 
        self.pads.push(format!("  (pad {} thru_hole oval (at {:.6} {:.6} {}) (size {:.6} {:.6}) (drill oval {:.6} {:.6}) (layers *.Cu *.Mask) (solder_mask_margin 0.051) (tstamp {}))",  
            pad_name, mil_to_mm(center_x), - mil_to_mm(center_y), angle, mil_to_mm(width), mil_to_mm(height), mil_to_mm(drill_x), mil_to_mm(drill_y), Uuid::new_v4()));  
    }
    
    ///添加通孔矩形焊盘
    /// 
    /// # 参数
    /// * `pad_name` - 焊盘名称
    /// * `center_x` - 中心的X坐标
    /// * `center_y` - 中心的Y坐标
    /// * `angle` - 角度
    /// * `width` - 宽度
    /// * `height` - 高度
    /// * `drill` - 钻孔直径
    pub fn add_pad_hole_rect(&mut self, pad_name: &str, center_x: f64, center_y: f64, width: f64, height: f64, angle: f64, drill:f64) {
        info!("添加通孔矩形焊盘元素: 中心({:.6},{:.6})，宽度{:.6}，高度{:.6}，层{}，角度{}，钻孔直径{}", mil_to_mm(center_x), mil_to_mm(center_y), mil_to_mm(width), mil_to_mm(height), "F.Cu", angle, drill); 
        self.pads.push(format!("  (pad {} thru_hole rect (at {:.6} {:.6} {}) (size {:.6} {:.6}) (drill {:.6}) (layers *.Cu *.Mask) (solder_mask_margin 0.051) (tstamp {}))", 
            pad_name, mil_to_mm(center_x), - mil_to_mm(center_y), angle, mil_to_mm(width), mil_to_mm(height), mil_to_mm(drill), Uuid::new_v4()));  
    }
    

    /// 生成完整的kicad_mod文件内容
    ///
    /// # 返回值
    ///
    /// * `String` - 完整的kicad_mod文件内容
    pub fn generate_content(&self) -> String {
        let mut content = String::new();
        
        // 添加文件头
        content.push_str(&self.header);
        content.push('\n');
        // 添加版本
        content.push_str(&self.version);
        content.push('\n');
        // 添加生成者
        content.push_str(&self.generator);
        content.push('\n');
        // 添加所在层
        content.push_str(&self.layer);
        content.push('\n');
        // 添加最后编辑时间
        content.push_str(&self.tedit);
        content.push('\n');
        // 添加描述
        content.push_str(&self.description);
        content.push('\n');
        // 添加标签
        content.push_str(&self.tags);
        content.push('\n');
        // 添加元件类型
        content.push_str(&self.attr);
        content.push('\n');
        // 添加文本元素
        for element in &self.text_elements {
            content.push_str(element);
            content.push('\n');
        }
        
        // 添加图形元素
        for element in &self.graphic_elements {
            content.push_str(element);
            content.push('\n');
        }
        
        // 添加焊盘信息
        for pad in &self.pads {
            content.push_str(pad);
            content.push('\n');
        }
        
        
        // 添加文件结尾
        content.push_str(")");
        
        info!("生成kicad_mod文件内容，共{}个文本元素，{}个图形元素，{}个焊盘", self.text_elements.len(), self.graphic_elements.len(), self.pads.len()); // 添加日志
        
        content
    }
}

// mil转mm的函数
fn mil_to_mm(mil: f64) -> f64 {
    mil * 0.0254
}


/// 输入圆弧的起点、终点、角度计算出圆弧的第三点
/// 
/// # 参数
///
/// * `x1` - 圆弧起点的X坐标
/// * `y1` - 圆弧起点的Y坐标
/// * `x2` - 圆弧终点的X坐标
/// * `y2` - 圆弧终点的Y坐标
/// * `angle` - 圆弧的角度（角度）
/// 
/// # 返回值
///
fn calculate_arc_midpoint(x1: f64, y1: f64, x2: f64, y2: f64, angle: f64) -> (f64, f64, f64, f64, f64, f64) {
    //将angle转换为弧度
    let angle = angle.to_radians();
    // 接近零角度或同一点
    if angle.abs() < 1e-10 || (x1 - x2).hypot(y1 - y2) < 1e-10 {
        return (x1, y1, x1, y1, x2, y2);
    }

    let mut x1 = x1;
    let mut y1 = y1;
    let mut x2 = x2;
    let mut y2 = y2;
    let abs_angle = angle.abs();

    // 如果角度为负，说明用户想要顺时针，我们转换成等效的逆时针：交换起点终点
    if angle < 0.0 {
        std::mem::swap(&mut x1, &mut x2);
        std::mem::swap(&mut y1, &mut y2);
    }

    // 现在统一按逆时针处理
    let dx = x2 - x1;
    let dy = y2 - y1;
    let chord_len = dx.hypot(dy);

    if chord_len < 1e-10 {
        return (x1, y1, x1, y1, x2, y2);
    }

    let half_angle = abs_angle * 0.5;

    // sin(half_angle) 必须合理
    let sin_half = half_angle.sin();
    if sin_half.abs() < 1e-10 {
        return (x1, y1, x1, y1, x2, y2);
    }

    // 半径 R = chord_len / (2 * sin(half_angle))
    let radius = chord_len / (2.0 * sin_half);

    // 圆心到弦中点的距离：d = R * cos(half_angle)
    let dist_to_center = radius * half_angle.cos();

    // 弦中点
    let mx = (x1 + x2) * 0.5;
    let my = (y1 + y2) * 0.5;

    // 单位向量：从起点到终点
    let ex = dx / chord_len;
    let ey = dy / chord_len;

    // 法向量：逆时针旋转90度 => (-ey, ex)
    let nx = -ey;
    let ny = ex;

    // 圆心 = 弦中点 + 法向量 * dist_to_center
    let cx = mx + nx * dist_to_center;
    let cy = my + ny * dist_to_center;

    // 向量从圆心指向起点
    let start_vec_x = x1 - cx;
    let start_vec_y = y1 - cy;
    let start_angle = start_vec_y.atan2(start_vec_x);

    // 中点对应的角度：起始角 + 半个角度（逆时针）
    let mid_angle = start_angle + half_angle;

    // 计算中点
    let mid_x = cx + radius * mid_angle.cos();
    let mid_y = cy + radius * mid_angle.sin();

    (x1, y1, mid_x, mid_y, x2, y2)
}

/// 点的绝对坐标变为相对坐标
/// 
/// # 参数
/// * `pts` - 点的绝对坐标，格式为[x1, y1, x2, y2, ...]
/// * `center_x` - 中心的X坐标
/// * `center_y` - 中心的Y坐标
/// 
/// # 返回值
/// * `Vec<f64>` - 相对坐标的(x1, y1, x2, y2, ...)格式
pub fn absolute_to_relative(pts: &[f64], center_x: f64, center_y: f64) -> Vec<f64> {
    let mut relative_pts = Vec::new();
    for i in 0..pts.len() / 2 {
        let x = pts[i * 2] - center_x;
        let y = -(pts[i * 2 + 1] - center_y); // 修正Y坐标符号
        relative_pts.push(x);
        relative_pts.push(y);
    }
    relative_pts
}

pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// 创建一个新的点
    pub fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    /// 计算两点之间的距离
    fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// 向量加法
    fn add(&self, other: &Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }

    /// 向量减法
    fn sub(&self, other: &Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }

    /// 向量数乘
    fn mul(&self, scalar: f64) -> Point {
        Point::new(self.x * scalar, self.y * scalar)
    }

    /// 向量旋转90度 (逆时针)
    fn perp(&self) -> Point {
        Point::new(-self.y, self.x)
    }

    /// 向量单位化
    fn normalize(&self) -> Point {
        let len = (self.x * self.x + self.y * self.y).sqrt();
        if len > 1e-10 { // 避免除以零
            Point::new(self.x / len, self.y / len)
        } else {
            Point::new(0.0, 0.0)
        }
    }
}

/// 使用线段拟合圆弧
///
/// # 参数
/// * `start`: 圆弧的起点
/// * `end`: 圆弧的终点
/// * `angle_radians`: 圆弧的圆心角（弧度）。正值表示逆时针方向，负值表示顺时针方向。
/// * `num_segments`: 用于拟合圆弧的线段数量（即平滑度）。至少为1。
///
/// # 返回值
/// 返回一个包含拟合点的 `Vec<Point>`。向量的长度为 `num_segments + 1`。
/// 第一个点是 `start`，最后一个点是 `end`。
///
/// # 注意事项
/// * 如果 `start` 和 `end` 非常接近，则视为点，返回包含该点的向量。
/// * 如果 `angle_radians` 非常接近 0，则视为直线，返回起点和终点。
/// * 如果 `num_segments` 小于 1，则按 1 处理。
pub fn fit_arc_with_lines(
    start: Point,
    end: Point,
    angle_radians: f64,
    num_segments: usize,
) -> Vec<Point> {
    let mut points = Vec::with_capacity(num_segments + 1);

    // 处理特殊情况
    let chord_vec = end.sub(&start);
    let chord_length = chord_vec.distance(&Point::new(0.0, 0.0));

    if chord_length < 1e-10 || angle_radians.abs() < 1e-10 {
        // 如果点重合或角度为0，返回直线
        points.push(start);
        points.push(end);
        return points;
    }

    if num_segments < 1 {
         points.push(start);
         points.push(end);
         return points;
    }

    // 计算半径
    // R = (chord_length / 2) / sin(angle / 2)
    let half_angle = angle_radians.abs() / 2.0;
    let sin_half_angle = half_angle.sin();
    if sin_half_angle < 1e-10 {
        // 角度接近180度的倍数，处理为直线或半圆
         points.push(start);
         points.push(end);
         return points;
    }
    let radius = (chord_length / 2.0) / sin_half_angle;

    // 计算弦的中点
    let mid_point = start.add(&end).mul(0.5);

    // 计算弦的垂直平分线方向向量 (单位向量)
    // 垂直向量可以通过旋转90度得到
    let perp_chord_unit = chord_vec.perp().normalize();

    // 计算圆心到弦中点的距离
    // d = R * cos(angle / 2)
    let center_dist = radius * half_angle.cos();

    // 确定圆心位置
    // 圆心在垂直平分线上，距离中点为 center_dist
    // 方向由 angle 的正负决定（逆时针/顺时针）
    let center_offset = if angle_radians > 0.0 {
        perp_chord_unit.mul(center_dist)
    } else {
        perp_chord_unit.mul(-center_dist)
    };
    let center = mid_point.add(&center_offset);

    // 计算起点和终点相对于圆心的角度
    let start_angle = (start.y - center.y).atan2(start.x - center.x);
    // let end_angle = (end.y - center.y).atan2(end.x - center.x);
    // 我们不直接使用 end_angle，而是通过 start_angle 和 angle_radians 计算

    // 生成拟合点
    points.push(start); // 添加起点

    let angle_step = angle_radians / (num_segments as f64);
    for i in 1..num_segments {
        let current_angle = start_angle + (i as f64) * angle_step;
        let x = center.x + radius * current_angle.cos();
        let y = center.y + radius * current_angle.sin();
        points.push(Point::new(x, y));
    }

    points.push(end); // 添加终点

    points
}

