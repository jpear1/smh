use macaddr::MacAddr6;
use std::{fmt::{Display}, net::Ipv4Addr};
use std::str::FromStr;
use std::string::ToString;
use regex::Regex;
use anyhow::Result;

#[derive(Debug)]
pub struct Destination {
    scheme: Option<String>,
    user: Option<String>,
    pub host: Host,
    port: Option<String>
}

const MAC_ADDRESS_REGEX: &str = r"^(?:[[:xdigit:]]{2}:){5}[[:xdigit:]]{2}$";
const DESTINATION_REGEX: &str = r"^(ssh://)?([[:alnum:]]+@)?(.+?)(:[[:digit:]]+)?$";

impl FromStr for Destination {
    
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Destination> {
        
        let dest_regex = Regex::new(DESTINATION_REGEX).unwrap();
        // regex only fails to match empty string
        let captures = dest_regex.captures(s).unwrap();

        let scheme = match captures.get(1) {
            Some(mat) => Some(String::from(mat.as_str())),
            None => None
        };

        let user = match captures.get(2) {
            Some(mat) => Some(String::from(mat.as_str())),
            None => None
        };

        let host_str = captures.get(3).unwrap().as_str();
        let host = Host::from_str(host_str).unwrap();

        let port = match captures.get(4) {
            Some(mat) => Some(String::from(mat.as_str())),
            None => None
        };
        
        Ok(Destination{scheme, user, host, port})
    }
}

impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let scheme = match self.scheme.as_ref() {
            Some(s) => s,
            None => ""
        };
        let user = match self.user.as_ref() {
            Some(s) => s,
            None => ""
        };
        let port = match self.port.as_ref() {
            Some(s) => s,
            None => ""
        };
        write!(f, "{}{}{}{}", scheme, user, self.host, port)
    }
}

#[derive(Debug)]
pub enum Host {
    Addressed(MacAddr6),
    Named(String),
    Resolved(Ipv4Addr)
}

impl Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
                Host::Addressed(addr) => write!(f, "{}", &addr.to_string()),
                Host::Named(name) => write!(f, "{}", &name),
                Host::Resolved(addr) => write!(f, "{}", &addr.to_string())
        }
    }
}

impl FromStr for Host {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Host> {
        let mac_regex = Regex::new(MAC_ADDRESS_REGEX).unwrap();
        match mac_regex.captures(s) {
            Some(captures) => {
                let macaddr = captures.get(0).unwrap().as_str();
                Ok(Host::Addressed(macaddr.parse().unwrap()))
            },
            None => Ok(Host::Named(String::from(s)))
        }
    }
}

pub mod ssh {
    use std::process::Command;
    use anyhow::{Result, bail};

    pub fn connect(args: &Vec<String>) -> Result<()> {
        let mut child = Command::new("ssh")
                         .args(args)
                         .spawn()?;
        
        let exit_status = child.wait().expect("Unable to wait on SSH process for some reason");
        
        if exit_status.code().unwrap() != 0 {
            bail!("ssh exited with error");
        }

        Ok(())
    }
}

pub mod arpscan {
    use std::collections::BTreeMap;
    use std::process::Command;
    use macaddr::MacAddr6;
    use anyhow::{Result, ensure};
    use std::net::Ipv4Addr;

    pub fn scan() -> Result<BTreeMap<MacAddr6, Ipv4Addr>> {
        let output = Command::new("arp-scan")
                         .arg("--localnet")
                         .output()
                         .expect("Unable to launch arp-scan, please verify that arp-scan is in your path");

        ensure!(output.status.success(), "Arp scan exited with nonzero exit code");

        let output = String::from_utf8(output.stdout).unwrap();
        parse_arp_output(&output)
    }
    
    pub fn parse_arp_output(arpout: &str) -> Result<BTreeMap<MacAddr6, Ipv4Addr>> {
        let lines: Vec<&str> = arpout.lines().collect();
        ensure!(lines.len() >= 5, "Arp scan returned invalid input");
        let mut map = BTreeMap::new();
        for i in 2..lines.len()-3 {
            let mut tokens = lines[i].trim().split("\t");
            let ipaddr = tokens.next().unwrap();
            let ipaddr = ipaddr.parse::<Ipv4Addr>()?;
            let macaddr = tokens.next().unwrap().parse::<MacAddr6>()?;
            map.insert(macaddr, ipaddr);
        }
        Ok(map)
    }
}

pub mod argparser {
    pub fn find_destination_candidate_mut(args: &mut Vec<String>) -> Option<&mut String> {
        let mut nondashed_count = 1;
        let mut it = args.iter_mut();
        it.next();
        for arg in it {
            if arg.starts_with("-") {
                nondashed_count = 0;
            } else {
                nondashed_count += 1;
            };
            if nondashed_count == 2 {
                return Some(arg);
            }
        }
        None
    }
}

pub mod config {
    use std::fs;
    use etc_passwd::Passwd;
    use std::collections::HashMap;
    use anyhow::{Result, bail};
    use macaddr::MacAddr6;
    use toml::value::Table;
    use toml::value::Value;

    pub fn get_host_map_from_config(s: &str) -> Result<HashMap<String, MacAddr6>> {
        let config = s.parse::<toml::Value>()?;
        let config_table = config.try_into::<Table>()?;
        let host_table = match config_table.get("Hosts") {
            Some(Value::Table(t)) => t,
            _ => bail!("Config does not contain table \"Hosts\"")
        };
        let mut host_map = HashMap::<String, MacAddr6>::new();

        for (host, val) in host_table.iter() {
            if let Value::String(macstr) = val {
                if let Ok(addr) = macstr.parse::<MacAddr6>() {
                    host_map.insert(String::from(host), addr);
                    continue;
                }
            }
            bail!("Invalid MAC Address: {}", val);
        }
        Ok(host_map)
    }
    
    pub fn read_user_config() -> Result<String> {
        let cur_user = Passwd::current_user()?.unwrap();
        let user_home_dir = cur_user.dir.to_str()?;

        let path = format!("{}/.config/smh/hosts.toml", &user_home_dir);
        
        Ok(String::from_utf8(fs::read(path)?)?)
    }
}

pub mod tests {
    #[test]
    fn test_parse() {
        use crate::arpscan::parse_arp_output;

        let filler_output = "Interface: wlo1, type: EN10MB, MAC: 10:5b:ad:07:05:25, IPv4: 192.168.1.147
        Starting arp-scan 1.9.7 with 256 hosts (https://github.com/royhills/arp-scan)
        192.168.1.3	46:02:b2:12:e3:cc	(Unknown: locally administered)
        192.168.1.4	c6:62:a9:12:52:c3	(Unknown: locally administered)
        192.168.1.5	ea:42:09:16:a1:c5	(Unknown: locally administered)

        13 packets received by filter, 0 packets dropped by kernel
        Ending arp-scan 1.9.7: 256 hosts scanned in 2.009 seconds (127.43 hosts/sec). 13 responded".to_string();

        let map_result = parse_arp_output(&filler_output);
        assert!(matches!(map_result, Result::Ok(_)));
        let map = map_result.unwrap();
        assert_eq!(map.len(), 3);
    }
    
    #[test] 
    fn test_find_dest() {
        use crate::argparser::find_destination_candidate_mut;
        let mut args: Vec<String> = vec![String::from("smh"), String::from("-l"), String::from("foo"), String::from("dest")];
        let ds = find_destination_candidate_mut(&mut args).unwrap();
        assert_eq!(*ds, String::from("dest"));
    }
}