
use std::fs::File;
use std::io::{BufReader, Read};
use zip::ZipArchive;
use log::{info, error, warn}; // 添加日志库引用
use serde_json::{Value}; // 添加JSON处理库引用
use crate::kicad_tool::KicadModData;
use crate::kicad_tool::fit_arc_with_lines;
use crate::kicad_tool::Point;
use std::collections::HashMap;


// 层映射   
// 1 顶层 -> F.Cu
// 3 顶层丝印层 -> F.Silkscreen
// 5 顶层阻焊层 -> F.Mask
// 7 顶层锡膏层 -> F.Paste
// 9 顶层装配层 -> F.Fab

// 2 底层 -> B.Cu
// 4 底层丝印层 -> B.Silkscreen
// 6 底层阻焊层 -> B.Mask
// 8 底层锡膏层 -> B.Paste
// 10 底层装配层 -> B.Fab

// 11 板框层 -> Edge.Cuts
// 12 多层 -> F&B.Cu *.Mask
// 13 文档层 -> User.Drawings
// 14 机械层 -> null
// 56 钻孔图层 -> null
// 57 飞线层 -> null

// 48 元件外形层 -> F.Fab
// 49 元件标识层 -> User.7
// 50 引脚焊接层 -> User.8
// 51 引脚悬空层 -> User.9
const LAYER_MAP: &[(u64, &str)] = &[
    (1, "F.Cu"),
    (2, "B.Cu"),
    (3, "F.SilkS"),
    (4, "B.SilkS"),
    (5, "F.Mask"),
    (6, "B.Mask"),
    (7, "F.Paste"),
    (8, "B.Paste"),
    (9, "F.Fab"),
    (10, "B.Fab"),
    (11, "Edge.Cuts"),
    (12, "F&B.Cu *.Mask"),
    (13, "Dwgs.User"),
    (48, "F.Fab"),
    (49, "User.7"),
    (50, "User.8"),
    (51, "User.9"),
];




