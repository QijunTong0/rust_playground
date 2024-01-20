use rand::{rngs::ThreadRng, Rng};
const NUM: usize = 1000000;
fn main() {
    let mut count = 0u32;
    let mut pool: Vec<f32> = vec![1.0; NUM];
    let mut rng = rand::thread_rng();
    loop {
        count += 1;
        if count % 10000 == 0 {
            println!("{}.Entropy: {}", count, calc_entropy(&pool));
        }
        exchange(&mut pool, &mut rng);
    }
}

fn calc_entropy(pool: &Vec<f32>) -> f32 {
    let mut ent: f32 = 0.0;
    for p in pool {
        if *p > 0.0 {
            let prob = *p / (NUM as f32);
            ent -= prob * prob.log2();
        }
    }
    ent
}

fn exchange(pool: &mut Vec<f32>, rng: &mut ThreadRng) {
    let i: usize = rng.gen::<usize>() % pool.len();
    let j: usize = rng.gen::<usize>() % pool.len();
    if pool[i] > 0.0 {
        pool[i] -= 1.0;
        pool[j] += 1.0;
    }
}
