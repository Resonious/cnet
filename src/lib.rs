use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::thread::Thread;
use std::mem::transmute;

struct Header {
    op: i8,
}

fn join() {
    let local_addr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 34568 };
    let mut socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("JOIN Failed to bind socket: {}", e)
    };

    let host_addr = SocketAddr { ip: Ipv4Addr(167, 114, 96, 204), port: 34567 };
    let mut buf = [Header { op: 8 }];
    unsafe { socket.send_to(transmute(buf.as_slice()), host_addr); }
}

fn host() {
    let addr = SocketAddr { ip: Ipv4Addr(167, 114, 96, 204), port: 34567 };
    let mut socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => panic!("HOST Failed to bind socket: {}", e)
    };

    let mut buf = [0u8; 256];
    match socket.recv_from(&mut buf) {
        Ok((len, src)) => {
            println!("Received {} bytes!", len);
            println!("First byte is {}", buf[0]);
            println!("And it was from {}", src);
        }

        Err(e) => panic!("FUCK")
    }
}

#[test]
fn it_works() {
    join();
}
