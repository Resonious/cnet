macro_rules! server_test(
  ((timeout: $timeout:expr) ($socket:ident, $host_addr:ident) $test:expr) => ({
    let (c_tx, c_rx) = mpsc::channel();
    thread::spawn(move || {
      let ip = Ipv4Addr::new(0, 0, 0, 0);
      let addr = SocketAddr::new(IpAddr::V4(ip), 34550);
      let socket = match UdpSocket::bind(addr) {
        Ok(s) => s,
        Err(e1) => {
          match UdpSocket::bind(SocketAddr::new(IpAddr::V4(ip), 34540)) {
            Ok(s) => s,
            Err(e2) => {
              c_tx.send(false).unwrap();
              panic!("Attempt #2 to bind to socket failed. error 1: {}, error 2: {}", e1, e2);
            }
          }
        }
      };

      let host_ip = Ipv4Addr::new(127, 0, 0, 1);
      let host_addr = SocketAddr::new(IpAddr::V4(host_ip), 34561);

      let test_procedure = |$socket:UdpSocket, $host_addr:SocketAddr| { $test; };
      test_procedure(socket, host_addr);

      c_tx.send(true).unwrap();
    });

    let mut client_done = false;
    let mut client_died = false;

    let mut time = Duration::seconds(0);

    while !client_done {
      time = time + Duration::span(|| {
        if !client_done && !client_died {
          match c_rx.try_recv() {
            Ok(good) => if good { client_done = true } else { client_died = true },
            Err(e) => match e {
              Empty => {}
              Disconnected => client_died = true
            }
          }
        }
      });

      if client_died {
        break;
      }
      if time > $timeout {
        panic!("Timed out");
      }
    }

    assert!(!client_died);
    assert!(client_done);
  });

  (($socket:ident, $host_addr:ident) $test:expr) => (
    server_test!((timeout: Duration::seconds(10))
                 ($socket, $host_addr) $test);
  );
);
