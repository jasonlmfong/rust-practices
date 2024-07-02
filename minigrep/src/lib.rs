use std::env;
use std::error::Error;
use std::fs;

pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();
        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{line}");
    }

    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut res = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            res.push(line);
        }
    }
    res
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase();

    let mut res = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            res.push(line);
        }
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_case_sensitive_returns_one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn search_case_sensitive_returns_no_results_not_found() {
        let query = "monomorphization";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        let expected_res: Vec<&str> = vec![];
        assert_eq!(expected_res, search(query, contents));
    }

    #[test]
    fn search_case_sensitive_returns_no_results_uppercase() {
        let query = "RUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        let expected_res: Vec<&str> = vec![];
        assert_eq!(expected_res, search(query, contents));
    }

    #[test]
    fn search_case_insensitive_returns_two_result() {
        let query = "RUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}
