fn main() {
    let nums: Vec<i32> = vec![1, 2, 3, 4];
    for x in nums.iter() {
        println!("{}", x * 2);
    }
    nums.iter().for_each(|x: &i32| println!("{}", x));
    let count: usize = nums.iter().map(|x: &i32| x + 10).count();
    let sum: i32 = nums.iter().map(|x: &i32| x + 10).sum();
    println!("{},{}", count, sum);
    let prod = (1..10).fold(1, |reduction, x| reduction * x);
    println!("{}", prod);
    let nums2: Vec<&i32> = nums.iter().filter(|x: &&i32| *x % 2 == 0).collect();
}