/// 对.elib文件进行处理函数
///
/// # 参数
///
/// * `elib_path` - .elib文件路径
/// * `kicad_mod_path` - kicad_mod文件目录路径
/// * `kicad_sym_path` - kicad_sym文件路径
///
/// # 返回值
///
/// * `()` - 无
#[tauri::command]
pub fn process_elib_file(elibz_file: String, output_dir: String, kicad_sym_file: String) -> String {
    let file_path = elibz_file.as_str();
    let kicad_mod_path = output_dir.as_str();
    let kicad_sym_path = kicad_sym_file.as_str();


    info!("开始处理文件: {}", file_path);
    // file_path去掉扩展名
    let file_name = file_path.split(".elibz").next().unwrap();
    // 检查文件是否存在
    if !std::path::Path::new(file_path).exists() {
        return "文件不存在".to_string();
    }
    
    // 判断是否为.elibz文件
    if !file_path.ends_with(".elibz") {
        return "文件不是.elibz格式".to_string();
    }

    // 判断压缩包里文件数量
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return "无法打开文件".to_string(),
    };
    let reader = BufReader::new(file);
    let mut archive = match ZipArchive::new(reader) {
        Ok(archive) => archive,
        Err(_) => return "无法打开压缩包".to_string(),
    };
    if archive.len()<3 {
        return "压缩包文件数量不足".to_string();
    }

    // 确定压缩包里每个文件的扩展名，并记录.json、.efoo、.esym文件的次序
    let mut json_index = 10;
    let mut efoo_index = 10;
    let mut esym_index = 10;
    for i in 0..archive.len() {
        let file = match archive.by_index(i) {
            Ok(file) => file,
            Err(_) => return "无法读取压缩包文件".to_string(),
        };  
        if file.name().ends_with(".json") {
            json_index = i;
        } else if file.name().ends_with(".efoo") {
            efoo_index = i;
        } else if file.name().ends_with(".esym") {
            esym_index = i;
        }
    }

    // 检查是否找到所有文件
    if json_index == 10 || efoo_index == 10 || esym_index == 10 {
        return "压缩包中缺少必要的.json .efoo .esym文件".to_string();
    }

    // 声明变量以存储符号和封装的标题，使其在main函数的其他地方也能访问
    let mut symbol_title_str = file_name.to_string();
    let mut footprint_title_str = file_name.to_string();
    
    // 读取json文件内容
    let json_content = {
        let mut json_file = match archive.by_index(json_index) {
            Ok(file) => file,
            Err(_) => return "无法读取压缩包文件".to_string(),
        };
        let mut content = String::new();
        match json_file.read_to_string(&mut content) {
            Ok(_) => content,
            Err(_) => return "无法读取压缩包文件".to_string(),
        }
    };
    
    // 解析JSON值
    let json_value: Value = match serde_json::from_str(&json_content) {
        Ok(value) => value,
        Err(_) => return "无法解析JSON文件".to_string(),
    };
    
    // 获取JSON里"symbols"的第一个元素里的"title"的值
    if let Some(symbols) = json_value["symbols"].as_object() {
        if let Some(first_symbol) = symbols.values().next() {
            if let Some(title) = first_symbol["display_title"].as_str() {
                symbol_title_str = title.to_string();
            }
        }
    }
    
    // 获取JSON里"footprints"的第一个元素里的"title"的值
    if let Some(footprints) = json_value["footprints"].as_object() {
        if let Some(first_footprint) = footprints.values().next() {
            if let Some(title) = first_footprint["display_title"].as_str() {
                footprint_title_str = title.to_string();
            }
        }
    }

    
    let parse_esym_file_result = if kicad_sym_path!="" {
        let mut esym_file = match archive.by_index(esym_index) {
            Ok(file) => file,
            Err(_) => return "无法读取压缩包文件".to_string(),
        };
        // 读取文件内容
        let mut contents = String::new();
        match esym_file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(_) => return "无法读取压缩包文件".to_string(),
        }
        
        parse_esym_file(&contents, &kicad_mod_path, &symbol_title_str)
    } else {
        "跳过符号文件解析".to_string()
    };

    let parse_efoo_file_result = if kicad_mod_path!="" {
        let mut efoo_file = match archive.by_index(efoo_index) {
            Ok(file) => file,
            Err(_) => return "无法读取压缩包文件".to_string(),
        };
        // 读取文件内容
        let mut contents = String::new();
        match efoo_file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(_) => return "无法读取压缩包文件".to_string(),
        }
        
        parse_efoo_file(&contents, &kicad_mod_path, &footprint_title_str)
    } else {
        "跳过封装文件解析".to_string()
    };

    return format!("成功解析\n{}\n{}", parse_esym_file_result, parse_efoo_file_result);
}


// 定义一个esym文件内容解析函数
fn parse_esym_file(esym_content: &str, kicad_mod_path: &str, symbol_title_str: &str) -> String {
    "OK".to_string()
}

