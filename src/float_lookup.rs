static LOOKUP: [f32; 1000] = {
    let mut table = [0.0; 1000];
    let mut i = 0;
    while i < 1000 {
        table[i] = i as f32 / 10.0;
        i += 1;
    }
    table
};

// pub fn float_lookup(bytes: &[u8]) -> f32 {
//     let mut neg = false;
//     let idx = if bytes.len() == 3 {
//         ((bytes[0] - b'0') as usize) * 10 + ((bytes[2] - b'0') as usize)
//     } else if bytes.len() == 4 {
//         if bytes[0] == b'-' {
//             neg = true;
//             ((bytes[1] - b'0') as usize) * 10 + ((bytes[3] - b'0') as usize)
//         } else {
//             ((bytes[0] - b'0') as usize) * 100
//                 + ((bytes[1] - b'0') as usize) * 10
//                 + ((bytes[3] - b'0') as usize)
//         }
//     } else {
//         if bytes[0] == b'-' {
//             neg = true;
//             ((bytes[1] - b'0') as usize) * 100
//                 + ((bytes[2] - b'0') as usize) * 10
//                 + ((bytes[4] - b'0') as usize)
//         } else {
//             ((bytes[0] - b'0') as usize) * 100
//                 + ((bytes[1] - b'0') as usize) * 10
//                 + ((bytes[3] - b'0') as usize)
//         }
//     };

//     if neg { -LOOKUP[idx] } else { LOOKUP[idx] }
// }

#[inline(always)]
pub fn float_lookup(mut bytes: &[u8]) -> f32 {
    let mut neg = false;
    if bytes[0] == b'-' {
        neg = true;
        bytes = &bytes[1..];
    }
    let idx = if bytes.len() == 3 {
        ((bytes[0] - b'0') as usize) * 10 + ((bytes[2] - b'0') as usize)
    } else {
        ((bytes[0] - b'0') as usize) * 100
            + ((bytes[1] - b'0') as usize) * 10
            + ((bytes[3] - b'0') as usize)
    };

    if neg { -LOOKUP[idx] } else { LOOKUP[idx] }
}
