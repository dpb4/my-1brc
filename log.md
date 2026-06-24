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

## V2.0 (29.381s) -- getting `unsafe`
The primary bottleneck was file loading; *1 billion* lines need to be read and they're only a handful of characters each. That means that a ton of time is spent reading from disk and working with the `Lines<_>` iterator that rust provides through the `BufReader::read_lines()` function. There are a few options to deal with this:
1) Use only a single disk read and read the whole file into RAM at once
-> Unreasonable, doesn't scale
2) Load the file in large chunks and do the line processing myself
-> This could work, but care must be taken around line boundaries, and even worse, utf-8 boundaries. This would need a custom implementation. Not a bad option, but not the simplest
3) Use memory mapped IO through rust's `memmap2` crate
I won't pretend to understand how it works on the backend, but this crate *feels* like doing option 1, except it scales. It really is magical. 

#### Using `memmap2`
`memmap2` interfaces with the kernel to give the CPU access to a file on disk as if it were just RAM, and manages the file loading itself internally. Because its such a low-level interface however, the data is only ever returned as raw bytes. Because the kernel is responsible for providing data from the file, nothing is stopping another process from modifying the file you're trying to read from. That makes the `memmap2` interface fundamentally unsafe. Furthermore, everything is returned as raw bytes, meaning that some unsafe shenanigans are likely necessary for type conversions. If performance is a concern (which it is), using unsafe code allows for further optimizations by skipping things like utf-8 verification and bounds checking.

#### Using `memchr`
Even after reading lines from the file, each line needed to be parsed to extract the city name and reported temperature. I was using built in str splitting functions, but that was too slow, and overcomplicated for my needs. The solution is the `memchr` crate. It is very limited in functionality, but it makes up for that with its excellent performance. It basically just searches through `[u8]`s for a given byte. That's about 90% of the functionality. But it is *hyper* optimized, and it can search extremely quickly. This is perfect because, since getting rid of the calls to `read_lines()`, the input data is no longer automatically split into lines, so searching for `\n` characters is a requirement.

- File loading and line splitting is super fast now
- The time is now dominated by hashmap lookups (the hashing part) and `f32` parsing

### V2.1 (19.903s)

- Rust's `ahash` crate purports to be the fastest hashmap in rust, specifically having fast String hashing (which I need) so I swapped that in in place of the standard `HashMap`

#### Float parsing
Float parsing is a harder problem than it seems. Imagine trying to parse a float the naive way, something like this:
`"12.34" -> (1 * 10.0) + (2 * 1.0) + (3 * 0.1) + (4 * 0.01) = 12.34_f32`
This actually doesn't work, at least not in the general case. The issue isn't clear with this example, but as the number has more and more significant digits, sometimes there is a binary encoding which is *closer* to the intended mathematical value than the one you'd get by adding the digits.

In our case however, we don't need to parse arbitrary floating point values. In fact the numbers we parse are a very small subset of floats, ranging from about -40.0 to 40.0, and all with exactly 1 decimal place. Doing full blown float parsing (like `str::parse::<f32>()` does) is way overkill for our code, so we can optimize this significantly.

My first approach was to implement the naive algorithm I showed above, just going across each digit. This is alright, but looping over every digit and accumulating a value still has too much overhead. Since we know that every value either has 2 or 3 digits (and possibly a negative sign) we can unroll this to speed things up further. Just using an if statement over the length of the bytes means we can split it into two linear branches.

#### Avoiding (float) arithmetic

Since the values used really are so limited (only about 500 unique values, ignoring negatives) I decided to try a lookup table. This is the kind of optimization which is highly situational, and really needs to be benchmarked. Between a couple floating point multiplications vs a couple integer multiplications then a memory read, theres no clear winner. Thankfully in my case it did make a difference. By treating every decimal as an index (eg. `10.5 -> FLOATS\[105\]`) I could immediately find exact floating point representations.

These optimizations together brought the time down to 19.903 seconds, which is a good improvement. Now the float parsing only takes 5-10% of the total time (as opposed to ~30% before).
