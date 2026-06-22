use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    time::Instant,
};

fn main() -> std::io::Result<()> {
    let beginning = Instant::now();
    let f = File::open("measurements.txt")?;
    let reader = BufReader::new(f);

    let mut map: HashMap<String, (f32, f32, f32, usize)> = HashMap::with_capacity(10000);

    for line in reader.lines() {
        let line2 = line?;
        let mut line_content = line2.split(';');
        let name = line_content.next().unwrap().to_string();
        let temp = line_content.next().unwrap().parse::<f32>().unwrap();

        map.entry(name)
            .and_modify(|t| {
                if temp < t.0 {
                    t.0 = temp;
                } else if temp > t.2 {
                    t.2 = temp;
                }
                t.1 += temp;
                t.3 += 1;
            })
            .or_insert((temp, temp, temp, 1));
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
