pub fn f32_to_string(f: f32, precision: usize) -> String {
    let mut s = f.to_string();
    let mut split = s.split('.');

    // the number won't always have a decimal and fraction, so we account for it
    let decimal = match split.next() {
        Some(s) => s.to_string(),
        None => "".to_string(),
    };
    let mut fraction = match split.next() {
        Some(s) => s.to_string(),
        None => "".to_string(),
    };

    if fraction.len() > precision {
        fraction = fraction[..precision].to_string();
    }
    if fraction.len() < precision {
        fraction = format!("{:0>precision$}", fraction, precision = precision);
    }
    s = format!("{}.{}", decimal, fraction);
    s
}