use num_cpus;
use rand::Rng;
use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::{io, thread};
use std::time::Instant;

const CHAR: &str = "しかのこのこのここしたんたん";
const MAX_RATES_COUNT: usize = 10;

fn randomize_char(str: &str, rng: &mut impl Rng) -> String {
    let chars: Vec<char> = str.chars().collect();
    (0..chars.len())
        .map(|_| {
            let idx = rng.gen_range(0..chars.len());
            chars[idx]
        })
        .collect()
}

fn get_char(rng: &mut impl Rng) -> String {
    randomize_char(CHAR, rng)
}

fn format_with_commas(value: usize) -> String {
    let s = value.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

fn main() {
    let num_threads = num_cpus::get();
    let challange_count = Arc::new(AtomicUsize::new(0));
    let target = CHAR.to_string();
    let found = Arc::new(Mutex::new(false));
    let counts_per_second = Arc::new(Mutex::new(Vec::new()));
    let start_time = Instant::now();
    let mut last_count = 0;

    let mut handles = vec![];

    for _ in 0..num_threads {
        let challange_count = Arc::clone(&challange_count);
        let found = Arc::clone(&found);
        let target = target.clone();
        let counts_per_second = Arc::clone(&counts_per_second);
        let start_time = start_time.clone();

        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng(); // スレッド内で乱数生成器を初期化
            let mut last_time = start_time.elapsed(); // 前回の計測時間を記録
            while !*found.lock().unwrap() {
                let char = get_char(&mut rng); // 毎回乱数生成器を再初期化しない
                let count = challange_count.fetch_add(1, Ordering::SeqCst) + 1;

                if char == target {
                    let elapsed_time = start_time.elapsed();
                    let total_count = challange_count.load(Ordering::SeqCst);
                    let probability = 1.0 / (total_count as f64);
                    let mut found_lock = found.lock().unwrap();
                    if !*found_lock {
                        *found_lock = true; // ターゲット文字列が見つかったことをマーク
                        println!("Target found: {}", target); // ターゲット文字列を表示
                        println!("Total attempts: {}", format_with_commas(total_count));
                        println!("Probability: {:.10}", probability);
                        println!("Time elapsed: {:.2} seconds", elapsed_time.as_secs_f64());
                        println!(
                            "Average generation rate: {} per second",
                            format_with_commas(
                                (total_count as f64 / elapsed_time.as_secs_f64()) as usize
                            )
                        );
                    }
                    return;
                } else {
                    if count % 1000000 == 0 {
                        let elapsed = start_time.elapsed();
                        let duration = elapsed - last_time; // 前回からの経過時間
                        last_time = elapsed; // 次回のために現在の時間を保存

                        let rate = 1000000.0 / duration.as_secs_f64(); // 増加量(100万)を経過秒数で割る

                        let mut rates = counts_per_second.lock().unwrap();
                        if rates.len() >= MAX_RATES_COUNT {
                            rates.remove(0); // 古いレートを削除
                        }
                        rates.push(rate);

                        let average_rate: f64 = rates.iter().sum::<f64>() / rates.len() as f64;

                        last_count = count;
                        println!(
                            "{}: {} - {} per/s (平均: {} per/s)",
                            count,
                            char,
                            format_with_commas(rate.round() as usize),
                            format_with_commas(average_rate.round() as usize)
                        );
                    }
                }
            }
        });
        handles.push(handle);
    }

    let mut target_found = false;
    while !target_found {
        if *found.lock().unwrap() {
            target_found = true;
        }
        thread::sleep(std::time::Duration::from_millis(100));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Press any key to exit...");
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}
