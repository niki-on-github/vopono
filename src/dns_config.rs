use anyhow::Context;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::IpAddr;
use std::io::BufRead;
use regex::{Regex,Captures};

#[derive(Serialize, Deserialize, Debug)]
pub struct DnsConfig {
    ns_name: String,
}

impl DnsConfig {
    pub fn new(ns_name: String, servers: &[IpAddr], suffixes: &[&str]) -> anyhow::Result<Self> {
        std::fs::create_dir_all(format!("/etc/netns/{}", ns_name))
            .with_context(|| format!("Failed to create directory: /etc/netns/{}", ns_name))?;

        let mut resolv = std::fs::File::create(format!("/etc/netns/{}/resolv.conf", ns_name))
            .with_context(|| {
                format!(
                    "Failed to open resolv.conf: /etc/netns/{}/resolv.conf",
                    ns_name
                )
            })?;

        debug!(
            "Setting namespace {} DNS server to {}",
            ns_name,
            &servers
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        );

        let suffix = suffixes.join(" ");
        if !suffix.is_empty() {
            writeln!(resolv, "search {}", suffix).with_context(|| {
                format!(
                    "Failed to overwrite resolv.conf: /etc/netns/{}/resolv.conf",
                    ns_name
                )
            })?;
        }

        for dns in servers {
            writeln!(resolv, "nameserver {}", dns).with_context(|| {
                format!(
                    "Failed to overwrite resolv.conf: /etc/netns/{}/resolv.conf",
                    ns_name
                )
            })?;
        }

        let mut hosts = std::fs::File::create(format!("/etc/netns/{}/hosts", ns_name))
            .with_context(|| {
                format!(
                    "Failed to open hosts: /etc/netns/{}/hosts",
                    ns_name
                )
            })?;

        writeln!(hosts, "{} jdownloader.server01.lan", "10.0.1.10").with_context(|| {
            format!(
                "Failed to overwrite hosts: /etc/netns/{}/hosts",
                ns_name
            )
        })?;

        writeln!(hosts, "{} vaultwarden.server01.lan", "10.0.1.10").with_context(|| {
            format!(
                "Failed to overwrite hosts: /etc/netns/{}/hosts",
                ns_name
            )
        })?;

        let src = std::fs::File::open("/etc/nsswitch.conf")?;
        let file = std::fs::File::create(format!("/etc/netns/{}/nsswitch.conf", ns_name)).unwrap();
        let mut file = std::io::LineWriter::new(file);
        let reader = std::io::BufReader::new(src);
        for line in reader.lines() {
            let re = Regex::new(r"^hosts:.*$").unwrap();
            let buffer = line?;
            let k = re.replace(&buffer, |_caps: &Captures| { "hosts: files mymachines myhostname dns" });
            file.write(k.as_ref().as_ref()).unwrap();
            write!(file, "\n")?;
        }

        file.flush().unwrap();


        Ok(Self { ns_name })
    }
}

impl Drop for DnsConfig {
    fn drop(&mut self) {
        let path = format!("/etc/netns/{}", self.ns_name);
        match std::fs::remove_dir_all(&path) {
            Ok(_) => {}
            Err(e) => warn!(
                "Failed to delete network namespace directory: {}: {:?}",
                &path, e
            ),
        }
    }
}
