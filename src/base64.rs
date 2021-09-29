const ENCODE_TABLE: [u8; 64] = [
	b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H',
	b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
	b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X',
	b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
	b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n',
	b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
	b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3',
	b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];

#[inline]
pub const fn bound(n: usize) -> usize {
	return ((n * 4 / 3) + 3) & !3;
}
	
pub fn encode(dst: &mut [u8], src: &[u8]) {
	let mut i = 0;
	let mut j = 0;

	loop {
		if src.len() - i < 3 {
			break;
		}

		let t0 = src[i + 0];
		let t1 = src[i + 1];
		let t2 = src[i + 2];

		let k0 = t0 >> 2;
		let k1 = ((t0 & 0x3) << 4) | t1 >> 4;
		let k2 = ((t1 & 0xF) << 2) | t2 >> 6;
		let k3 = t2 & 0x3F;

		dst[j] = ENCODE_TABLE[k0 as usize];
		dst[j + 1] = ENCODE_TABLE[k1 as usize];
		dst[j + 2] = ENCODE_TABLE[k2 as usize];
		dst[j + 3] = ENCODE_TABLE[k3 as usize];

		i += 3;
		j += 4;
	}

	match src.len() - i {
		0 => { },
		1 => {
			let t0 = src[i + 0];

			let k0 = t0 >> 2;
			let k1 = (t0 & 0x3) << 4;

			dst[j] = ENCODE_TABLE[k0 as usize];
			dst[j + 1] = ENCODE_TABLE[k1 as usize];
			dst[j + 2] = b'=';
			dst[j + 3] = b'=';
		},
		2 => {
			let t0 = src[i + 0];
			let t1 = src[i + 1];

			let k0 = t0 >> 2;
			let k1 = ((t0 & 0x3) << 4) | t1 >> 4;
			let k2 = (t1 & 0xF) << 2;
	
			dst[j] = ENCODE_TABLE[k0 as usize];
			dst[j + 1] = ENCODE_TABLE[k1 as usize];
			dst[j + 2] = ENCODE_TABLE[k2 as usize];
			dst[j + 3] = b'=';
		},
		_ => unreachable!(),
	};
}

