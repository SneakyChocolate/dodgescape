
pub fn get_bits(byte: u8) -> String {
    let mut bits = "".to_owned();
    for i in 0..8 {
        let mask = 1 << i;
        let bit = mask & byte;
        bits.push_str(format!("{}", bit).as_str());
    }
    bits
}

pub fn get_bits_vec(bytes: &Vec<u8>) -> String {
    let mut bits = "".to_owned();
    for byte in bytes.iter() {
        bits.push_str(get_bits(*byte).as_str());
    }
    bits
}
