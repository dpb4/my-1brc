use std::{fs::File, time::Instant};

use ahash::AHashMap;
use memchr::memchr_iter;
use memmap2::Mmap;
use rayon::prelude::*;

const MB: usize = 1024 * 1024;
const CHUNK_SIZE: usize = 32 * MB;

fn main() -> std::io::Result<()> {
    let beginning = Instant::now();

    let file = File::open("measurements.txt")?;

    // SAFETY: this could be UB if an external process edits the file or somehow messes with it
    //
    //         assumption: that will not happen
    let mapped_file = unsafe { Mmap::map(&file)? };
    // let _ = mapped_file.advise(memmap2::Advice::Sequential);

    let map = (0..mapped_file.len())
        .step_by(CHUNK_SIZE)
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|start| {
            // this will ensure every chunk starts and ends on line boundaries
            let end = adjust_end(&mapped_file, start + CHUNK_SIZE);
            let start = adjust_start(&mapped_file, start);
            process_chunk(&mapped_file[start..end])
        })
        .reduce(
            || AHashMap::new(),
            |mut m1, m2| {
                merge_maps(&mut m1, m2);
                m1
            },
        );

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

    let mut parsed = (bytes[len - 1] - b'0') as i32 + ((bytes[len - 3] - b'0') as i32) * 10;
    if len > 3 {
        parsed += (bytes[len - 4] - b'0') as i32 * 100;
    }

    if neg { -parsed } else { parsed }
}

fn adjust_start(bytes: &[u8], start: usize) -> usize {
    if start == 0 {
        return 0;
    }

    if bytes[start - 1] == b'\n' {
        return start;
    }

    match memchr::memchr(b'\n', &bytes[start..]) {
        Some(i) => start + i + 1,
        None => bytes.len(), // no complete lines remain
    }
}

fn adjust_end(bytes: &[u8], end: usize) -> usize {
    if end >= bytes.len() {
        return bytes.len();
    }

    match memchr::memchr(b'\n', &bytes[end..]) {
        Some(i) => end + i,
        None => bytes.len(),
    }
}

fn process_chunk(chunk: &[u8]) -> AHashMap<String, (i32, i32, i32, usize)> {
    let mut map: AHashMap<String, (i32, i32, i32, usize)> = AHashMap::new();

    let mut start_byte = 0;

    for end_byte in memchr_iter(b'\n', chunk) {
        let line_bytes = &chunk[start_byte..end_byte];

        let semicolon = {
            let len = line_bytes.len();
            if line_bytes[len - 4] == b';' {
                len - 4
            } else if line_bytes[len - 5] == b';' {
                len - 5
            } else if line_bytes[len - 6] == b';' {
                len - 6
            } else {
                panic!("bad semicolon :(")
            }
        };

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
    map
}

fn merge_maps(
    m1: &mut AHashMap<String, (i32, i32, i32, usize)>,
    m2: AHashMap<String, (i32, i32, i32, usize)>,
) {
    for (city, vals) in m2.into_iter() {
        if let Some(m1_temp) = m1.get_mut(&city) {
            if vals.0 < m1_temp.0 {
                m1_temp.0 = vals.0;
            } else if vals.2 > m1_temp.2 {
                m1_temp.2 = vals.2;
            }
            m1_temp.1 += vals.1;
            m1_temp.3 += vals.3;
        } else {
            m1.insert(city, vals);
        }
    }
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
