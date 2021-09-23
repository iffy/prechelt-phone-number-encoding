use std::collections::HashMap;
use std::env::args;
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::io::{self, BufRead, BufWriter, StdoutLock, Write};
use std::path::Path;

type Dictionary = HashMap<Vec<u8>, Vec<String>, ahash::RandomState>;

#[derive(Debug, Copy, Clone)]
enum WordOrDigit<'a> {
    Word(&'a str),
    Digit(u8),
}

impl<'a> WordOrDigit<'a> {
    fn len(&self) -> usize {
        if let WordOrDigit::Word(w) = self { w.len() } else { 1 }
    }
}

impl Display for WordOrDigit<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WordOrDigit::Word(s) => s.fmt(formatter),
            WordOrDigit::Digit(d) => d.fmt(formatter),
        }
    }
}


/// Port of Peter Norvig's Lisp solution to the Prechelt phone-encoding problem.
///
/// Even though this is intended as a port, it deviates quite a bit from it
/// due to the very different natures of Lisp and Rust.
fn main() -> io::Result<()> {
    // drop itself from args
    let mut args: Vec<_> = args().skip(1).collect();
    let words_file: String = if !args.is_empty() { args.remove(0) } else { "tests/words.txt".into() };
    let input_file: String = if !args.is_empty() { args.remove(0) } else { "tests/numbers.txt".into() };

    let mut solution_count = 0;
    let mut rejected_solution_count: u64 = 0;
    let dict = load_dict(words_file)?;

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    for line in read_lines(input_file)? {
        let num = line?;
        let digits: Vec<u8> = num.chars()
            .filter(char::is_ascii_digit)
            .map(|ch| ch as u8)
            .collect();
        print_translations(&mut solution_count, &mut rejected_solution_count, &num, &digits, &mut Vec::new(), &dict, &mut writer);
    }
    eprintln!("Found solutions: {}, rejected: {}", solution_count, rejected_solution_count);
    Ok(())
}

fn print_translations<'a>(
    solution_count: &mut usize,
    rejected_solution_count: &mut u64,
    num: &str,
    digits: &[u8],
    words: &mut Vec<WordOrDigit<'a>>,
    dict: &'a Dictionary,
    writer: &mut BufWriter<StdoutLock>,
) {
    if digits.is_empty() {
        if print_solution(num, &words, writer) {
            *solution_count += 1;
        } else {
            *rejected_solution_count += 1;
        }
        return;
    }
    let mut found_word = false;
    for i in 0..digits.len() {
        let (key, rest_of_digits) = digits.split_at(i + 1);
        if let Some(found_words) = dict.get(key) {
            for word in found_words {
                found_word = true;
                words.push(WordOrDigit::Word(word));
                print_translations(solution_count, rejected_solution_count, num, rest_of_digits, words, dict, writer);
                words.pop();
            }
        }
    }
    if found_word {
        return;
    }
    let last_is_digit = match words.last() {
        Some(WordOrDigit::Digit(..)) => true,
        _ => false,
    };
    if !last_is_digit {
        let digit = digits[0] - b'0';
        words.push(WordOrDigit::Digit(digit));
        print_translations(solution_count, rejected_solution_count, num, &digits[1..], words, dict, writer);
        words.pop();
    }
}

fn print_solution(
    num: &str,
    words: &[WordOrDigit<'_>],
    writer: &mut BufWriter<StdoutLock>,
) -> bool {
    if !should_print(words) { return false; }
    // do a little gymnastics here to avoid allocating a big string just for printing it
    write!(writer, "{}", num).unwrap();
    if words.is_empty() {
        write!(writer, ":").unwrap();
        return true;
    }
    write!(writer, ": ").unwrap();
    let (head, tail) = words.split_at(words.len() - 1);
    for word in head {
        write!(writer, "{} ", word).unwrap();
    }
    for word in tail { // only last word in tail
        write!(writer, "{}", word).unwrap();
    }
    write!(writer, "\n").unwrap();
    true
}

fn should_print(words: &[WordOrDigit]) -> bool {
    if words.is_empty() { return false; }
    if words.len() == 1 { return true; }
    let word = words[0];
    let (mut was_digit, mut acceptable_len) = match word {
        WordOrDigit::Word(w) => (false, w.len()),
        WordOrDigit::Digit(_) => (true, 0)
    };
    for word in words[1..].iter() {
        if acceptable_len == 0 {
            acceptable_len = word.len();
        }
        let is_digit = matches!(word, WordOrDigit::Digit(..));
        if was_digit == is_digit ||
            (!is_digit && word.len() != acceptable_len) {
            return false;
        }
        was_digit = is_digit;
    }
    true
}

fn load_dict(words_file: String) -> io::Result<Dictionary> {
    let mut dict: Dictionary = HashMap::with_capacity_and_hasher(
        100,
        ahash::RandomState::default());

    for line in read_lines(words_file)? {
        let word = line?;
        let key = word_to_number(&word);
        let words = dict.entry(key).or_default();
        words.push(word);
    }
    Ok(dict)
}

// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn word_to_number(word: &str) -> Vec<u8> {
    word.chars()
        .filter(char::is_ascii_alphabetic)
        .map(char_to_digit)
        .map(|d| d + b'0')
        .collect()
}

fn char_to_digit(ch: char) -> u8 {
    match ch.to_ascii_lowercase() {
        'e' => 0,
        'j' | 'n' | 'q' => 1,
        'r' | 'w' | 'x' => 2,
        'd' | 's' | 'y' => 3,
        'f' | 't' => 4,
        'a' | 'm' => 5,
        'c' | 'i' | 'v' => 6,
        'b' | 'k' | 'u' => 7,
        'l' | 'o' | 'p' => 8,
        'g' | 'h' | 'z' => 9,
        _ => panic!("invalid input: not a digit: {}", ch)
    }
}
