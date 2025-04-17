use crossbeam_channel::{Sender, bounded};
use digest::Digest;
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::sync::Arc;
use std::thread;
use std::env;
use std::process;

const CHUNK_SIZE: usize = 512 * 1024 * 1024;
const CHANNEL_CAPACITY: usize = 2;

#[derive(Debug)]
enum HashResult {
    MD5(Vec<u8>),
    SHA1(Vec<u8>),
    SHA256(Vec<u8>),
    SHA512(Vec<u8>),
}

fn hash_worker<D: Digest + Send + 'static>(
    data_rx: crossbeam_channel::Receiver<Vec<u8>>,
    result_tx: Arc<Sender<HashResult>>,
    hash_type: &str,
) where
    D: Default,
{
    let mut hasher = D::default();
    while let Ok(chunk) = data_rx.recv() {
        if chunk.is_empty() {
            break;
        }
        hasher.update(&chunk);
    }

    let result = hasher.finalize().to_vec();
    let hash_result = match hash_type {
        "md5" => HashResult::MD5(result),
        "sha1" => HashResult::SHA1(result),
        "sha256" => HashResult::SHA256(result),
        "sha512" => HashResult::SHA512(result),
        _ => unreachable!(),
    };
    result_tx.send(hash_result).unwrap();
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        process::exit(1);
    }

    let file = File::open(&args[1])?;
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
    let mut buffer = vec![0; CHUNK_SIZE];

    let (md5_tx, md5_rx) = bounded(CHANNEL_CAPACITY);
    let (sha1_tx, sha1_rx) = bounded(CHANNEL_CAPACITY);
    let (sha256_tx, sha256_rx) = bounded(CHANNEL_CAPACITY);
    let (sha512_tx, sha512_rx) = bounded(CHANNEL_CAPACITY);
    let (result_tx, result_rx) = bounded(4);

    // 启动工作线程
    let result_tx = Arc::new(result_tx);
    let threads = vec![
        thread::spawn({
            let result_tx = Arc::clone(&result_tx);
            move || hash_worker::<Md5>(md5_rx, result_tx, "md5")
        }),
        thread::spawn({
            let result_tx = Arc::clone(&result_tx);
            move || hash_worker::<Sha1>(sha1_rx, result_tx, "sha1")
        }),
        thread::spawn({
            let result_tx = Arc::clone(&result_tx);
            move || hash_worker::<Sha256>(sha256_rx, result_tx, "sha256")
        }),
        thread::spawn({
            let result_tx = Arc::clone(&result_tx);
            move || hash_worker::<Sha512>(sha512_rx, result_tx, "sha512")
        }),
    ];

    let senders = [&md5_tx, &sha1_tx, &sha256_tx, &sha512_tx];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = buffer[..bytes_read].to_vec();
        for sender in &senders {
            sender.send(chunk.clone()).unwrap();
        }
    }

    for sender in &senders {
        sender.send(Vec::new()).unwrap();
    }

    for thread in threads {
        thread.join().unwrap();
    }

    let mut results = Vec::new();
    for _ in 0..4 {
        results.push(result_rx.recv().unwrap());
    }

    for result in results {
        match result {
            HashResult::MD5(hash) => println!("MD5:     {}", hex::encode(hash)),
            HashResult::SHA1(hash) => println!("SHA1:    {}", hex::encode(hash)),
            HashResult::SHA256(hash) => println!("SHA256:  {}", hex::encode(hash)),
            HashResult::SHA512(hash) => println!("SHA512:  {}", hex::encode(hash)),
        }
    }

    Ok(())
}