// 定义一个efoo文件内容解析函数
fn parse_efoo_file(efoo_content: &str, kicad_mod_path: &str, footprint_title_str: &str) -> String {
    let layer_map: HashMap<u64, &str> = LAYER_MAP.iter().cloned().collect();
    let mut kicad_mod_data = KicadModData::new(footprint_title_str);
    // 未能完整翻译的行数
    let mut failed_count = 0;
    for line in efoo_content.lines() {
        info!("当前行: {}", line);

        // 跳过空行
        if line.trim().is_empty() {
            info!("空行");
            continue;
        }
        
        // 解析JSON
        let json_value: Value = match serde_json::from_str(line) {    
            Ok(value) => value,
            Err(e) => {
                info!("无法解析JSON: {}", e);
                continue;
            }
        };
        
        // 检查是否为数组
        let array = match json_value.as_array() {
            Some(arr) => arr,
            None => {
                info!("无法解析成json数组");
                continue;
            }
        };
        
        // 检查数组长度
        if array.len() < 6 {
            info!("json数组长度不足");
            continue;
        }
        
        
        let type_name = array[0].as_str().unwrap().to_string();
        // 图形类
        if type_name == "FILL" ||  type_name == "POLY"{
            // 图形是否填充
            let fill_bool = type_name == "FILL";
            if fill_bool{
                info!("填充类图形");
            }else{
                info!("非填充类图形");
            }
            // 所在层
            let key = array[4].as_u64().unwrap();
            let layer: String = layer_map.get(&key).unwrap().to_string();
            info!("所在层: {}", layer);
            // 线宽
            let line_width: f64 = array[5].as_f64().unwrap();
            // 获取图形数组
            let shape_array = if let Some(inner_array) = array.get(if fill_bool{7}else{6})
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|v| v.as_array())
            {
                if array.get(if fill_bool{7}else{6}).unwrap().as_array().unwrap().len() > 1{
                    failed_count+=1;
                    info!("填充类图形，数组长度大于1，未完全解析");
                }
                inner_array
            } else {
                array[if fill_bool{7}else{6}].as_array().unwrap()
            };

            // 是否是圆形
            let shape_type = if let Some(s) = shape_array[0].as_str() {
                s.to_string()
            } else {
                // 如果既不是字符串也不是数字，则跳过
                "".to_string()
            };
            if shape_type == "CIRCLE" {
                // 中心点
                let center: (f64, f64) = (shape_array[1].as_f64().unwrap(), shape_array[2].as_f64().unwrap());
                if layer=="F&B.Cu *.Mask" {
                    // 2.1.1 圆挖槽
                    info!("圆挖槽");
                    kicad_mod_data.add_graphic_element_circle_hole(center.0, center.1, shape_array[3].as_f64().unwrap());
                } else {
                    // 2.1.2 圆
                    // 结束点
                    info!("圆形");
                    let end: (f64, f64) = (center.0, center.1+shape_array[3].as_f64().unwrap());
                    kicad_mod_data.add_graphic_element_circle(center.0, center.1,end.0,end.1 , layer, line_width, fill_bool);
                }
                if shape_array.len() > 4 {
                    info!("圆形未完全解析");
                    failed_count+=1;
                }
                continue;
            }
            
            // 其它图形 
            // 判断shape_array里面"L"、"ARC"和"CARC"的总数，不要分别统计，shape_array里面有数字等其它类型，要注意
            let shape_type_count = shape_array.iter()
                .filter(|x| {
                    matches!(x.as_str(), Some("L") | Some("ARC") | Some("CARC"))
                })
                .count();
            if shape_type_count < 1 {
                info!("非多边形、圆弧图形，未完全解析");
                failed_count+=1;
            }
            else if shape_type_count == 1 {
                info!("多边形或圆弧图形");
                let shape_type_2 = shape_array[2].as_str().unwrap().to_string();
                if shape_type_2 == "L" {
                    info!("填充图形:多边形");
                    let pts: Vec<f64> = shape_array.iter()
                        .enumerate()
                        .filter_map(|(i, v)| if i != 2 { v.as_f64() } else { None })
                        .collect();
                    kicad_mod_data.add_graphic_element_polygon(&pts, layer, line_width, fill_bool);
                    continue;
                }
                else if shape_type_2 == "ARC" || shape_type_2 == "CARC" {
                    //[-59.055,0,"ARC",-180.47154258588805,59.056,0,"ARC",-180.47154258588805,-59.055,0]
                    info!("非填充图形:圆弧");
                    // 起点
                    let start: (f64, f64) = (shape_array[0].as_f64().unwrap(), shape_array[1].as_f64().unwrap());
                    info!("起点: {:?}", start);
                    // 角度
                    let angle: f64 = shape_array[3].as_f64().unwrap();
                    info!("角度: {}", angle);
                    // 结束点
                    let end: (f64, f64) = (shape_array[4].as_f64().unwrap(),shape_array[5].as_f64().unwrap());
                    info!("结束点: {:?}", end);
                    kicad_mod_data.add_graphic_element_arc(start.0, start.1, angle, end.0,end.1 , layer, line_width);
                    continue;
                }
            }
            else {
                info!("多边形、圆弧组合图形");
                
                let mut pts: Vec<f64> = Vec::new();
                let mut i = 2; // 手动控制索引
                while i < shape_array.len() {
                    if let Some(s) = shape_array[i].as_str() {
                        if s == "L" {
                            info!("组合中：多边形");
                            let mut polygon_pts: Vec<f64> = Vec::new();
                            let mut j = i + 1; // 从下一个元素开始读取 f64

                            // 连续读取 f64，直到遇到非 f64
                            while j < shape_array.len() {
                                if let Some(f) = shape_array[j].as_f64() {
                                    polygon_pts.push(f);
                                    j += 1;
                                } else {
                                    break;
                                }
                            }

                            pts.append(&mut polygon_pts);
                            //跳过已处理的 f64 数据：直接把 i 设置为 j
                            i = j; 
                        } else if s == "ARC" || s == "CARC" {
                            info!("组合中：圆弧");
                            // 起点
                            let start: Point = Point::new(shape_array[i-2].as_f64().unwrap(), -shape_array[i-1].as_f64().unwrap());
                            // 角度
                            let angle: f64 = -shape_array[i+1].as_f64().unwrap().to_radians();
                            // 结束点
                            let end: Point = Point::new(shape_array[i+2].as_f64().unwrap(),-shape_array[i+3].as_f64().unwrap());
                            let arc_pts = fit_arc_with_lines(start, end, angle, 20);
                            let mut polygon_pts: Vec<f64> = Vec::new();
                            for pt in arc_pts {
                                polygon_pts.push(pt.x);
                                polygon_pts.push(pt.y);
                            }
                            pts.append(&mut polygon_pts);
                            i += 3; // 跳过圆弧的3个参数
                        } else {
                            // 其他字符串类型，正常前进
                            info!("其他字符串类型: {},未能完全解析", s);
                            failed_count+=1;
                            i += 1;
                        }

                    } else {
                        // 非字符串元素，正常前进（或根据需要处理）
                        i += 1;
                    }
                }
                kicad_mod_data.add_graphic_element_polygon(&pts, layer, line_width, fill_bool);
            }
        }
        else if type_name == "PAD" {
            info!("焊盘");
            // 获取焊盘数据
            if array.len() < 11 {
                warn!("焊盘数据长度不足,未完全解析");
                failed_count+=1;
                continue;
            }
            
            // 焊盘名称
            let pad_name = array[5].as_str().unwrap_or("");
            // 中心坐标
            let center_x = array[6].as_f64().unwrap_or(0.0);
            let center_y = array[7].as_f64().unwrap_or(0.0);
            // 旋转角度
            let angle = array[8].as_f64().unwrap_or(0.0);

            // 焊盘形状描述数组
            let pad_shape = if let Some(shape_array) = array[10].as_array() {
                shape_array
            } else {
                warn!("焊盘形状描述数组为空,未完全解析");
                failed_count+=1;
                continue;
            };
            // array[9]可能是数组也可能是null，需要判断
            let drill_shape = if let Some(shape_array) = array[9].as_array() {
                shape_array
            } else {
                warn!("钻孔形状描述数组为空");
                &Vec::new()
            };

            if drill_shape.len() > 0 {
                // 通孔
                info!("通孔焊盘");
                // 判断是否为圆
                if drill_shape[0] == "ROUND" {
                    let drill_radius = drill_shape[1].as_f64().unwrap_or(0.0);
                    if pad_shape[0] == "ELLIPSE" {
                        // 圆
                        let pad_radius = pad_shape[1].as_f64().unwrap_or(0.0);
                        kicad_mod_data.add_pad_hole(pad_name, center_x, center_y, pad_radius, drill_radius);
                    }
                    else if pad_shape[0] == "RECT" {
                        // 矩形
                        let pad_width = pad_shape[1].as_f64().unwrap_or(0.0);
                        let pad_height = pad_shape[2].as_f64().unwrap_or(0.0);
                        kicad_mod_data.add_pad_hole_rect(pad_name, center_x, center_y, pad_width, pad_height, angle, drill_radius);
                    }
                    else if pad_shape[0] == "OVAL" {
                        // 椭圆
                        let pad_radius_x = pad_shape[1].as_f64().unwrap_or(0.0);
                        let pad_radius_y = pad_shape[2].as_f64().unwrap_or(0.0);
                        kicad_mod_data.add_pad_hole_oval(pad_name, center_x, center_y, angle, pad_radius_x, pad_radius_y,  drill_radius, drill_radius);
                    }
                    else {
                        warn!("未知的焊盘形状");
                    }
                }
                // 判断是否为槽
                else if drill_shape[0] == "SLOT" {
                    let mut drill_width = 0.0;
                    let mut drill_height = 0.0;
                    //如果为90度的奇数倍则交换drill_width和drill_height
                    if (array[14].as_u64().unwrap_or(0)/90) % 2 == 1 {
                        // 水平
                        drill_width = drill_shape[2].as_f64().unwrap_or(0.0);
                        drill_height = drill_shape[1].as_f64().unwrap_or(0.0);
                    }
                    else {
                        // 垂直
                        drill_width = drill_shape[1].as_f64().unwrap_or(0.0);
                        drill_height = drill_shape[2].as_f64().unwrap_or(0.0);
                    }
                    if pad_shape[0] == "ELLIPSE" {
                        // 圆形
                        info!("圆通孔的圆形焊盘");
                    }
                    else if pad_shape[0] == "RECT" {
                        // 矩形
                        info!("矩形通孔的矩形焊盘");
                    }
                    else if pad_shape[0] == "OVAL" {
                        let pad_radius_x = pad_shape[1].as_f64().unwrap_or(0.0);
                        let pad_radius_y = pad_shape[2].as_f64().unwrap_or(0.0);
                        kicad_mod_data.add_pad_hole_oval(pad_name, center_x, center_y, angle, pad_radius_x, pad_radius_y,  drill_width, drill_height);
                    }
                    else {
                        warn!("未知的焊盘形状,未完全解析");
                        failed_count+=1;
                    }
                }
                else{
                    warn!("未知的钻孔形状,未完全解析");
                    failed_count+=1;
                }
            }
            else {
                // 贴片
                info!("贴片焊盘");
                // 阻焊扩展
                let solder_mask_margin = array[18].as_f64().unwrap_or(2.0);
                // 锡膏扩展
                let solder_paste_margin = array[20].as_f64().unwrap_or(0.0);
                if pad_shape[0] == "ELLIPSE" {
                    info!("圆形焊盘");
                    //圆形
                    let pad_radius = pad_shape[1].as_f64().unwrap_or(0.0);
                    kicad_mod_data.add_pad_circle(pad_name, center_x, center_y, pad_radius, solder_mask_margin, solder_paste_margin);
                }
                else if pad_shape[0] == "RECT" {
                    //矩形
                    info!("矩形焊盘");
                    let pad_width = pad_shape[1].as_f64().unwrap_or(0.0);
                    let pad_height = pad_shape[2].as_f64().unwrap_or(0.0);
                    kicad_mod_data.add_pad_rect(pad_name, center_x, center_y, angle, pad_width, pad_height, solder_mask_margin, solder_paste_margin);
                }
                else if pad_shape[0] == "OVAL" {
                    //椭圆
                    info!("椭圆焊盘");
                    let (pad_radius_x, pad_radius_y) = {
                        let w = pad_shape[1].as_f64().unwrap();
                        let h = pad_shape[2].as_f64().unwrap();
                        if ((array[14].as_u64().unwrap_or(0) / 90) % 2) == 1 {
                            (w, h)  // 90度奇数倍：水平
                        } else {
                            (h, w)  // 否则：垂直
                        }
                    };
                    kicad_mod_data.add_pad_ellipse(pad_name, center_x, center_y, pad_radius_x, pad_radius_y, angle, solder_mask_margin, solder_paste_margin);
                }
                else if pad_shape[0] == "POLY" {
                    //多边形
                    info!("多边形焊盘");
                    let pad_points = pad_shape[1].as_array().unwrap();
                    let pts: Vec<f64> = pad_points.iter()
                    .enumerate()
                    .filter_map(|(i, v)| if i != 2 { v.as_f64() } else { None })
                    .collect();
                    kicad_mod_data.add_pad_poly(pad_name, center_x, center_y, &pts, solder_mask_margin, solder_paste_margin);
                }

                else {
                    warn!("未知的焊盘形状,未完全解析");
                    failed_count+=1;
                }
                
            }
        }
        else {
            info!("其它类型: {}", type_name);
        }
    }

    // 写入文件,路径kicad_mod_path，
    let content = kicad_mod_data.generate_content();
    match std::fs::write(format!("{}\\{}.kicad_mod", kicad_mod_path, footprint_title_str), content) {
        Ok(_) => format!("{}解析成功，未完全解析行数:{}", footprint_title_str,failed_count),
        Err(e) => {
            error!("写入文件失败: {:?}", e);
            "写入文件失败".to_string()
        }
    }
}




