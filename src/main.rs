use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};

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

fn main() -> std::io::Result<()> {
    match TcpStream::connect(("115.238.56.198", 7709)) {
        Ok(mut socket) => {
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            socket
                .set_write_timeout(Some(Duration::from_secs(5)))
                .unwrap();

            socket.write_all(&PACK1)?;
            let mut head = [0u8; 16];
            let head_size = socket.read(&mut head)?;
            let deflate_size = u16::from_le_bytes([head[12], head[13]]); // 响应信息中的待解压长度
            let mut buf = vec![0; deflate_size as usize];
            socket.read_exact(&mut buf)?;
            let inflate_size = u16::from_le_bytes([head[14], head[15]]); // 响应信息中的解压后长度


            socket.write_all(&PACK2)?;
            let mut head = [0u8; 16];
            let head_size = socket.read(&mut head)?;
            let deflate_size = u16::from_le_bytes([head[12], head[13]]); // 响应信息中的待解压长度
            let mut buf = vec![0; deflate_size as usize];
            socket.read_exact(&mut buf)?;
            let inflate_size = u16::from_le_bytes([head[14], head[15]]); // 响应信息中的解压后长度


            socket.write_all(&PACK3)?;
            let mut head = [0u8; 16];
            let head_size = socket.read(&mut head)?;
            let deflate_size = u16::from_le_bytes([head[12], head[13]]); // 响应信息中的待解压长度
            let mut buf = vec![0; deflate_size as usize];
            socket.read_exact(&mut buf)?;
            let inflate_size = u16::from_le_bytes([head[14], head[15]]); // 响应信息中的解压后长度
            
            println!("Connected to server");

        }
        Err(_) => {}
    };

    Ok(())
}
