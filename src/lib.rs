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

fn host() {
    let addr = SocketAddr { ip: Ipv4Addr(167, 114, 96, 204), port: 34567 };
    let mut socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e) => panic!("HOST Failed to bind socket: {}", e)
    };

    let mut buf = [0u8; 256];
    match socket.recv_from(&mut buf) {
        Ok((len, src_addr)) => {
            println!("Received {} bytes!", len);
            println!("First byte is {}", buf[0]);
            println!("And it was from {}", src_addr);
            let mut buf = [10u8];
            println!("Now we're gonna send them a {}", buf[0]);
            unsafe { socket.send_to(buf.as_slice(), src_addr); }
        }

        Err(e) => panic!("FUCK {}", e)
    }
}

#[test]
#[should_fail]
fn it_works() {
    host();
    join();
}
