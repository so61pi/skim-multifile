use clap::Parser;
use itertools::Itertools;
use skim::prelude::*;
use std::thread;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

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
                result.rank[0] += item.group;
            }
        }
        result
    }
}

impl std::fmt::Display for CustomEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(CustomEngine: {})", self.inner)
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliOptions {
    #[arg(long)]
    color: Option<String>,

    #[arg(long, default_value_t = String::from("10"))]
    min_height: String,

    #[arg(long)]
    no_height: bool,

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
    multi: bool,

    #[arg(long, default_value_t = String::from("default"))]
    layout: String,

    #[arg(long)]
    reverse: bool,

    #[arg(long)]
    no_hscroll: bool,

    #[arg(long)]
    no_mouse: bool,

    #[arg(long)]
    no_clear: bool,

    #[arg(long)]
    tabstop: Option<String>,

    #[arg(long)]
    tac: bool,

    #[arg(long)]
    nosort: bool,

    #[arg(long)]
    tiebreak: Option<String>,

    #[arg(long)]
    inline_info: bool,

    #[arg(long)]
    header: Option<String>,

    #[arg(long)]
    header_lines: Option<usize>,

    #[arg(long)]
    case: Option<String>,

    #[arg(long)]
    keep_right: bool,

    #[arg(long)]
    skip_to_pattern: Option<String>,

    #[arg(long)]
    select1: bool,

    #[arg(long)]
    exit0: bool,

    #[arg(required = true)]
    file: Vec<PathBuf>,
}

fn to_skim_options(options: &CliOptions) -> SkimOptions {
    SkimOptionsBuilder::default()
        .engine_factory(Some(Rc::new(CustomEngineFactory::new())))
        .color(options.color.as_deref())
        .min_height(Some(options.min_height.as_str()))
        .no_height(options.no_height)
        .height(Some(options.height.as_str()))
        .margin(Some(options.margin.as_str()))
        .preview(options.preview.as_deref())
        .cmd(options.cmd.as_deref())
        .query(options.query.as_deref())
        .prompt(options.prompt.as_deref())
        .expect(options.expect.clone())
        .multi(options.multi)
        .layout(&options.layout)
        .reverse(options.reverse)
        .no_hscroll(options.no_hscroll)
        .no_mouse(options.no_mouse)
        .no_clear(options.no_clear)
        .tabstop(options.tabstop.as_deref())
        .tac(options.tac)
        .nosort(options.nosort)
        .tiebreak(options.tiebreak.clone())
        .inline_info(options.inline_info)
        .header(options.header.as_deref())
        .header_lines(options.header_lines.unwrap_or_default())
        .case(
            options
                .case
                .as_ref()
                .map(|v| match v.as_str() {
                    "smart" => CaseMatching::Smart,
                    "ignore" => CaseMatching::Ignore,
                    _ => CaseMatching::Respect,
                })
                .unwrap_or(CaseMatching::Respect),
        )
        .keep_right(options.keep_right)
        .skip_to_pattern(options.skip_to_pattern.as_deref().unwrap_or_default())
        .select1(options.select1)
        .exit0(options.exit0)
        .build()
        .unwrap()
}

fn read_files_to_skim_items<'a, I>(filepaths: I, tx: SkimItemSender)
where
    I: IntoIterator<Item = &'a PathBuf>,
{
    filepaths
        .into_iter()
        .map(itertools::Either::Left)
        .intersperse(itertools::Either::Right(()))
        .enumerate()
        .map(|(i, v)| (10000 * i as i32, v))
        .for_each(|(group, v)| match v {
            itertools::Either::Left(filepath) => {
                let file = File::open(filepath).unwrap_or_else(|_| {
                    panic!("{} could not be opened", filepath.to_string_lossy())
                });
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    tx.send(Arc::new(CustomItem {
                        group,
                        persist: false,
                        inner: line.unwrap().to_string(),
                    }))
                    .unwrap();
                }
            }
            itertools::Either::Right(_) => {
                tx.send(Arc::new(CustomItem {
                    group,
                    persist: true,
                    inner: String::from("---"),
                }))
                .unwrap();
            }
        });
}

pub fn main() {
    let options = CliOptions::parse();
    let skim_options = to_skim_options(&options);

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = bounded(1024);
    thread::scope(|s| {
        s.spawn(|| {
            read_files_to_skim_items(options.file.iter(), tx);
        });

        let selected_items = Skim::run_with(&skim_options, Some(rx))
            .map(|out| {
                if out.is_abort {
                    Vec::new()
                } else {
                    out.selected_items
                }
            })
            .unwrap_or_else(Vec::new);

        for item in selected_items.iter() {
            println!("{}", item.output());
        }
    });
}
