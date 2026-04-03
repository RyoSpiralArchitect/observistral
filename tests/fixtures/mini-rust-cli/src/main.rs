use mini_rust_cli::config::{AppConfig, resolve_profile_alias};
use mini_rust_cli::render::{RenderOptions, render_greeting};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Default)]
struct CliArgs {
    profile: Option<String>,
    root: Option<PathBuf>,
    shout: bool,
    slug: bool,
}

fn parse_args() -> CliArgs {
    let mut args = CliArgs::default();
    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--profile" => args.profile = iter.next(),
            "--config-root" => args.root = iter.next().map(PathBuf::from),
            "--shout" => args.shout = true,
            "--slug" => args.slug = true,
            _ => {}
        }
    }
    args
}

fn main() {
    let args = parse_args();
    let root = args
        .root
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let config = AppConfig::default();
    let profile = resolve_profile_alias(&root, args.profile.as_deref(), &config);
    let output = render_greeting(
        &profile,
        RenderOptions {
            shout: args.shout,
            include_slug: args.slug,
        },
    );
    println!("{output}");
}
