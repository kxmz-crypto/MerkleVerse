use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};

pub fn b64_to_loc(b64: &str, length: usize) -> Result<Vec<u8>> {
    let mut bytes = general_purpose::STANDARD.decode(b64)?;
    if bytes.len() < length{
        let mut tmp = vec![0; length - bytes.len()];
        tmp.extend(bytes);
        bytes = tmp;
    }else if bytes.len() > length {
        return Err(anyhow!("Decoded bytes exceded specified length"));
    }
    Ok(bytes)
}

fn binary_string(bytes: Vec<u8>, length: usize) -> String {
    let mut binary_string: String = bytes.iter().map(|byte| format!("{:08b}", byte)).collect();
    if binary_string.len() < length {
        let mut tmp = String::new();
        for _ in 0..(length - binary_string.len()) {
            tmp.push('0');
        }
        tmp.push_str(&binary_string);
        binary_string = tmp;
    } else if binary_string.len() > length {
        binary_string = binary_string.chars().skip(binary_string.len() - length).collect();
    }
    binary_string
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn tst_decoding() -> Result<()>{
        let base = "Ag==";
        assert_eq!(binary_string(b64_to_loc(base, 8)?, 8), "00000010");
        assert_eq!(binary_string(b64_to_loc(base, 3)?, 3), "010");
        let base_2 = "ATs=";
        assert_eq!(binary_string(b64_to_loc(base_2, 9)?, 9), "100111011");
        Ok(())
    }
}