use std::fs::read_dir;
use std::env::args;
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::{Instant, Duration};
use regex::Regex;

fn parse_directory(pattern: Arc<Mutex<Regex>>, directory: String, deposit: Arc<Mutex<Vec<String>>>, thread_tracker: Arc<Mutex<Vec<JoinHandle<()>>>>, active: Arc<Mutex<usize>>, progress: Arc<Mutex<[usize; 2]>>) {
    {
        let mut active = active.lock().unwrap();
        *active += 1;
    }
    if let Ok(contents) = read_dir(&directory) {
        for item in contents {
            if let Ok(item) = item {
                match item.path().is_dir() {
                    true => {
                        {
                            let mut progress = progress.lock().unwrap();
                            progress[0] += 1;
                        }
                        let pattern_clone: Arc<Mutex<Regex>> = Arc::clone(&pattern);
                        let deposit_clone: Arc<Mutex<Vec<String>>> = Arc::clone(&deposit); 
                        let thread_tracker_clone: Arc<Mutex<Vec<JoinHandle<()>>>> = Arc::clone(&thread_tracker);
                        let active_clone: Arc<Mutex<usize>> = Arc::clone(&active);
                        let progress_clone: Arc<Mutex<[usize; 2]>> = Arc::clone(&progress);
                        let mut thread_tracker = thread_tracker.lock().unwrap();
                        thread_tracker.push(spawn(move ||{
                            parse_directory(pattern_clone, item.path().to_string_lossy().to_string(), deposit_clone, thread_tracker_clone, active_clone, progress_clone);
                        }));
                    }
                    false => {
                        let filename: String = item.file_name().to_string_lossy().to_string();
                        {
                            {
                                let mut progress = progress.lock().unwrap();
                                progress[1] += 1;
                            }
                            let pattern = pattern.lock().unwrap();
                            if pattern.is_match(&filename.to_lowercase()) {
                                let mut deposit = deposit.lock().unwrap();
                                deposit.push(item.path().to_string_lossy().to_string()); 
                            }
                        }
                    }
                }
            }
        }
    }
    {
        let mut active = active.lock().unwrap();
        *active -= 1;
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() >= 3 {
        let start_time: Instant = Instant::now();
        let search = &args[2].to_lowercase();
        let pattern: Arc<Mutex<Regex>> = Arc::new(Mutex::new(Regex::new(search).unwrap()));
        let directory: String = String::from(args[1].to_string());
        let deposit: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let deposit_clone: Arc<Mutex<Vec<String>>> = Arc::clone(&deposit);
        let thread_tracker: Arc<Mutex<Vec<JoinHandle<()>>>> = Arc::new(Mutex::new(vec![]));
        let active: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let active_clone: Arc<Mutex<usize>> = Arc::clone(&active);
        let progress: Arc<Mutex<[usize; 2]>> = Arc::new(Mutex::new([0, 0]));
        let progress_clone: Arc<Mutex<[usize; 2]>> = Arc::clone(&progress);
        parse_directory(pattern, directory, deposit_clone, thread_tracker, active_clone, progress_clone);
        loop {
            sleep(Duration::from_millis(10));
            {
                let progress = progress.lock().unwrap();
                let seconds_elapsed = Instant::now().duration_since(start_time).as_secs();
                let items_per_second = (progress[1] as f64 / seconds_elapsed as f64).round();
                print!("\rDirectories Scanned: [{}] | Files Scanned: [{}] | Duration: {:?}s | files/sec: [{}].", progress[0], progress[1], seconds_elapsed, items_per_second);
                let active = active.lock().unwrap();
                if *active == 0 {
                    break;
                }
            }
        }
        println!("\n\nDONE:");
        let deposit = deposit.lock().unwrap();
        for path in deposit.iter() {
            println!("--> {path}");
        }
        println!("--------------------");
    }
    else {
        eprintln!("Expected 3 args -> .exe directory regex_pattern");
    }
}
