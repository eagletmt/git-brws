extern crate getopts;

use std::io::Write;
use std::path::PathBuf;
use self::getopts::Options;
use command;
use git;

pub enum ParsedArgv {
    Help,
    Version,
    Parsed(command::Options, bool),
}

type ErrorMsg = String;

fn normalize_repo_format(mut s: String, git_dir: &PathBuf) -> Result<String, ErrorMsg> {
    if let Ok(url) = git::new(git_dir)?.remote_url(&s) {
        return Ok(url);
    }

    if !s.ends_with(".git") {
        s.push_str(".git");
    }

    if s.starts_with("git@") || s.starts_with("https://") || s.starts_with("http://") {
        return Ok(s);
    }

    match s.chars().filter(|c| *c == '/').count() {
        1 => Ok(format!("https://github.com/{}", s)),
        2 => Ok(format!("https://{}", s)),
        _ => Err(format!("Error: Invalid repository format '{}'. Format must be one of 'user/repo', 'service/user/repo' or remote name or Git URL.", s)),
    }
}

pub fn parse_options(argv: Vec<String>) -> Result<ParsedArgv, ErrorMsg> {
    let program = argv[0].clone();
    let mut opts = Options::new();

    opts.optopt("r", "repo", "Shorthand format (user/repo, service/user/repo) or remote name (e.g. origin) or Git URL you want to see", "REPO");
    opts.optopt("b", "branch", "Branch name of the repository", "BRANCH");
    opts.optopt("d", "dir", "Directory path to your repository", "PATH");
    opts.optflag("u", "url", "Output URL to STDOUT instead of opening in browser");
    opts.optflag("h", "help", "Print this help");
    opts.optflag("v", "version", "Show version");

    let matches = opts.parse(&argv[1..]).map_err(|f| format!("{}", f))?;

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [Options] {{Args}}", program);
        errorln!("{}", opts.usage(&brief));
        return Ok(ParsedArgv::Help);
    }

    if matches.opt_present("v") {
        println!("{}", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
        return Ok(ParsedArgv::Version);
    }

    let git_dir = git::git_dir(matches.opt_str("d"))?;

    let (repo, branch) = match (matches.opt_str("r"), matches.opt_str("b")) {
        (Some(r), Some(b)) => (normalize_repo_format(r, &git_dir)?, b),
        (Some(r), None) => (normalize_repo_format(r, &git_dir)?, git::new(&git_dir)?.tracking_remote()?.1),
        (None, Some(b)) => (git::new(&git_dir)?.tracking_remote()?.0, b),
        (None, None) => git::new(&git_dir)?.tracking_remote()?,
    };

    let show_url = matches.opt_present("u");

    Ok(ParsedArgv::Parsed(command::Options {
        repo: repo,
        branch: branch,
        git_dir: git_dir,
        args: matches.free,
    }, show_url))
}
