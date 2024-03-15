use port_scanner::local_port_available;

fn main() {
    let mut i = 0;
    while i < 10000 {
        println!("{}{}", i, local_port_available(i));
        i += 1;
    }
}
