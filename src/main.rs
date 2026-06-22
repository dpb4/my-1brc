use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    time::Instant,
};

fn main() -> std::io::Result<()> {
    let beginning = Instant::now();
    let f = File::open("measurements.txt")?;
    let mut reader = BufReader::new(f);

    let mut map: BTreeMap<String, (f32, f32, f32, usize)> = BTreeMap::new();

    for line in reader.lines() {
        let line2 = line?;
        let mut line_content = line2.split(';');
        let name = line_content.next().unwrap();
        let temp = line_content.next().unwrap().parse::<f32>().unwrap();
        if let Some(t) = map.get(name) {
            map.insert(
                name.to_string(),
                (
                    if t.0 > temp { temp } else { t.0 },
                    t.1 + temp,
                    if t.2 < temp { temp } else { t.2 },
                    t.3 + 1,
                ),
            );
        } else {
            map.insert(name.to_string(), (temp, temp, temp, 1));
        }
    }

    let elapsed = beginning.elapsed();

    for (name, (min, sum, max, count)) in map {
        println!("{}={:.1}/{:.1}/{:.1}", name, min, sum / (count as f32), max);
    }

    println!("TIME TAKEN: {:.3} seconds", elapsed.as_secs_f32());

    Ok(())
}
