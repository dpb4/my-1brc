use std::{fs::File, time::Instant};

use ahash::AHashMap;
use memchr::memchr_iter;
use memmap2::Mmap;

mod float_lookup;

fn main() -> std::io::Result<()> {
    let beginning = Instant::now();

    let file = File::open("measurements.txt")?;

    // SAFETY: this could be UB if an external process edits the file or somehow messes with it
    //
    //         assumption: that will not happen
    let mapped_file = unsafe { Mmap::map(&file)? };

    let mut map: AHashMap<String, (i32, i32, i32, usize)> = AHashMap::with_capacity(10000);

    let mut start_byte = 0;

    for end_byte in memchr_iter(b'\n', &mapped_file) {
        let line_bytes = &mapped_file[start_byte..end_byte];
        let semicolon = memchr_iter(b';', line_bytes).next().unwrap();

        // SAFETY: if semicolon was not a valid offset into line_bytes, could be UB
        //         if the line contained invalid utf-8, it would not be caught
        //         if the float was invalid, this would be UB
        //
        //         safe: every input line is guaranteed to have 1 semicolon followed by a float,
        //               and every line is valid utf-8
        let (city, temperature) = unsafe {
            let (city_bytes, temperature_bytes) = line_bytes.split_at_unchecked(semicolon);
            (
                str::from_utf8_unchecked(city_bytes),
                parse_temperature_as_int(&temperature_bytes[1..]),
                // float_lookup::float_lookup(&temperature_bytes[1..]), // ignore the first char which is ';'
            )
        };

        if let Some(t) = map.get_mut(city) {
            if temperature < t.0 {
                t.0 = temperature;
            } else if temperature > t.2 {
                t.2 = temperature;
            }
            t.1 += temperature as i32;
            t.3 += 1;
        } else {
            map.insert(
                city.to_string(),
                (temperature, temperature as i32, temperature, 1),
            );
        }

        start_byte = end_byte + 1;
    }

    let mut vec = map.into_iter().collect::<Vec<_>>();
    vec.sort_by(|(s1, _), (s2, _)| s1.cmp(s2));
    let vec = vec.iter_mut().map(|(s, (min, sum, max, count))| {
        (
            s,
            (
                *min as f32 * 0.1,
                (*sum as f32) / (*count as f32) * 0.1,
                *max as f32 * 0.1,
            ),
        )
    });

    let elapsed = beginning.elapsed();

    for (name, (min, mean, max)) in vec {
        println!("{}={:.1}/{:.1}/{:.1}", name, min, mean, max);
    }

    println!("TIME TAKEN: {:.3} seconds", elapsed.as_secs_f32());

    Ok(())
}

#[inline(always)]
fn parse_temperature_as_int(mut bytes: &[u8]) -> i32 {
    let mut neg = false;
    if bytes[0] == b'-' {
        neg = true;
        bytes = &bytes[1..]
    }
    let len = bytes.len();

    let mut parsed = bytes[len - 1] as i32 + (bytes[len - 3] as i32) * 10;
    if len > 3 {
        parsed += bytes[len - 4] as i32 * 100;
    }

    if neg { -parsed } else { parsed }
}
// #[inline(always)]
// #[rustfmt::skip]
// fn parse_temperature(bytes: &[u8]) -> f32 {
//     match *bytes {
//         // one digit
//                  [o, _, d] =>   ( 1.0 * u8_to_f32(o)) +
//                                 ( 0.1 * u8_to_f32(d)),
//         // negative one digit
//         [b'-',    o, _, d] => -(( 1.0 * u8_to_f32(o)) +
//                                 ( 0.1 * u8_to_f32(d))),
//         // two digit
//               [t, o, _, d] =>   (10.0 * u8_to_f32(t)) +
//                                 ( 1.0 * u8_to_f32(o)) +
//                                 ( 0.1 * u8_to_f32(d)),
//         // negative two digit
//         [b'-', t, o, _, d] => -((10.0 * u8_to_f32(t)) +
//                                 ( 1.0 * u8_to_f32(o)) +
//                                 ( 0.1 * u8_to_f32(d))),
//         _ => unreachable!(),
//     }
// }

// #[inline(always)]
// fn u8_to_f32(byte: u8) -> f32 {
//     (byte - b'0') as f32
// }
