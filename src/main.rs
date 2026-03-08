use clap::Parser;
use shrinkpath::{shrink, shrink_detailed, PathStyle, ShrinkOptions, Strategy};
use std::io::{self, BufRead, Write};

#[derive(Parser)]
#[command(
    name = "shrinkpath",
    version,
    about = "Smart cross-platform path shortening"
)]
struct Cli {
    /// Paths to shorten (reads stdin if omitted, one path per line)
    paths: Vec<String>,

    /// Maximum length of output
    #[arg(short = 'm', long, default_value = "40")]
    max_len: usize,

    /// Strategy: ellipsis, fish, hybrid
    #[arg(short, long, default_value = "hybrid")]
    strategy: String,

    /// Force path style: unix, windows, auto
    #[arg(long, default_value = "auto")]
    style: String,

    /// Custom ellipsis string
    #[arg(long, default_value = "...")]
    ellipsis: String,

    /// Output JSON with original, shortened, metadata
    #[arg(long)]
    json: bool,
}

fn parse_strategy(s: &str) -> Strategy {
    match s.to_lowercase().as_str() {
        "ellipsis" | "e" => Strategy::Ellipsis,
        "fish" | "f" => Strategy::Fish,
        "hybrid" | "h" => Strategy::Hybrid,
        "unique" | "u" => Strategy::Unique,
        _ => {
            eprintln!("Unknown strategy '{s}', using hybrid");
            Strategy::Hybrid
        }
    }
}

fn parse_style(s: &str) -> Option<PathStyle> {
    match s.to_lowercase().as_str() {
        "unix" | "u" => Some(PathStyle::Unix),
        "windows" | "win" | "w" => Some(PathStyle::Windows),
        _ => None,
    }
}

fn process_path(path: &str, opts: &ShrinkOptions, json: bool) {
    if json {
        let result = shrink_detailed(path, opts);
        let style_str = match result.detected_style {
            PathStyle::Unix => "unix",
            PathStyle::Windows => "windows",
        };
        println!(
            "{{\"original\":{},\"shortened\":{},\"original_len\":{},\"shortened_len\":{},\"truncated\":{},\"style\":\"{}\"}}",
            json_escape(path),
            json_escape(&result.shortened),
            result.original_len,
            result.shortened_len,
            result.was_truncated,
            style_str,
        );
    } else {
        println!("{}", shrink(path, opts));
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn main() {
    let cli = Cli::parse();

    let mut opts = ShrinkOptions::new(cli.max_len)
        .strategy(parse_strategy(&cli.strategy))
        .ellipsis(cli.ellipsis);

    if let Some(style) = parse_style(&cli.style) {
        opts = opts.path_style(style);
    }

    if cli.paths.is_empty() {
        // Read from stdin
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());
        for line in stdin.lock().lines() {
            match line {
                Ok(path) => {
                    let trimmed = path.trim();
                    if !trimmed.is_empty() {
                        if cli.json {
                            let result = shrink_detailed(trimmed, &opts);
                            let style_str = match result.detected_style {
                                PathStyle::Unix => "unix",
                                PathStyle::Windows => "windows",
                            };
                            let _ = writeln!(
                                out,
                                "{{\"original\":{},\"shortened\":{},\"original_len\":{},\"shortened_len\":{},\"truncated\":{},\"style\":\"{}\"}}",
                                json_escape(trimmed),
                                json_escape(&result.shortened),
                                result.original_len,
                                result.shortened_len,
                                result.was_truncated,
                                style_str,
                            );
                        } else {
                            let _ = writeln!(out, "{}", shrink(trimmed, &opts));
                        }
                    }
                }
                Err(_) => break,
            }
        }
    } else {
        for path in &cli.paths {
            process_path(path, &opts, cli.json);
        }
    }
}
