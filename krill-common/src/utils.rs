pub struct KrillUtils;

impl KrillUtils {
    pub fn array_of_bytes_to_hex(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|byte| format!("{:0x?} ", byte))
            .collect::<String>()
            .trim()
            .to_string()
    }
}
