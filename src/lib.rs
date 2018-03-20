extern "C" {
    pub fn sign_c(
        out: *mut u8,
        out_len: *mut usize,
        msg: *const u8,
        msg_len: usize,
        data: *const u8,
    );
    pub fn verify_c(
        msg: *const u8,
        msg_len: usize,
        data_s: *const u8,
        data_g: *const u8,
        data_p: *const u8,
    ) -> i32;
    pub fn key_gen_c(
        out_sk: *mut u8,
        sk_len: *mut usize,
        out_pk: *mut u8,
        pk_len: *mut usize,
        out_g: *mut u8,
        g_len: *mut usize,
    );
}

pub const PRIVATE_KEY_LEN: usize = 20;
pub const PUBLIC_KEY_LEN: usize = 41;
pub const PUBLIC_G_LEN: usize = 41;
pub const SIGN_LEN: usize = 21;

//sign the msg, only msg[0..20] is used.
pub fn sign(msg: &[u8], private_key: &[u8; PRIVATE_KEY_LEN]) -> [u8; SIGN_LEN] {
    unsafe {
        let mut sig = [0u8; SIGN_LEN];
        let mut sig_len = 0usize;
        let msg_len = msg.len();

        sign_c(
            sig.as_mut_ptr(),
            &mut sig_len,
            msg.as_ptr(),
            msg_len,
            private_key.as_ptr(),
        );

        debug_assert_eq!(sig_len, SIGN_LEN);
        sig
    }
}

pub fn verify(msg: &[u8], sig: &[u8], public_key: &[u8; PUBLIC_KEY_LEN], g: &[u8; PUBLIC_G_LEN]) -> bool {
    unsafe {
        let msg_len = msg.len();

        let r = verify_c(
            msg.as_ptr(),
            msg_len,
            sig.as_ptr(),
            g.as_ptr(),
            public_key.as_ptr(),
        );

        r != 0
    }
}

pub fn key_gen() -> ([u8; PRIVATE_KEY_LEN], [u8; PUBLIC_KEY_LEN], [u8; PUBLIC_G_LEN]) {
    unsafe {
        let mut private_key = [0u8; PRIVATE_KEY_LEN];
        let mut private_len = 0usize;
        let mut public_key = [0u8; PUBLIC_KEY_LEN];
        let mut public_len = 0usize;
        let mut g = [0u8; PUBLIC_G_LEN];
        let mut g_len = 0usize;

        key_gen_c(
            private_key.as_mut_ptr(),
            &mut private_len,
            public_key.as_mut_ptr(),
            &mut public_len,
            g.as_mut_ptr(),
            &mut g_len,
        );

        debug_assert_eq!(private_len, PRIVATE_KEY_LEN);
        debug_assert_eq!(public_len, PUBLIC_KEY_LEN);
        debug_assert_eq!(g_len, PUBLIC_G_LEN);

        (private_key, public_key, g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let (private_key, public_key, g) = key_gen();
        let msg = vec![1; 20];
        let sig = sign(&msg, &private_key);
        let r = verify(&msg, &sig, &public_key, &g);
        assert_eq!(r, true);

        let msg = vec![2; 20];
        let r = verify(&msg, &sig, &public_key, &g);
        assert_eq!(r, false);
    }

    #[test]
    fn test_key_gen() {
        let (private_key, public_key, g) = key_gen();

        println!("{:?} {} {}", private_key, public_key.len(), g.len());
    }
}
