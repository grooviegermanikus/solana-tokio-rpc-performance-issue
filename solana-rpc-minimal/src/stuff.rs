use {
    crossbeam_channel::unbounded,
    // log::*,
    rand::{thread_rng, Rng},
    socket2::{Domain, SockAddr, Socket, Type},
    std::{
        collections::{BTreeMap, HashSet},
        io::{self, Read, Write},
        net::{IpAddr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket},
        sync::{Arc, RwLock},
        time::{Duration, Instant},
    },
    url::Url,
};

pub type PortRange = (u16, u16);

pub fn find_available_port_in_range(ip_addr: IpAddr, range: PortRange) -> io::Result<u16> {
    let (start, end) = range;
    let mut tries_left = end - start;
    let mut rand_port = thread_rng().gen_range(start, end);
    loop {
        match bind_common(ip_addr, rand_port, false) {
            Ok(_) => {
                break Ok(rand_port);
            }
            Err(err) => {
                if tries_left == 0 {
                    return Err(err);
                }
            }
        }
        rand_port += 1;
        if rand_port == end {
            rand_port = start;
        }
        tries_left -= 1;
    }
}

pub fn bind_common(
    ip_addr: IpAddr,
    port: u16,
    reuseaddr: bool,
) -> io::Result<(UdpSocket, TcpListener)> {
    let sock = udp_socket(reuseaddr)?;

    let addr = SocketAddr::new(ip_addr, port);
    let sock_addr = SockAddr::from(addr);
    sock.bind(&sock_addr)
        .and_then(|_| TcpListener::bind(addr).map(|listener| (sock.into(), listener)))
}

#[cfg(any(windows, target_os = "ios"))]
fn udp_socket(_reuseaddr: bool) -> io::Result<Socket> {
    let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    Ok(sock)
}

#[cfg(not(any(windows, target_os = "ios")))]
fn udp_socket(reuseaddr: bool) -> io::Result<Socket> {
    use {
        nix::sys::socket::{
            setsockopt,
            sockopt::{ReuseAddr, ReusePort},
        },
        std::os::unix::io::AsRawFd,
    };

    let sock = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    let sock_fd = sock.as_raw_fd();

    if reuseaddr {
        // best effort, i.e. ignore errors here, we'll get the failure in caller
        setsockopt(sock_fd, ReusePort, &true).ok();
        setsockopt(sock_fd, ReuseAddr, &true).ok();
    }

    Ok(sock)
}