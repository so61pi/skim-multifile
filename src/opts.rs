use clap::Parser;
use skim::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliOptions {
    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub color: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long, default_value_t = String::from("10"))]
    pub min_height: String,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub no_height: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long, default_value_t = String::from("100%"))]
    pub height: String,

    #[arg(help_heading = "Skim Options")]
    #[arg(long, default_value_t = String::from("0,0,0,0"))]
    pub margin: String,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub preview: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub cmd: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub query: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub prompt: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub expect: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub multi: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long, default_value_t = String::from("default"))]
    pub layout: String,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub reverse: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub no_hscroll: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub no_mouse: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub no_clear: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub tabstop: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub tac: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub nosort: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub tiebreak: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub inline_info: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub header: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub header_lines: Option<usize>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub case: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub keep_right: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub skip_to_pattern: Option<String>,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub select1: bool,

    #[arg(help_heading = "Skim Options")]
    #[arg(long)]
    pub exit0: bool,

    /// Group score
    #[arg(help_heading = "Custom Options")]
    #[arg(long, default_value_t = 10000, value_name = "SCORE")]
    pub group_score: u16,

    /// Separator between groups
    #[arg(help_heading = "Custom Options")]
    #[arg(long, default_value_t = String::from("---"), value_name = "SEPARATOR")]
    pub group_separator: String,

    #[arg(required = true)]
    pub file: Vec<PathBuf>,
}

impl CliOptions {
    pub fn to_skim_options(&self) -> SkimOptions {
        SkimOptionsBuilder::default()
            .engine_factory(Some(Rc::new(crate::skcustom::CustomEngineFactory::new())))
            .color(self.color.as_deref())
            .min_height(Some(self.min_height.as_str()))
            .no_height(self.no_height)
            .height(Some(self.height.as_str()))
            .margin(Some(self.margin.as_str()))
            .preview(self.preview.as_deref())
            .cmd(self.cmd.as_deref())
            .query(self.query.as_deref())
            .prompt(self.prompt.as_deref())
            .expect(self.expect.clone())
            .multi(self.multi)
            .layout(&self.layout)
            .reverse(self.reverse)
            .no_hscroll(self.no_hscroll)
            .no_mouse(self.no_mouse)
            .no_clear(self.no_clear)
            .tabstop(self.tabstop.as_deref())
            .tac(self.tac)
            .nosort(self.nosort)
            .tiebreak(self.tiebreak.clone())
            .inline_info(self.inline_info)
            .header(self.header.as_deref())
            .header_lines(self.header_lines.unwrap_or_default())
            .case(
                self.case
                    .as_ref()
                    .map(|v| match v.as_str() {
                        "smart" => CaseMatching::Smart,
                        "ignore" => CaseMatching::Ignore,
                        _ => CaseMatching::Respect,
                    })
                    .unwrap_or(CaseMatching::Respect),
            )
            .keep_right(self.keep_right)
            .skip_to_pattern(self.skip_to_pattern.as_deref().unwrap_or_default())
            .select1(self.select1)
            .exit0(self.exit0)
            .build()
            .unwrap()
    }
}
