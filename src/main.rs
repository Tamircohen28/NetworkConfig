use std::os::unix::prelude::RawFd;

// Linux supports some standard ioctls to configure network devices.
// They can be used on any socket's file descriptor regardless of
// the family or type. Most of them pass an ifreq structure.
// Source: netdevice(7)
use anyhow::{bail, Result};
use ifstructs::ifreq;
use nix::libc::{SIOCGIFADDR, SIOCSIFADDR};
use nix::sys::socket::{socket, AddressFamily, SockFlag, SockProtocol, SockType};
use nix::{ioctl_read_bad, ioctl_write_ptr_bad};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Create an endpoint for communication
fn crate_sock<T: Into<Option<SockProtocol>>>(
    domain: AddressFamily,
    ty: SockType,
    flags: SockFlag,
    protocol: T,
) -> Result<RawFd> {
    Ok(socket(domain, ty, flags, protocol)?)
}

ioctl_read_bad!(get_interface_ip, SIOCGIFADDR, ifreq);
ioctl_write_ptr_bad!(set_interface_ip, SIOCSIFADDR, ifreq);

fn get_ip(ifr: &ifreq) -> Result<IpAddr> {
    let sock_addr = unsafe { ifr.ifr_ifru.ifr_addr };

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
        libc::AF_INET6 => {
            let mut arr = [0u16; 8];
            for i in 0..arr.len() {
                arr[i] = sock_addr.sa_data[i + 6] as u16;
            }
            Ok(IpAddr::from(Ipv6Addr::from(arr)))
        }
        _ => bail!("Received unknown sa_family"),
    }
}

fn main() -> Result<()> {
    let sock_fd = crate_sock(
        AddressFamily::Inet,
        SockType::Datagram,
        SockFlag::empty(),
        None,
    )?;
    let mut ifreq = ifreq::from_name("ens36")?;

    unsafe { get_interface_ip(sock_fd, &mut ifreq)? };
    let address = get_ip(&ifreq)?;
    println!("Interface ip is: {:?}", address);
    Ok(())
}
