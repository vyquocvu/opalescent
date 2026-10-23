fn collatz_steps(start: i64) -> i64 {
    let mut n = start;
    let mut steps = 0;
    while n != 1 {
        if n - (n / 2) * 2 == 0 {
            n /= 2;
        } else {
            n = 3 * n + 1;
        }
        steps += 1;
    }
    steps
}

fn main() {
    let mut total: i64 = 0;
    for i in 1..=300_000 {
        total += collatz_steps(i);
    }
    println!("{total}");
}
