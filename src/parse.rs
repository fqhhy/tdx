use base::model::Bar;

pub fn parse_kline(v: Vec<u8>, category: u16) -> Vec<Bar> {
    let count = u16::from_le_bytes([v[0], v[1]]);
    let mut pos = 2;
    let mut base = 0;
    let mut data = Vec::new();
    for _ in 0..count {
        let time: i64 = datetime(&v[pos..pos + 4], category);
        pos += 4;
        let open = base + price(&v, &mut pos);
        let close = open + price(&v, &mut pos);
        let high = open + price(&v, &mut pos);
        let low = open + price(&v, &mut pos);
        let vol = f32::from_le_bytes([v[pos], v[pos + 1], v[pos + 2], v[pos + 3]]);
        let amount = f32::from_le_bytes([v[pos + 4], v[pos + 5], v[pos + 6], v[pos + 7]]);
        pos = pos + 8;

        base = close;
        let bar = Bar {
            time,
            open: open as f64 / 1000.,
            close: close as f64 / 1000.,
            high: high as f64 / 1000.,
            low: low as f64 / 1000.,
            volume: vol as f64,
            amount: amount as f64,
        };
        data.push(bar);
    }
    return data;
}

#[inline]
fn real_price(p: i32, base: i32) -> f64 {
    (p + base) as f64 / 1000.
}

/// 解析价格 (open / close / high / low)
///
/// 注意：
/// 1. 第二次之后计算的价格为浮动价格，基于第一次解析的实际价格而浮动；
/// 2. 返回的 pos 是不定长的。
fn price(arr: &[u8], pos: &mut usize) -> i32 {
    let mut shl = 6;
    let mut bit = arr[*pos] as i32;
    let mut res = bit & 0x3f;
    let sign = (bit & 0x40) == 0;

    while (bit & 0x80) != 0 {
        *pos += 1;
        bit = arr[*pos] as i32;
        res += (bit & 0x7f) << shl;
        shl += 7;
    }
    *pos += 1;

    if sign { res } else { -res }
}

fn vol_amount(ivol: i32) -> f64 {
    let logpoint = ivol >> 24;
    let hleax = (ivol >> 16) & 0xff;
    let lheax = (ivol >> 8) & 0xff;
    let lleax = ivol & 0xff;
    let dw_ecx = logpoint * 2 - 0x7f;
    let dw_edx = logpoint * 2 - 0x86;
    let dw_esi = logpoint * 2 - 0x8e;
    let dw_eax = logpoint * 2 - 0x96;

    let dbl_xmm6 = if dw_ecx < 0 {
        1.0 / 2.0f64.powi(-dw_ecx)
    } else {
        2.0f64.powi(dw_ecx)
    };

    let dbl_xmm4 = if hleax > 0x80 {
        2.0f64.powi(dw_edx) * 128.0 + (hleax & 0x7f) as f64 * 2.0f64.powi(dw_edx + 1)
    } else if dw_edx >= 0 {
        2.0f64.powi(dw_edx) * hleax as f64
    } else {
        (1.0 / 2.0f64.powi(dw_edx)) * hleax as f64
    };

    let (dbl_xmm3, dbl_xmm1) = if (hleax & 0x80) != 0 {
        (
            2.0f64.powi(dw_esi + 1) * lheax as f64,
            2.0f64.powi(dw_eax + 1) * lleax as f64,
        )
    } else {
        (
            2.0f64.powi(dw_esi) * lheax as f64,
            2.0f64.powi(dw_eax) * lleax as f64,
        )
    };

    // dbg!(dbl_xmm6, dbl_xmm4, dbl_xmm3, dbl_xmm1);
    dbl_xmm6 + dbl_xmm4 + dbl_xmm3 + dbl_xmm1
}

fn datetime(arr: &[u8], category: u16) -> i64 {
    if category < 4 || category == 7 || category == 8 {
        let day = u16::from_le_bytes([arr[0], arr[1]]);
        let minutes = u16::from_be_bytes([arr[2], arr[3]]);
        let year = ((day >> 11) + 2004) as i32;
        let month = (day % 2048 / 100) as u32;
        let day = (day % 2048 % 100) as u32;
        let hour = (minutes / 60) as u32;
        let minute = (minutes % 60) as u32;
        let time = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
        // utc 时间和北京时间差8
        time.and_hms_opt(hour - 8, minute, 0)
            .unwrap()
            .and_utc()
            .timestamp()
    } else {
        let day = u32::from_le_bytes([arr[0], arr[1], arr[2], arr[3]]);
        let year = (day / 10000) as i32;
        let month = (day % 10000 / 100) as u32;
        let day = (day % 100) as u32;

        let day = chrono::NaiveDate::from_ymd_opt(year, month, day).unwrap();
        day.and_hms_opt(7, 0, 0).unwrap().and_utc().timestamp()
    }
}

fn price_2(arr: &[u8], pos: usize) -> i32 {
    let mut pos = pos;
    let mut shl = 6;
    let mut bit = arr[pos] as i32;
    let mut res = bit & 0x3f;
    let sign = (bit & 0x40) == 0;

    while (bit & 0x80) != 0 {
        pos += 1;
        bit = arr[pos] as i32;
        res += (bit & 0x7f) << shl;
        shl += 7;
    }

    if sign { res } else { -res }
}

