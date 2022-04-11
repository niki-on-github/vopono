use crate::firewall::Firewall;
use std::net::IpAddr;
use crate::netns::NetworkNamespace;

pub fn open_hosts(
    netns: &NetworkNamespace,
    hosts: Vec<IpAddr>,
    firewall: Firewall,
) -> anyhow::Result<()> {
    for host in hosts {
        match firewall {
            Firewall::IpTables => {
                netns.exec(&[
                    "iptables",
                    "-I",
                    "OUTPUT",
                    &host.to_string(),
                    "-j",
                    "ACCEPT",
                ])?;
            }
            Firewall::NfTables => {
                netns.exec(&["nft", "add", "table", "inet", &netns.name])?;
                netns.exec(&[
                    "nft",
                    "add",
                    "chain",
                    "inet",
                    &netns.name,
                    "output",
                    "{ type filter hook output priority 100 ; }",
                ])?;
                netns.exec(&[
                    "nft",
                    "add",
                    "rule",
                    "inet",
                    &netns.name,
                    "output",
                    "ip",
                    "daddr",
                    &host.to_string(),
                    "counter",
                    "accept",
                ])?;
            }
        }
    }
    Ok(())
}
