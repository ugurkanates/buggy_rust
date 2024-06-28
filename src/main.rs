use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::collections::HashSet;
use std::io::{self, Write};

#[derive(Clone, Debug)]
struct Word {
    data: String,
    count: i32,
}

impl Word {
    fn new(data: String) -> Self {
        Word { data, count: 1 }
    }

    fn set_data(&mut self, data: String) {
        self.data = data;
    }

    fn get_data(&self) -> &str {
        &self.data
    }

    fn set_count(&mut self, count: i32) {
        self.count = count;
    }

    fn get_count(&self) -> i32 {
        self.count
    }
}

fn worker_thread(words: Arc<Mutex<Vec<Word>>>, word: Arc<Mutex<Word>>, cv: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*cv;
    let mut end_encountered = false;
    while !end_encountered {
        let mut g_ready = lock.lock().unwrap();
        while !*g_ready {
            g_ready = cvar.wait(g_ready).unwrap();
        }
        let mut w = word.lock().unwrap().clone();
        end_encountered = w.get_data() == "end";
        if !end_encountered {
            let mut words = words.lock().unwrap();
            if let Some(existing) = words.iter_mut().find(|word| word.get_data() == w.get_data()) {
                existing.set_count(existing.get_count() + 1);
            } else {
                words.push(w);
            }
        }
        *g_ready = false;
        cvar.notify_one();
    }
}

fn producer_thread(word: Arc<Mutex<Word>>, cv: Arc<(Mutex<bool>, Condvar)>) {
    let (lock, cvar) = &*cv;
    loop {
        let mut input = String::new();
        print!("Enter a word: ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim().to_string();
        if input == "end" {
            break;
        }
        {
            let mut w = word.lock().unwrap();
            w.set_data(input);
        }
        let mut g_ready = lock.lock().unwrap();
        *g_ready = true;
        cvar.notify_one();
        while *g_ready {
            g_ready = cvar.wait(g_ready).unwrap();
        }
    }
}

fn main() {
    let words = Arc::new(Mutex::new(Vec::new()));
    let word = Arc::new(Mutex::new(Word::new("".to_string())));
    let cv = Arc::new((Mutex::new(false), Condvar::new()));

    let words_clone = Arc::clone(&words);
    let word_clone = Arc::clone(&word);
    let cv_clone = Arc::clone(&cv);

    let worker = thread::spawn(move || {
        worker_thread(words_clone, word_clone, cv_clone);
    });

    let producer = thread::spawn(move || {
        producer_thread(word, cv);
    });

    worker.join().unwrap();
    producer.join().unwrap();

    let words = words.lock().unwrap();
    for word in words.iter() {
        println!("{}: {}", word.get_data(), word.get_count());
    }
}