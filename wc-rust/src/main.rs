use std::fs;
use std::env;
use std::thread;
use std::sync::mpsc;
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::io::prelude::*;

fn count_lines(path: &Path) -> Result<(usize, PathBuf), std::io::Error> {
    let file = fs::File::open(path)?;
    let buf_reader = BufReader::new(file);
    let lines = buf_reader.lines().count();
    Ok((lines, path.to_path_buf()))
}

fn list_dir(path: PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut output = Vec::new();
    for entry in fs::read_dir(path)? {
        let dir = entry?;
        let entry_path = dir.path();
        let metadata = fs::metadata(entry_path)?;
        if metadata.is_file() {
            output.push(dir.path());
        }
    }
    Ok(output)
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let path = match args.len() {
        1 => Path::new("./"),
        _ => Path::new(&args[1])
    };

    let metadata = fs::metadata(path)?;

    let mut total_lines = 0;

    let mut line_count: Vec<(usize, PathBuf)> = Vec::new();

    let paths: Vec<PathBuf> = match metadata.is_dir() {
        true => {
            list_dir(path.to_path_buf())?
        }
        false => {
            vec![path.to_path_buf()]
        }
    };

    let (tx, rx) = mpsc::channel();

    let path_count = paths.len();

    for path in paths {
        let tx1 = mpsc::Sender::clone(&tx);
        let _join_handle = thread::spawn(move || {
            let count_result = count_lines(&path);
            tx1.send(count_result).unwrap();
        });
    }

    let mut num_received = 0;
    while num_received < path_count {
        let received = rx.recv().unwrap();
        if let Ok(count) = received {
            let (num_lines, _) = count;
            total_lines += num_lines;
            line_count.push(count);
        }
        num_received += 1;
    }

    line_count.sort_unstable_by(|a, b| b.0.cmp(&a.0));

    for lines in line_count {
        println!("{:>10} {}", lines.0, lines.1.as_path().to_string_lossy());
    }
    println!("{:>10} {}", total_lines, "[TOTAL]");
    Ok(())
}
