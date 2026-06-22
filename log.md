# Process

This file will document by optimization process and the various approaches I take.

## v0.0 (124.634s)
This was my first attempt, giving very little thought to optimization. I did it in almost the most naive way possible, though avoiding a few pitfalls (or so I thought).

- The file is parsed line by line (probably slow)
- Each line is split by semicolons, then parsed into an owned String and f32s
- I keep track of results using a BTreeMap<String, (f32, f32, f32, usize)>
  - BTreeMap is used because it keeps items sorted by keys automatically (I wanted to avoid sorting them at the end)
  - The values of the map are (min, sum, max, count) - the average is calculated only at the very last step

  
