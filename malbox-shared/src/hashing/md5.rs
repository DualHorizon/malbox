use md5::compute;

pub fn get_md5(buf: &mut [u8]) -> String {
    let digest = compute(buf);
    format!("{:x}", digest)
}
