use std::{collections::HashMap, fs::File, time::Instant};

use memchr::memchr_iter;
use memmap2::Mmap;

fn main() -> std::io::Result<()> {
    let beginning = Instant::now();

    let file = File::open("measurements.txt")?;

    // SAFETY: this could be UB if an external process edits the file or somehow messes with it
    //
    //         assumption: that will not happen
    let mapped_file = unsafe { Mmap::map(&file)? };

    let mut map: HashMap<String, (f32, f32, f32, usize)> = HashMap::with_capacity(10000);

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
                str::from_utf8_unchecked(&temperature_bytes[1..]) // ignore the first char which is ';'
                    .parse()
                    .unwrap_unchecked(),
            )
        };

        if let Some(t) = map.get_mut(city) {
            if temperature < t.0 {
                t.0 = temperature;
            } else if temperature > t.2 {
                t.2 = temperature;
            }
            t.1 += temperature;
            t.3 += 1;
        } else {
            map.insert(city.to_string(), (temperature, temperature, temperature, 1));
        }

        start_byte = end_byte + 1;
    }

    let mut vec = map.into_iter().collect::<Vec<_>>();
    vec.sort_by(|(s1, _), (s2, _)| s1.cmp(s2));

    let elapsed = beginning.elapsed();

    for (name, (min, sum, max, count)) in vec {
        println!("{}={:.1}/{:.1}/{:.1}", name, min, sum / (count as f32), max);
    }

    println!("TIME TAKEN: {:.3} seconds", elapsed.as_secs_f32());

    Ok(())
}
