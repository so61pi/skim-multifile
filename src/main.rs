use clap::Parser;
use itertools::Itertools;
use skim::prelude::*;
use std::thread;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

mod opts;
mod skcustom;

fn read_files_to_skim_items<'a, I>(
    filepaths: I,
    group_score: u16,
    group_separator: &str,
    tx: SkimItemSender,
) where
    I: IntoIterator<Item = &'a PathBuf>,
{
    filepaths
        .into_iter()
        .map(itertools::Either::Left)
        .intersperse(itertools::Either::Right(()))
        .enumerate()
        .map(|(i, v)| (group_score * i as u16, v))
        .for_each(|(group, v)| match v {
            itertools::Either::Left(filepath) => {
                let file = File::open(filepath).unwrap_or_else(|_| {
                    panic!("{} could not be opened", filepath.to_string_lossy())
                });
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    tx.send(Arc::new(skcustom::CustomItem {
                        group,
                        persist: false,
                        inner: line.unwrap().to_string(),
                    }))
                    .unwrap();
                }
            }
            itertools::Either::Right(_) => {
                tx.send(Arc::new(skcustom::CustomItem {
                    group,
                    persist: true,
                    inner: group_separator.to_string(),
                }))
                .unwrap();
            }
        });
}

pub fn main() {
    let options = opts::CliOptions::parse();
    let skim_options = options.to_skim_options();

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = bounded(1024);
    thread::scope(|s| {
        s.spawn(|| {
            read_files_to_skim_items(
                options.file.iter(),
                options.group_score,
                &options.group_separator,
                tx,
            );
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
