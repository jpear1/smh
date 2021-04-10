use smh::argparser::find_destination_candidate_mut;
use smh::config::{get_host_map_from_config, read_user_config};
use smh::arpscan::scan;
use smh::ssh::connect;
use smh::{Destination, Host};

fn main() {
    let mut args = std::env::args().collect();
    let ds_str = find_destination_candidate_mut(&mut args).unwrap();

    let mut ds: Destination = ds_str.parse().unwrap();
    
    let config_str = read_user_config().unwrap();

    let host_map = get_host_map_from_config(&config_str).unwrap();
    
    if let Host::Named(host) = &mut ds.host {
        if let Some(addr) = host_map.get(host) {
            ds.host = Host::Addressed(*addr);
        }
    }

    let mac_map = scan().unwrap();
    
    if let Host::Addressed(addr) = &mut ds.host {
        if let Some(addr) = mac_map.get(addr) {
            ds.host = Host::Resolved(*addr);
        } else {
            eprint!("Unable to find MAC Address <{}> on local network", *addr);
            std::process::exit(1);
        }
    }
    
    *ds_str = ds.to_string();
    args.remove(0);
    connect(&args).unwrap();
}