pub fn parse_sort_hq(v: &[u8]) {
    let mut pos = 2;
    println!("{:?}",v);
    
    let count = u16::from_le_bytes([v[pos], v[pos + 1]]);
    println!("记录数: {}\n", count);
    pos += 2;

    for i in 0..count {
        // 市场代码 (1字节)

        let market = v[pos];
        pos += 1;

        // 股票代码 (6字节)

        let code = String::from_utf8_lossy(&v[pos..pos + 6]);
        println!("{}", code);
        pos += 6;

        let flag =  [v[pos], v[pos+1]]; 

        // 115 ,13 
        pos += 2;
        let xianjia = price(v, &mut pos);
        print!("现价: {} ", xianjia as f64 / 100.0);
        let zhangdie = price(v, &mut pos);
        print!("涨跌: {} ", -zhangdie as f64 / 100.0);
        let open = xianjia + price(v, &mut pos);
        print!("今开: {} ", open as f64 / 100.0);
        let high = xianjia + price(v, &mut pos);
        print!("最高: {} ", high as f64 / 100.0);
        let low = xianjia + price(v, &mut pos);
        print!("最低: {} ,", low as f64 / 100.0);

        let a = pos;
        let val = price(v, &mut pos);
        print!("???: {} , {:?}", val, &v[a..pos]);

        let a = pos;
        print!("买价？？: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a = pos;
        print!("总量: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a: usize = pos;
        print!("现量: {} , {:?}", price(v, &mut pos), &v[a..pos]);

        print!(
            "总金额: {} , {:?}",
            f32::from_le_bytes([v[pos], v[pos + 1], v[pos + 2], v[pos + 3]]),
            &v[pos..pos + 4]
        );
        pos += 4;

        let a = pos;
        print!("内盘: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a: usize = pos;
        print!("外盘: {} , {:?}", price(v, &mut pos), &v[a..pos]);

        let a = pos;
        print!("未知: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a = pos;
        print!("开盘金额（百）: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a = pos;
        print!("未知: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a = pos;
        print!("差值: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a = pos;
        print!("买量: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        let a = pos;
        println!("卖量？？: {} , {:?}", price(v, &mut pos), &v[a..pos]);
        
        // 在 v中 查找和 flag一样值的位置，从 pos开始
        let end_pos = v[pos..].windows(2).position(|w| w[0] == flag[0] && w[1] == flag[1]).unwrap();
        pos = pos + end_pos + 2;
    }
}

fn test_parse(v: &[u8], pos: usize) {
    println!("-------------");
    println!(
        "next u16: {} , {:?}",
        u16::from_le_bytes([v[pos], v[pos + 1]]),
        &v[pos..pos + 2]
    );
    println!(
        "next i32: {} , {:?}",
        i32::from_le_bytes([v[pos], v[pos + 1], v[pos + 2], v[pos + 3]]),
        &v[pos..pos + 4]
    );
    println!(
        "next f32: {} , {:?}",
        f32::from_le_bytes([v[pos], v[pos + 1], v[pos + 2], v[pos + 3]]),
        &v[pos..pos + 4]
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hq_sort() {
        let v = [
            0, 0, 9, 0, //
            0, 51, 48, 48, 57, 56, 49, // 市场+代码
            115, 13, // 结束标识？？
            169, 24, // 1577 现价
            199, 4, // -263  涨跌
            226, 3, // -226,  1577-226=1351=open
            0, // 价差  1577-0=1577=high
            226, 3, // -263,  1577-226=1351=low
            128, 156, 207, 14, 0, // 卖价0
            142, 198, 53, // 总量 price解析 438670
            137, 7, // 现量 457
            108, 212, 31, 78, // 总金额
            133, 142, 21, // 内盘
            137, 184, 32, // 外盘
            0,  //
            183, 153, 4, //
            0, //
            233, 24, // -1577
            186, 151, //
            4, 0, 150, 9, 0, 0, 12, 0, 0, 28, 48, 73, 67,
             0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 
             116, 235, 35, 62,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 115, 13, //
            //
            0, 51, 48, 48, 49, 56, 53, 59, 17, 165, 6, 198, 1, 127, 0, 127, 132, 174, 207, 14, 188,
            85, 189, 221, 150, 8, 136, 240, 3, 52, 163, 76, 79, 190, 128, 194, 3, 128, 221, 212, 4,
            0, 169, 168, 17, 0, 229, 6, 143, 215, 72, 0, 86, 19, 0, 0, 19, 0, 0, 249, 75, 75, 113,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 94, 98, 153, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 59, 17, //
                //
        ];

        parse_sort_hq(&v);
    }

    #[test]
    fn test_parse_kline() {
        let arr = vec![
            0x03, 0x00, 0xeb, 0x64, 0x34, 0x01, 0xb4, 0x9a, 0x02, 0xe4, 0x06, 0x9c, 0x03, 0xc2,
            0x07, 0xe8, 0x6f, 0xa8, 0x49, 0x59, 0xf7, 0x12, 0x4f, 0xec, 0x64, 0x34, 0x01, 0xd0,
            0x01, 0xfa, 0x03, 0x90, 0x01, 0xc4, 0x04, 0x00, 0x81, 0x9a, 0x49, 0xb7, 0xb1, 0x03,
            0x4f, 0xef, 0x64, 0x34, 0x01, 0xcc, 0x02, 0xa8, 0x05, 0x96, 0x07, 0xd6, 0x02, 0xd8,
            0x3d, 0x8b, 0x49, 0x4b, 0xf0, 0xeb, 0x4e,
        ];

        let data = parse_kline(arr, 9);
        println!("{:?}", data);
    }
}
