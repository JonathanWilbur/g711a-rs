use std::io;
use hound;

extern "C" {
    fn ALaw_Encode (number: std::os::raw::c_short) -> std::os::raw::c_schar;
    fn ALaw_Decode (number: std::os::raw::c_schar) -> std::os::raw::c_short;
}

pub fn encode(mut sample: i16) -> i8 {
    const ALAW_MAX: i16 = 0x0FFF;
    let mut mask: u16 = 0x0800;
    let mut sign: u8 = 0;
    let mut position: u8 = 11;
    let mut lsb: u8 = 0;
    if sample < 0 {
        sample = sample.overflowing_neg().0;
        sign = 0x80;
    }
    if sample > ALAW_MAX {
		sample = ALAW_MAX;
    }
    while (sample as u16 & mask) != mask && position >= 5 {
        mask >>= 1;
        position -= 1;
    }
    if position == 4 {
        lsb = ((sample >> 1) & 0x0f) as u8;
    } else {
        lsb = ((sample >> (position - 4)) & 0x0f) as u8;
    }
    return ((sign | ((position - 4) << 4) | lsb) ^ 0x55) as i8;
}

pub fn decode (mut sample: i8) -> i16 {
	let mut sign: u8 = 0x00;
	let mut position: u8 = 0;
	let mut decoded: i16 = 0;
	sample ^= 0x55;
	if (sample as u8 & 0x80) > 0 {
		sample &= 0x7F;
		sign = 0x80;
	}
	position = (((sample as u8 & 0xF0) as u8 >> 4) + 4) as u8;
	if position != 4 {
		decoded = ((1 << position) as i16
            | (((sample & 0x0F) as i16) << (position - 4)) as i16
            | (1 << (position - 5))) as i16;
	} else {
		decoded = ((sample as i16) << 1) | 1;
	}
    if sign == 0 {
        return decoded;
    } else {
        return decoded.overflowing_neg().0;
    }
}

fn main() -> io::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    // let mut writer = hound::WavWriter::create("g711a.wav", spec).unwrap();
    let mut rust_writer = hound::WavWriter::create("g711a.wav", spec).unwrap();
    let mut c_writer = hound::WavWriter::create("g711a-c.wav", spec).unwrap();
    let mut reader = hound::WavReader::open("zugzug_3.wav").unwrap();
    let mut sample_index: u32 = 0;
    for read_sample in reader.samples::<i16>() {
        match read_sample {
            Ok(v) => unsafe {
                // We skip every other sample, because source audio is stereo.
                if (sample_index % 2) != 0 {
                    // let write = writer.write_sample(decode(encode(v)));
                    let rust_write = rust_writer.write_sample(decode(encode(v >> 3)));
                    match rust_write {
                        Ok(a) => {},
                        Err(b) => println!("write error: {:?}", b),
                    }
                    let c_write = c_writer.write_sample(ALaw_Decode(ALaw_Encode(v >> 3)));
                    match c_write {
                        Ok(a) => {},
                        Err(b) => println!("write error: {:?}", b),
                    }
                }
            },
            Err(e) => println!("read error: {:?}", e),
        }
        sample_index += 1;
    }
    Ok(())
}