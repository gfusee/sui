use std::net::SocketAddr;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Multiaddr(::multiaddr::Multiaddr);

impl Multiaddr {
    pub fn empty() -> Self {
        Self(::multiaddr::Multiaddr::empty())
    }
}

impl std::fmt::Display for Multiaddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Multiaddr {
    type Err = multiaddr::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Multiaddr)
    }
}

impl Multiaddr {
    pub fn replace<F>(&self, at: usize, by: F) -> Option<Multiaddr>
    where
        F: FnOnce(&mut ::multiaddr::Multiaddr)
            -> Result<Option<::multiaddr::Protocol<'_>>, multiaddr::Error>,
    {
        let mut cloned = self.0.clone();
        let protos = cloned.iter().collect::<Vec<_>>();
        if at >= protos.len() {
            return None;
        }
        let mut builder = protos.iter().cloned().collect::<::multiaddr::MultiaddrBuilder>();
        if let Ok(Some(p)) = by(&mut cloned) {
            builder.set(at, p);
        }
        Some(Multiaddr(builder.build()))
    }

    pub fn to_socket_addr(&self) -> Option<SocketAddr> {
        self.0.iter().fold(None, |mut address, proto| {
            match proto {
                ::multiaddr::Protocol::Ip4(ip) => {
                    address.get_or_insert_with(|| SocketAddr::new((*ip).into(), 0));
                }
                ::multiaddr::Protocol::Ip6(ip) => {
                    address.get_or_insert_with(|| SocketAddr::new((*ip).into(), 0));
                }
                ::multiaddr::Protocol::Tcp(port) => {
                    if let Some(addr) = &mut address {
                        addr.set_port(*port);
                    }
                }
                ::multiaddr::Protocol::Udp(port) => {
                    if let Some(addr) = &mut address {
                        addr.set_port(*port);
                    }
                }
                _ => {}
            }
            address
        })
    }
}

pub mod multiaddr {
    pub use super::Multiaddr;
}
