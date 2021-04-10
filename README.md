# SMH
## Secure MAC sHell

This program allows you to establish SSH connections using the MAC address of
the machine you want to SSH into. This is useful if you have multiple devices
you take with you to different DHCP networks and you want to be able to SSH into
one from the other.

An alternate solution to this problem is
[mDNS](https://en.wikipedia.org/wiki/Multicast_DNS), and this will work better
in most situations, because it works with all kinds of connections, including
HTTP.  However, if you're unable to install an mDNS server on each of your
devices, but you can install `arp-scan`, `smh` will be helpful to you. I
encountered this issue using [Termux](https://github.com/termux/termux-app) on a
rooted Android device, which prompted me to write this tool.

## Dependencies

`smh` is just a wrapper around `ssh`, so `ssh` is one dependency.

You'll also need a working installation of `arp-scan` in your path, as this is
the program `smh` uses to identify devices on your network.

SMH only uses EUI-48 MAC addresses, and IPv4 addresses, so your devices and
networks will need to use these. As far as I know, pretty much every device and
network will use these.

## Installation

Hopefully I'll get around to packaging this, but for now you'll need to install
from source. Just make sure you have a rust toolchain installed (`cargo`, etc.),
then run these commands:

```
git clone https://github.com/jpear1/smh.git
cd smh
cargo build --release
cp target/release/smh <directory-in-path>
```

where `<directory-in-path>` is where you usually put your binaries. `/usr/bin`,
for example.


## Usage

SSH's destination field is on the form `[user@]hostname` or
`ssh://[user@]hostname[:port]`. If `hostname` is a MAC address, SMH will replace
`hostname` with the corresponding IP address. If `hostname` is a configured
name, SMH will replace `hostname` with the IP address that identifies a device
with the configured MAC address.

Except for this one change, SMH will forward all other arguments to SSH.

Here's a couple of examples:

```
smh user@12:34:56:78:9a:bc
```

Establishes an SSH connection to the device with MAC address `12:34:56:78:9a:bc`

```
smh user@my-phone
```

Looks up `my-phone` to see if it has a configured MAC address. If it does,
replaces `my-phone` with corresponding IP address, otherwise forwards `my-phone`
to `ssh`.

## Configuration

SMH is configured with `~/.config/smh/hosts.toml`. There should be one table,
`[Hosts]`, then as many `hostname = "<MAC-address>"` entries as you please.

Ex.

```
# ~/.config/smh/hosts.toml
[Hosts]
foo = "aa:aa:aa:aa:aa:aa"
bar = "bb:bb:bb:bb:bb:bb"
```

Note that if you want to put dots in your hostname, you need to enclose the
hostname in quotes. So, write:
```
"phone.local" = "cc:cc:cc:cc:cc:cc"
```
Don't write:
```
phone.local = "cc:cc:cc:cc:cc:cc"
```

`arp-scan` requires root permissions to run, so you'll need to run `smh` with
`sudo smh hostname` and put your config in `/root/.config/smh/hosts.toml`, or
configure your system to give target users permission to run `smh` as root. You
can do this with:

```
sudo su
chown root:smhers smh
chmod =2750 smh
```

Then add any users who should be able to use `smh` to the `smhers` group with:

```
usermod -a -G smhers <username>
```