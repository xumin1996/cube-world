use simdnoise::*;

fn main() {
    let (noise, min, max) = NoiseBuilder::fbm_2d_offset(1f32, 2, 0f32, 2)
        .with_seed(1)
        .generate();

        let plain_height: Vec<Vec<f32>> = noise.chunks(2)
            .map(|chunk| {
                let mut rv = chunk.to_vec();
                rv
            })
            .collect();

    println!("{:?}", plain_height);
    
}
