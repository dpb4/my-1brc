# Process

This file will document by optimization process and the various approaches I take.

## v0.0 (124.634s)
This was my first attempt, giving very little thought to optimization. I did it in almost the most naive way possible, though avoiding a few pitfalls (or so I thought).

- The file is parsed line by line (probably slow)
- Each line is split by semicolons, then parsed into an owned `String` and `f32`s
- I keep track of results using a `BTreeMap<String, (f32, f32, f32, usize)>`
  - `BTreeMap` is used because it keeps items sorted by keys automatically (I wanted to avoid sorting them at the end)
  - The values of the map are `(min, sum, max, count)` - the average is calculated only at the very last step

## v1.0 (72.993s)
After looking at a flamegraph of the previous run, I saw that significant amounts of time were being spent on `cmp<u8>`. The cause wasn't immediately obvious, since my code didn't deal with `u8`s at all. It turns out that it was actually checking `String` equality on every `BTreeMap` lookup. In hindsight this should have been obvious; `BTreeMap` does not use hashes, so comparisons betwee `String`s would dominate the time.

- The 1brc data is dominated by lookups, not unique items. There are 1 billion lines to process but only at most 10,000 different cities, so
  - Sorting is basically free (~0.0002 seconds on my machine)
  - Insertion into the map is what dominates the time; comparing strings byte-by-byte is too slow
- The HashMap can be allocated with capacity 10,000 to avoid reallocs (not a huge difference, but non-zero)

### v1.1 (60.259s)
There were a substantial number of `get()` and `set()` calls to the `HashMap`, so after looking through the interface I found the `.entry()` method which allows me to mutate elements instead of reconstructing them, and uses fewer lookups.



