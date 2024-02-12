use polars::prelude::*;
fn main() {
    let df = CsvReader::from_path("./sample/test_10.txt")
        .unwrap()
        .with_delimiter(b' ')
        .finish()
        .unwrap();
    let edge_from: Vec<usize> = df[0]
        .i64()
        .unwrap()
        .into_no_null_iter()
        .map(|x: i64| (x - 1) as usize)
        .collect();
    let edge_to: Vec<usize> = df[1]
        .i64()
        .unwrap()
        .into_no_null_iter()
        .map(|x: i64| (x - 1) as usize)
        .collect();
    let n: usize = df[0].name().parse().unwrap();
    let mut edges: Vec<Vec<usize>> = vec![vec![]; n];
    for (a, b) in edge_from.iter().zip(edge_to.iter()) {
        edges[*a].push(*b);
    }
    let topological_order: Vec<usize> = topological_sort(n, &edges);
}

struct Task {
    duration: i32,
    st_time: i32,
    ed_time: i32,
    dependency: Vec<usize>,
}

fn topological_sort(n: usize, edges: &Vec<Vec<usize>>) -> Vec<usize> {
    let mut st: Vec<usize> = Vec::new();
    let mut ans: Vec<usize> = Vec::new();
    let mut h: Vec<usize> = vec![0; n];

    for v in edges.iter() {
        for &u in v.iter() {
            h[u] += 1;
        }
    }

    for i in 0..n {
        if h[i] == 0 {
            st.push(i);
        }
    }

    while let Some(i) = st.pop() {
        ans.push(i);
        for &j in edges[i].iter() {
            h[j] -= 1;
            if h[j] == 0 {
                st.push(j);
            }
        }
    }

    ans
}
