pub fn f32_to_string(f: f32, precision: usize) -> String {
    let mut s = f.to_string();
    let mut split = s.split('.');
    let mut decimal = split.next().unwrap().to_string();
    let mut fraction = split.next().unwrap().to_string();
    if fraction.len() > precision {
        fraction = fraction[..precision].to_string();
    }
    if fraction.len() < precision {
        fraction = format!("{:0>precision$}", fraction, precision = precision);
    }
    s = format!("{}.{}", decimal, fraction);
    s
}