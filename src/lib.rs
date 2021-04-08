pub mod ssh {
    use std::process::Command;
    use anyhow::Result;
    pub fn connect(addr: &str) -> Result<()> {
        let mut child = Command::new("ssh")
                         .arg(addr)
                         .spawn()
                         .expect("Unable to launch ssh, please verify that ssh is in your path");
        
        child.wait().expect("Unable to wait on SSH process for some reason");

        Ok(())
    }
}

pub mod arptools {
    use std::collections::BTreeMap;
    use std::process::Command;
    use mac_address::MacAddress;
    use ipaddress::IPAddress;
    use anyhow::{Result, ensure};

    pub fn scan() -> Result<BTreeMap<IPAddress, MacAddress>> {
        let output = Command::new("arp-scan")
                         .arg("--localnet")
                         .output()
                         .expect("Unable to launch arp-scan, please verify that arp-scan is in your path");

        ensure!(output.status.success(), "Arp scan exited with nonzero exit code");

        let output = String::from_utf8(output.stdout).unwrap();
        parse_arp_output(&output)
    }
    
    fn parse_arp_output(arpout: &str) -> Result<BTreeMap<IPAddress, MacAddress>> {
        let lines: Vec<&str> = arpout.lines().collect();
        ensure!(lines.len() >= 4, "Arp scan returned invalid input");
        let mut map = BTreeMap::new();
        for i in 2..lines.len()-3 {
            let mut tokens = lines[i].split("\t");
            let ipaddr = IPAddress::parse(tokens.next().unwrap()).unwrap();
            let macaddr: MacAddress = tokens.next().unwrap().parse()?;
            map.insert(ipaddr, macaddr);
        }
        Ok(map)
    }
}