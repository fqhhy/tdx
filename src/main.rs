use std::{
    io::{Read, Write,Result},
    net::TcpStream,
    time::Duration,
};

use tdx::parse::{parse_kline, parse_sort_hq};
use log::trace;

pub const PACK1: &[u8] = &[
    0x0c, 0x02, 0x18, 0x93, 0x00, 0x01, 0x03, 0x00, 0x03, 0x00, 0x0d, 0x00, 0x01,
];
pub const PACK2: &[u8] = &[
    0x0c, 0x02, 0x18, 0x94, 0x00, 0x01, 0x03, 0x00, 0x03, 0x00, 0x0d, 0x00, 0x02,
];
pub const PACK3: &[u8] = &[
    0x0c, 0x03, 0x18, 0x99, 0x00, 0x01, 0x20, 0x00, 0x20, 0x00, 0xdb, 0x0f, 0xd5, 0xd0, 0xc9, 0xcc,
    0xd6, 0xa4, 0xa8, 0xaf, 0x00, 0x00, 0x00, 0x8f, 0xc2, 0x25, 0x40, 0x13, 0x00, 0x00, 0xd5, 0x00,
    0xc9, 0xcc, 0xbd, 0xf0, 0xd7, 0xea, 0x00, 0x00, 0x00, 0x02,
];

const KLINE_PACK: &[u8] = &[
    0x0c, 0x01, 0x08, 0x64, 0x01, 0x01, 0x1c, 0x00, 0x1c, 0x00, 0x2d, 0x05, 0x00, 0x00, 0x30, 0x30,
    0x30, 0x30, 0x30, 0x31, 0x09, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

pub fn send_recv(tcp: &mut TcpStream, send: &[u8]) -> Result<Vec<u8>> {
    tcp.write_all(send)?;
    let mut head = [0u8; 16];
    let _head_size = tcp.read(&mut head)?;
    let deflate_size = u16::from_le_bytes([head[12], head[13]]); // 响应信息中的待解压长度
    let mut buf = vec![0; deflate_size as usize];
    tcp.read_exact(&mut buf)?;
    let inflate_size = u16::from_le_bytes([head[14], head[15]]); // 响应信息中的解压后长度
    if deflate_size != inflate_size {
        let mut decompressed = vec![0u8; inflate_size as usize];
        let (result, rc) = zlib_rs::decompress_slice(&mut decompressed, &buf, zlib_rs::InflateConfig::default());
        assert_eq!(rc, zlib_rs::ReturnCode::Ok);
        buf = result.to_vec();
        trace!("解压后数据：\n{:?}\n", buf);
    } else {
        trace!("无需解压: \n{:?}\n", buf);
    };
    
    Ok(buf)
}



fn main() -> std::io::Result<()> {
    // 初始化日志系统
    env_logger::init();
    // 115.238.56.198
    match TcpStream::connect(("82.156.174.84", 7709)) {
        Ok(mut socket) => {
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            socket
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();


            send_recv(&mut socket, &PACK1)?;
            send_recv(&mut socket, &PACK2)?;
            send_recv(&mut socket, &PACK3)?;

            println!("Connected to server");

            let mut arr = [0; KLINE_PACK.len()];
            arr.copy_from_slice(KLINE_PACK);

            let market: u16 = 0;
            let code: &str = "000001";
            let category: u16 = 9;
            let start: u16 = 0;
            let count: u16 = 3;
            arr[12..14].copy_from_slice(&market.to_le_bytes());
            arr[14..20].copy_from_slice(code.as_bytes());
            arr[20..22].copy_from_slice(&category.to_le_bytes());
            arr[24..26].copy_from_slice(&start.to_le_bytes());
            arr[26..28].copy_from_slice(&count.to_le_bytes());

            let res = send_recv(&mut socket, &arr)?;
            let klines = parse_kline(res, category);
            println!("{:?}",klines);


            // 历史tick 请求 1
            let arr = [
                0xc, 0x25, 0x8, 0x0, 0x1, 0x1, 0x12, 0x0, 0x12, 0x0, 0xc6, 0xf, 0xce, 0x25, 0x35,
                0x1, 0x1, 0x0, 0x36, 0x30, 0x30, 0x30, 0x30, 0x34, 0x0, 0x0, 0x8, 0x7,
            ];
            let res = send_recv(&mut socket, &arr)?;
            // 历史tick 请求 2
            let arr = [
                0xc, 0x25, 0x8, 0x1, 0x1, 0x1, 0x12, 0x0, 0x12, 0x0, 0xc6, 0xf, 0xce, 0x25, 0x35,
                0x1, 0x1, 0x0, 0x36, 0x30, 0x30, 0x30, 0x30, 0x34, 0x8, 0x7, 0x8, 0x7,
            ];
            //  let res = send_recv(&mut socket, &arr)?;

            // 板块股票排序，按涨幅排序，全部A股板块
            // let arr = [
            //     0xc, 0xfe, 0x4, 0xa, 0x0, 0x1, 0x14, 0x0, 0x14, 0x0, 0x4b, 0x5, 0x6, 0x0, 0xe, 0x0,
            //     0x21, 0x0, 0x21, 0x0, 0x1, 0x0, 0x5, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0,
            // ];
            // let res = send_recv(&mut socket, &arr)?;
            // parse_sort_hq(&res);
        }
        Err(_) => {}
    };
    Ok(())
}
