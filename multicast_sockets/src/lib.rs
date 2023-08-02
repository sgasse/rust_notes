use pnet_datalink::NetworkInterface;

pub fn get_interface_by_name(name: &str) -> Option<NetworkInterface> {
    pnet_datalink::interfaces()
        .into_iter()
        .filter(pnet_datalink::NetworkInterface::is_multicast)
        .find(|intf| intf.name == name)
}
