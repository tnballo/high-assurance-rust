#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#[allow(warnings)]
#[derive(Debug)]
pub struct Rc4 {
    s: [u8; 256],
    i: u8,
    j: u8,
}

// ANCHOR: new_error_handling
#[derive(Debug)]
pub enum Rc4Error {
    KeyTooShort(usize),
    KeyTooLong(usize),
}

impl Rc4 {
    /// Init a new Rc4 stream cipher instance
    pub fn new(key: &[u8]) -> Result<Self, Rc4Error> {
        const MIN_KEY_LEN: usize = 5;
        const MAX_KEY_LEN: usize = 256;

        // Verify valid key length (40 to 2048 bits)
        if key.len() < MIN_KEY_LEN {
            return Err(Rc4Error::KeyTooShort(MIN_KEY_LEN));
        } else if key.len() > MAX_KEY_LEN {
            return Err(Rc4Error::KeyTooLong(MAX_KEY_LEN));
        }

        // Zero-init our struct
        let mut rc4 = Rc4 {
            s: [0; 256],
            i: 0,
            j: 0,
        };

        // ...more initialization code here...

        // Return our initialized Rc4
        Ok(rc4)
    }
}
// ANCHOR_END: new_error_handling

#[cfg(test)]
mod tests {
    #[allow(warnings)]
    use super::{Rc4, Rc4Error};

    // TODO: add error test
    #[test]
    fn test_new() {
        let key = [0x1, 0x2, 0x3];
        match Rc4::new(&key) {
            Ok(rc4) => println!("Do en/decryption here!"),
            Err(e) => match e {
                Rc4Error::KeyTooShort(min) => eprintln!("Key len >= {} bytes required!", min),
                Rc4Error::KeyTooLong(max) => eprintln!("Key len <= {} bytes required!", max),
            },
        }
    }
}
