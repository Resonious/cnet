use std::io::net::udp::UdpSocket;
use std::io::net::ip::{Ipv4Addr, SocketAddr};
use std::thread::Thread;
use std::mem::transmute;

fn join() {
    let local_addr = SocketAddr { ip: Ipv4Addr(0, 0, 0, 0), port: 34568 };
    let mut socket = match UdpSocket::bind(local_addr) {
        Ok(s) => s,
        Err(e) => panic!("JOIN Failed to bind socket: {}", e)
    };

    let host_addr = SocketAddr { ip: Ipv4Addr(167, 114, 96, 204), port: 34567 };
    let mut buf = [8u8];
    unsafe { socket.send_to(transmute(buf.as_slice()), host_addr); }

    println!("Sent data, now waiting for response");

    let mut in_buf = [0u8; 256];
    match socket.recv_from(&mut in_buf) {
        Ok((len, src_addr)) => {
            println!("Received response of {} bytes!", len);
            println!("First byte is {}", in_buf[0]);
            println!("Response was from {}", src_addr);
            println!("Host is {}", host_addr);
        }

        Err(e) => panic!("SHIT! {}", e)
    }
}

fn main() {
    join();
}
