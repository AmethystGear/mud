use rand::{prelude::StdRng, Rng};

pub fn generate_perlin_noise(
    width: usize,
    height: usize,
    octave_count: u8,
    rng: &mut StdRng,
) -> Vec<f64> {
    let mut white_noise = vec![0f64; (width) * (height)];
    let mut total_noise: Vec<Vec<f64>> = Vec::with_capacity(octave_count as usize);
    let mut p_noise = vec![0f64; (width) * (height)];

    let mut amplitude = 1.0f64;
    let mut total_amplitude = 0.0f64;
    let persistance = 0.5f64;

    for i in 0..((width) * (height)) {
        white_noise[i] = rng.gen::<f64>();
    }
    for i in 0..octave_count {
        let v = perlin_noise(width, height, i, &white_noise);
        total_noise.push(v);
    }
    for i in (0..octave_count as usize).rev() {
        amplitude *= persistance;
        total_amplitude += amplitude;
        for j in 0..((width) * (height)) {
            p_noise[j] = p_noise[j] + total_noise[i][j] * amplitude;
        }
    }
    for i in 0..((width) * (height)) {
        p_noise[i] /= total_amplitude;
    }
    return p_noise;
}

fn perlin_noise(width: usize, height: usize, octave: u8, white_noise: &Vec<f64>) -> Vec<f64> {
    let mut result = vec![0f64; (width) * (height)];
    let sample_period: usize = 1 << (octave as usize);
    let sample_frequency = 1.0f64 / sample_period as f64;

    for j in 0..(height) {
        let y1: usize = (j / sample_period) * sample_period;
        let y2: usize = (y1 + sample_period) % height;
        let y_blend = (j - y1) as f64 * sample_frequency;
        for i in 0..(width) {
            let x1: usize = (i / sample_period) * sample_period;
            let x2: usize = (x1 + sample_period) % width;
            let x_blend = (i - x1) as f64 * sample_frequency;

            let top = lerp(
                get(white_noise, width, x1, y1),
                get(white_noise, width, x2, y1),
                x_blend,
            );
            let bottom = lerp(
                get(white_noise, width, x1, y2),
                get(white_noise, width, x2, y2),
                x_blend,
            );
            set(&mut result, width, i, j, lerp(top, bottom, y_blend));
        }
    }
    return result;
}

fn get(a: &Vec<f64>, w: usize, x: usize, y: usize) -> f64 {
    return a[y * w + x];
}

fn set(a: &mut Vec<f64>, w: usize, x: usize, y: usize, val: f64) {
    a[y * w + x] = val;
}

fn lerp(a: f64, b: f64, blend: f64) -> f64 {
    return a * (1.0f64 - blend) + b * blend;
}
