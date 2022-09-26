use std::{path::PathBuf, fs::File, io::{BufReader, BufRead}};

use skim::prelude::*;

struct CustomItem {
    /// Group of item. The smaller the better score.
    group: i32,

    /// Set to true to always show this entry in selector.
    persist: bool,

    inner: String,
}

impl SkimItem for CustomItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.inner)
    }
}

struct CustomEngine {
    inner: Box<dyn MatchEngine>,
}

impl MatchEngine for CustomEngine {
    fn match_item(&self, item: Arc<dyn SkimItem>) -> Option<MatchResult> {
        if let Some(item) = (*item).as_any().downcast_ref::<CustomItem>() {
            if item.persist {
                return Some(MatchResult {
                    rank: [item.group, 0, 0, 0],
                    matched_range: MatchRange::Chars(Vec::new()),
                });
            }
        }

        let mut result = self.inner.match_item(item.clone());
        if let Some(item) = (*item).as_any().downcast_ref::<CustomItem>() {
            if let Some(result) = &mut result {
                result.rank[0] = result.rank[0] + item.group;
            }
        }
        return result;
    }
}

impl std::fmt::Display for CustomEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(CustomEngine: {})", self.inner.to_string())
    }
}

struct CustomEngineFactory {
    inner: Box<dyn MatchEngineFactory>,
}

impl CustomEngineFactory {
    fn new() -> Self {
        let fuzzy_engine_factory = ExactOrFuzzyEngineFactory::builder().build();
        Self {
            inner: Box::new(AndOrEngineFactory::new(fuzzy_engine_factory)),
        }
    }
}

impl MatchEngineFactory for CustomEngineFactory {
    fn create_engine_with_case(&self, query: &str, case: CaseMatching) -> Box<dyn MatchEngine> {
        let engine = self.inner.create_engine_with_case(query, case);
        Box::new(CustomEngine { inner: engine })
    }
}

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliOptions {
    #[arg(long)]
    color: Option<String>,

    #[arg(long, default_value_t = String::from("10"))]
    min_height: String,

    #[arg(long)]
    no_height: Option<String>,

    #[arg(long, default_value_t = String::from("100%"))]
    height: String,

    #[arg(long, default_value_t = String::from("0,0,0,0"))]
    margin: String,

    #[arg(long)]
    preview: Option<String>,

    #[arg(long)]
    cmd: Option<String>,

    #[arg(long)]
    query: Option<String>,

    #[arg(long)]
    prompt: Option<String>,

    #[arg(long)]
    expect: Option<String>,

    #[arg(long)]
    multi: Option<String>,

    #[arg(long, default_value_t = String::from("default"))]
    layout: String,

    #[arg(long)]
    reverse: Option<String>,

    #[arg(long)]
    no_hscroll: Option<String>,

    #[arg(long)]
    no_mouse: Option<String>,

    #[arg(long)]
    no_clear: Option<String>,

    #[arg(long)]
    tabstop: Option<String>,

    #[arg(long)]
    tac: Option<String>,

    #[arg(long)]
    nosort: Option<String>,

    #[arg(long)]
    inline_info: Option<String>,

    #[arg(long)]
    header: Option<String>,

    #[arg(long)]
    header_lines: Option<usize>,

    #[arg(long)]
    case: Option<String>,

    #[arg(long)]
    keep_right: Option<String>,

    #[arg(long)]
    skip_to_pattern: Option<String>,

    #[arg(long)]
    select1: Option<String>,

    #[arg(long)]
    exit0: Option<String>,

    file: Vec<PathBuf>,
}

fn parse_options(options: &CliOptions) -> SkimOptions {
    SkimOptionsBuilder::default()
        .engine_factory(Some(Rc::new(CustomEngineFactory::new())))
        .color(options.color.as_ref().map(|v| v.as_str()))
        .min_height(Some(options.min_height.as_str()))
        .no_height(options.no_height.is_some())
        .height(Some(options.height.as_str()))
        .margin(Some(options.margin.as_str()))
        .preview(options.preview.as_ref().map(|v| v.as_str()))
        .cmd(options.cmd.as_ref().map(|v| v.as_str()))
        .query(options.query.as_ref().map(|v| v.as_str()))
        .prompt(options.prompt.as_ref().map(|v| v.as_str()))
        .expect(options.expect.clone())
        .multi(options.multi.is_some())
        .layout(&options.layout)
        .reverse(options.reverse.is_some())
        .no_hscroll(options.no_hscroll.is_some())
        .no_mouse(options.no_mouse.is_some())
        .no_clear(options.no_clear.is_some())
        .tabstop(options.tabstop.as_ref().map(|v| v.as_str()))
        .tac(options.tac.is_some())
        .nosort(options.nosort.is_some())
        .inline_info(options.inline_info.is_some())
        .header(options.header.as_ref().map(|v| v.as_str()))
        .header_lines(options.header_lines.unwrap_or_default())
        .case(options.case.as_ref().map(|v| match v.as_str() {
            "smart" => CaseMatching::Smart,
            "ignore" => CaseMatching::Ignore,
            _ => CaseMatching::Respect,
        }).unwrap_or(CaseMatching::Respect))
        .keep_right(options.keep_right.is_some())
        .skip_to_pattern(options.skip_to_pattern.as_ref().map(|v| v.as_str()).unwrap_or_default())
        .select1(options.select1.is_some())
        .exit0(options.exit0.is_some())
        .build()
        .unwrap()
}

use std::thread;

fn reader(filepaths: Vec<PathBuf>) -> SkimItemReceiver {
    let (tx_item, rx_item): (SkimItemSender, SkimItemReceiver) = bounded(1024);

    thread::spawn(move || {
        for (i2, filepath) in filepaths.iter().enumerate().map(|(i, v)| (2*i, v)) {
            let file = File::open(filepath).expect(&format!("{} could not be opened", filepath.to_string_lossy()));
            let reader = BufReader::new(file);

            for line in reader.lines() {
                tx_item.send(Arc::new(CustomItem {
                    group: i2 as i32 * 10000,
                    persist: false,
                    inner: line.unwrap().to_string(),
                })).unwrap();
            }

            tx_item.send(Arc::new(CustomItem {
                group: (i2+1) as i32 * 10000,
                persist: true,
                inner: String::from("---"),
            })).unwrap();
        }
    });

    return rx_item;
}


pub fn main() {
    let cli = CliOptions::parse();
    let options = parse_options(&cli);
    let rx_item = reader(cli.file.clone());

    let selected_items = Skim::run_with(&options, Some(rx_item))
        .map(|out| out.selected_items)
        .unwrap_or_else(|| Vec::new());

    for item in selected_items.iter() {
        print!("{}{}", item.output(), "\n");
    }
}
