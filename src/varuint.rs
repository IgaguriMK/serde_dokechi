use std::io::{self, Read, Write};

/*
    Variable length unsigined integer format

    0, 1 : specified bit
    x    : value bit
    X    : Value byte (= xxxxxxxx)

    0xxxxxxx          :  7bit (0 ~ 127)
    10xxxxxx X        : 14bit (128 ~ 16383)
    110xxxxx XX       : 21bit (16384 ~ 2097151)
    1110xxxx XXX      : 28bit (2097152 ~ 268435455)
    11110xxx XXXX     : 35bit (268435456 ~ 34359738367)
    111110xx XXXXX    : 42bit (34359738368 ~ 4398046511103)
    1111110x XXXXXX   : 49bit (4398046511104 ~ 562949953421311)
    11111110 XXXXXXX  : 56bit (562949953421312 ~ 72057594037927935)
    11111111 XXXXXXXX : 64bit (72057594037927936 ~ 18446744073709551615)
*/

#[allow(unused)]
pub fn encode_u64(mut w: impl Write, v: u64) -> io::Result<()> {
    let bs = v.to_be_bytes();

    match 64 - v.leading_zeros() {
        x if x <= 7 => {
            w.write_all(&[bs[7]])?;
        }
        x if x <= 14 => {
            w.write_all(&[0b1000_0000 | bs[6]])?;
            w.write_all(&bs[7..8])?;
        }
        x if x <= 21 => {
            w.write_all(&[0b1100_0000 | bs[5]])?;
            w.write_all(&bs[6..8])?;
        }
        x if x <= 28 => {
            w.write_all(&[0b1110_0000 | bs[4]])?;
            w.write_all(&bs[5..8])?;
        }
        x if x <= 35 => {
            w.write_all(&[0b1111_0000 | bs[3]])?;
            w.write_all(&bs[4..8])?;
        }
        x if x <= 42 => {
            w.write_all(&[0b1111_1000 | bs[2]])?;
            w.write_all(&bs[3..8])?;
        }
        x if x <= 49 => {
            w.write_all(&[0b1111_1100 | bs[1]])?;
            w.write_all(&bs[2..8])?;
        }
        x if x <= 56 => {
            w.write_all(&[0b1111_1110])?;
            w.write_all(&bs[1..8])?;
        }
        x if x <= 64 => {
            w.write_all(&[0b1111_1111])?;
            w.write_all(&bs[0..8])?;
        }
        x => unreachable!("no encoding for {} bits", x),
    }
    Ok(())
}

pub fn decode_u64(mut r: impl Read) -> io::Result<u64> {
    let mut head = [0u8];
    let mut bs = [0u8; 8];

    r.read_exact(&mut head)?;
    let h = head[0];
    eprintln!("{}", h.reverse_bits().leading_zeros());

    match h {
        x if x <= 0b0111_1111 => {
            bs[7] = 0b0111_1111 & h;
        }
        x if x <= 0b1011_1111 => {
            bs[6] = 0b0011_1111 & h;
            r.read_exact(&mut bs[7..8])?;
        }
        x if x <= 0b1101_1111 => {
            bs[5] = 0b0001_1111 & h;
            r.read_exact(&mut bs[6..8])?;
        }
        x if x <= 0b1110_1111 => {
            bs[4] = 0b0000_1111 & h;
            r.read_exact(&mut bs[5..8])?;
        }
        x if x <= 0b1111_0111 => {
            bs[3] = 0b0000_0111 & h;
            r.read_exact(&mut bs[4..8])?;
        }
        x if x <= 0b1111_1011 => {
            bs[2] = 0b0000_0011 & h;
            r.read_exact(&mut bs[3..8])?;
        }
        x if x <= 0b1111_1101 => {
            bs[1] = 0b0000_0001 & h;
            r.read_exact(&mut bs[2..8])?;
        }
        x if x <= 0b1111_1110 => {
            r.read_exact(&mut bs[1..8])?;
        }
        0b1111_1111 => {
            r.read_exact(&mut bs)?;
        }
        _ => {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
    }

    Ok(u64::from_be_bytes(bs))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode_u64() {
        assert_eq!(&run_encode_u64(0), &[0b0000_0000]);
        assert_eq!(&run_encode_u64(1), &[0b0000_0001]);
        assert_eq!(&run_encode_u64(127), &[0b0111_1111]);
        assert_eq!(&run_encode_u64(128), &[0b1000_0000, 0b1000_0000]);
        assert_eq!(&run_encode_u64(16383), &[0b1011_1111, 0b1111_1111]);
        assert_eq!(
            &run_encode_u64(16384),
            &[0b1100_0000, 0b0100_0000, 0b0000_0000]
        );

        assert_eq!(
            &run_encode_u64(72057594037927935),
            &[
                0b1111_1110,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111
            ]
        );
        assert_eq!(
            &run_encode_u64(72057594037927936),
            &[
                0b1111_1111,
                0b0000_0001,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000,
                0b0000_0000
            ]
        );
    }

    fn run_encode_u64(v: u64) -> Vec<u8> {
        let mut buf = Vec::new();
        encode_u64(&mut buf, v).unwrap();
        buf
    }

    #[test]
    fn test_decode_u64() {
        decode_test_for(0);
        decode_test_for(1);
        decode_test_for(127);
        decode_test_for(128);
        decode_test_for(16383);
        decode_test_for(16384);
        decode_test_for(2097151);
        decode_test_for(2097152);
        decode_test_for(268435455);
        decode_test_for(268435456);
        decode_test_for(34359738367);
        decode_test_for(34359738368);
        decode_test_for(4398046511103);
        decode_test_for(4398046511104);
        decode_test_for(562949953421311);
        decode_test_for(562949953421312);
        decode_test_for(72057594037927935);
        decode_test_for(72057594037927936);
        decode_test_for(18446744073709551615);
    }

    fn decode_test_for(to_be: u64) {
        eprintln!("for {}", to_be);
        let mut buf = Vec::new();
        encode_u64(&mut buf, to_be).expect("encode error");
        let actual = decode_u64(buf.as_slice()).expect("decode error");
        assert_eq!(actual, to_be);
    }
}
