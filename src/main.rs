//! Linux supports some standard ioctls to configure network devices.
//! They can be used on any socket's file descriptor regardless of
//! the family or type. Most of them pass an ifreq structure.
//! Source: netdevice(7)
use std::os::unix::prelude::RawFd;
use anyhow::{bail, Result};
use ifstructs::ifreq;
use nix::libc::{SIOCGIFADDR, SIOCSIFADDR};
use nix::sys::socket::{socket, AddressFamily, SockFlag, SockProtocol, SockType};
use nix::{ioctl_read_bad, ioctl_write_ptr_bad};
use simple_logger::SimpleLogger;
use std::net::{IpAddr, Ipv4Addr};
use structopt::StructOpt;
use log::info;

/// Create an endpoint for communication
fn crate_sock<T: Into<Option<SockProtocol>>>(
    domain: AddressFamily,
    ty: SockType,
    flags: SockFlag,
    protocol: T,
) -> Result<RawFd> {
    Ok(socket(domain, ty, flags, protocol)?)
}

// Creation of icotl functions needed
ioctl_read_bad!(get_interface_ip, SIOCGIFADDR, ifreq);
ioctl_write_ptr_bad!(set_interface_ip, SIOCSIFADDR, ifreq);

/// Get `IpAddr` from sockaddr
pub fn ip_from_sockaddr(sock_addr: &libc::sockaddr) -> Result<IpAddr> {
    match sock_addr.sa_family as i32 {
        // IPV4
        libc::AF_INET => {
            let mut arr = [0u8; 4];
            for i in 0..arr.len() {
                arr[i] = sock_addr.sa_data[i + 2] as u8;
            }
            Ok(IpAddr::from(Ipv4Addr::from(arr)))
        }
        // IPV6
        libc::AF_INET6 => bail!("IPv6 is not supported at the time"),
        _ => bail!("Received unknown sa_family"),
    }
}

/// Get `sockaddr` from IpAddr
pub fn sockaddr_from_ip(ip_addr: &IpAddr) -> Result<libc::sockaddr> {
    let sa_family: libc::sa_family_t;
    let mut sa_data = [0i8; 14];

    match ip_addr {
        IpAddr::V4(ip) => {
            sa_family = libc::AF_INET as libc::sa_family_t;
            let data = ip.octets();
            for i in 0..data.len() {
                sa_data[i + 2] = data[i] as i8;
            }
        }
        _ => bail!("IPv6 is not supported at the time")
    };

    Ok(libc::sockaddr {
        sa_family,
        sa_data
    })
}

// get the ip of interface
pub fn get_ip(ifr: &ifreq) -> Result<IpAddr> {
    ip_from_sockaddr(unsafe { &ifr.ifr_ifru.ifr_addr })
}

// set the ip of interface
pub fn set_ip(ifr: &mut ifreq, ip_addr: &IpAddr) -> Result<()> {
    Ok(ifr.ifr_ifru.ifr_addr = sockaddr_from_ip(ip_addr)?)
}

#[derive(Debug, StructOpt)]
struct Args {
    /// Interface to set IP
    interface: String,

    /// IPv4 to set
    ip: Ipv4Addr,
}

fn main() -> Result<()> {
    SimpleLogger::new().init()?;
    let args = Args::from_args();

    info!("Opening socket to kernel...");
    let sock_fd = crate_sock(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )?;

    let mut ifreq = ifreq::from_name(&args.interface)?;
    let new_addr = IpAddr::from(args.ip);
    set_ip(&mut ifreq, &new_addr)?;
    unsafe { set_interface_ip(sock_fd, &mut ifreq)? };
    info!("Interface '{}' set to ip address '{}' succesfully!", args.interface, new_addr);
    Ok(())
}
