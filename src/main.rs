#![allow(dead_code)]
use std::{collections::{HashMap, HashSet}, fs::File, io::{BufRead, BufReader, Read}, path::Path};

use rust_stemmers::{Algorithm, Stemmer};
use clap::{builder::Str, Parser};
use colored::Colorize;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

struct Entry {
    amount: usize,
    contained_in: HashSet<String>,
}

struct Dict {
    hashmap: HashMap<String, Entry>,
}

impl Dict {
    pub fn new() -> Self {
        Self { hashmap: HashMap::new() }
    }

    pub fn add(&mut self, word: String, fname: String) -> usize {
        if self.hashmap.contains_key(&word) {
            let ptr = self.hashmap.get_mut(&word).unwrap();
            ptr.amount += 1;
            ptr.contained_in.insert(fname.to_string());

            return ptr.amount;
        } 
        else {
            let entry = Entry { amount: 1, contained_in: HashSet::from([fname.to_string()]) };
            self.hashmap.insert(word, entry);

            return 1;
        } 
    }

    pub fn sort(self, words: usize, filenum: usize, length: usize) -> Vec<(String, usize, HashSet<String>)> {
        let mut v = self.hashmap.iter()
                    .map(|(word, entry)| (word.clone(), entry.amount, entry.contained_in.clone()) )
                    .filter(|(w, n, f)| 
                            f.len() > filenum && 
                            *n >= words &&
                            &w[..].graphemes(true).count() > &length)
                    .collect::<Vec<(String, usize, HashSet<String>)>>();

        v.sort_by(|a, b| b.1.cmp(&a.1));

        v
    }
}

fn stem_and_compare(stemmer: &Stemmer, str1: &str, strvec: &Vec<String>) -> bool {
    strvec.iter().map(|word| stemmer.stem(word)).filter(|word| word == &stemmer.stem(str1)).count() > 0
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Наименьшее число повторов слова
    #[arg(short, long, default_value_t = 2)]
    words: usize,
    /// Наименьшее число файлов, содержащих слово
    #[arg(short, long, default_value_t = 2)]
    filenum: usize,
    /// Наименьшая длина слова
    #[arg(short, long, default_value_t = 6)]
    length: usize,
    /// Слова, которые следует исключить из выдачи
    #[arg(short, long)]
    exclude: Option<Vec<String>>,
    /// Файл со списком слов для исключения из выдачи
    #[arg(short='E', long)]
    exclude_file: Option<String>,
    /// Входные файлы
    filenames: Vec<String>,
}


fn main() {
    let ru_stemmer = Stemmer::create(Algorithm::Russian);
    let en_stemmer = Stemmer::create(Algorithm::English);
    let args = Cli::parse();

    let mut exclude: Vec<String> = vec![];

    if let Some(filepath) = args.exclude_file {
        let exclude_filepath = Path::new(&filepath);
        match File::open(exclude_filepath) {
            Ok(f) => exclude.append(&mut BufReader::new(f).lines()
                                                  .filter_map(|x| x.ok())
                                                  .collect::<Vec<String>>()),
            Err(e) => eprintln!("--exclude-file: Ошибка при открытии файла {}: {e}", exclude_filepath.display()),
        };
    }

    if let Some(mut exclude_entries) = args.exclude {
        exclude.append(&mut exclude_entries);
    }

    let mut ru_dict = Dict::new();
    let mut en_dict = Dict::new();

    let mut unstemmed: HashMap<String, String> = HashMap::new();
    
    if args.filenames.len() > 0 {
        for f in args.filenames {
            let path = Path::new(&f);
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Ошибка при открытии файла {}: {e}", path.display()); 
                    continue;
                }
            };

            let mut buf = String::new();
            match file.read_to_string(&mut buf) {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Ошибка при чтении файла {}: {e}", path.display());
                    continue;
                }
            };

            let words = buf.to_lowercase()
                           .chars()
                           .filter(|c| 
                                   c.is_alphabetic() || 
                                   c.is_whitespace() && 
                                   c != &'\n')
                           .collect::<String>()
                           .split(' ')
                           .map(|word| word.to_string())
                           .collect::<Vec<String>>();

            for w in &words {
                if !w.is_ascii() {
                    let stem = ru_stemmer.stem(&w).to_string();

                    if stem_and_compare(&ru_stemmer, w, &exclude) {
                        continue;
                    }

                    // Clone galore!
                    // TODO refac
                    ru_dict.add(stem.clone(), f.clone());
                    if !unstemmed.contains_key(&stem) {
                        unstemmed.insert(stem, w.clone());
                    }
                }
            }

            for w in &words {
                if w.is_ascii() {
                    let stem = en_stemmer.stem(&w).to_string();

                    if stem_and_compare(&en_stemmer, w, &exclude) {
                        continue;
                    }

                    en_dict.add(stem.clone(), f.clone());
                    if !unstemmed.contains_key(&stem) {
                        unstemmed.insert(stem, w.clone());
                    }
                }
            }
        }

        for (word, amount, files) in ru_dict.sort(args.words, args.filenum, args.length) {
            println!("{:-<60}", "-".bold());
            println!("Слово \"{}\" или его форма встречается {} в следующих файлах: ",
                     unstemmed[&word].bold().bright_green(),
                     format!("{amount} раз").bold().yellow());
            for file in files {
                println!("{}", file.purple());
            }
        }
    } else {
        println!("Не было передано ни одного файла!");
    }
}


#[cfg(test)]
mod tests {
    use crate::Dict;

// #[test]
//     fn sort_vec() {
//         let mut st = Dict::new();
//         st.add("foo", "a.txt");
//         st.add("bar", "b.txt");
//         st.add("foo", "c.txt");
//
//         assert_eq!(
//             st.sort(), 
//             vec![
//                 (&"foo", 2, &vec!["a.txt".to_string(), "c.txt".to_string()]),
//                 (&"bar", 1, &vec!["b.txt".to_string()]),
//             ])
//     }
}
