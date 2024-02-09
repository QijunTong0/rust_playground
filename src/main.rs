fn main() {
    let vec_of_vec = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
    let inner_vec = &vec_of_vec[0];
    println!("{:?}", inner_vec);
}
