/// Finds the index of the closest colour in the palette using squared Euclidean
/// distance in RGB space.
pub fn closest_palette_index(palette: &[(u8, u8, u8)], r: u8, g: u8, b: u8) -> u8 {
    let (best_idx, _) = palette
        .iter()
        .enumerate()
        .map(|(i, &(pr, pg, pb))| {
            let dr = (r as i32) - (pr as i32);
            let dg = (g as i32) - (pg as i32);
            let db = (b as i32) - (pb as i32);
            (i, 2 * dr * dr + 4 * dg * dg + 3 * db * db) // dr * dr + dg * dg + db * db)
        })
        .min_by_key(|&(_, dist)| dist)
        .unwrap(); // palette is never empty

    best_idx as u8
}

/// Based upon the description here: https://en.wikipedia.org/wiki/Median_cut
pub fn median_cut(pixels: &[[u8; 3]], depth: u32) -> Vec<(u8, u8, u8)> {
    let mut buckets: Vec<Vec<[u8; 3]>> = vec![pixels.to_vec()];

    for _ in 0..depth {
        let mut next = Vec::with_capacity(buckets.len() * 2);
        for bucket in buckets {
            let (a, b) = split_bucket(bucket);
            next.push(a);
            next.push(b);
        }
        buckets = next;
    }

    buckets
        .iter()
        .filter(|b| !b.is_empty())
        .map(|b| average_color(b))
        .collect()
}

/// Splits a bucket along the channel with the greatest range,
/// cutting at the median value on that channel.
fn split_bucket(mut bucket: Vec<[u8; 3]>) -> (Vec<[u8; 3]>, Vec<[u8; 3]>) {
    if bucket.is_empty() {
        return (vec![], vec![]);
    }

    // Find the channel with the greatest range
    let (mut r_min, mut r_max) = (u8::MAX, u8::MIN);
    let (mut g_min, mut g_max) = (u8::MAX, u8::MIN);
    let (mut b_min, mut b_max) = (u8::MAX, u8::MIN);

    for &[r, g, b] in &bucket {
        r_min = r_min.min(r);
        r_max = r_max.max(r);
        g_min = g_min.min(g);
        g_max = g_max.max(g);
        b_min = b_min.min(b);
        b_max = b_max.max(b);
    }

    let r_range = r_max - r_min;
    let g_range = g_max - g_min;
    let b_range = b_max - b_min;

    let channel = if r_range >= g_range && r_range >= b_range {
        0
    } else if g_range >= b_range {
        1
    } else {
        2
    };

    // Sort by that channel and split at the median
    bucket.sort_unstable_by_key(|p| p[channel]);
    let mid = bucket.len() / 2;
    let right = bucket.split_off(mid);
    (bucket, right)
}

/// Returns the average RGB of all pixels in a bucket.
fn average_color(bucket: &[[u8; 3]]) -> (u8, u8, u8) {
    let n = bucket.len() as u64;
    let (r, g, b) = bucket
        .iter()
        .fold((0u64, 0u64, 0u64), |(ar, ag, ab), &[r, g, b]| {
            (ar + r as u64, ag + g as u64, ab + b as u64)
        });
    ((r / n) as u8, (g / n) as u8, (b / n) as u8)
}
