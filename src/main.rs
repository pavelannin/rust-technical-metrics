mod analyze;
mod git;
mod gitea;
mod model;
mod report;
mod utils;

use crate::analyze::analyzer::Analyzer;
use crate::analyze::DataAnalysis;
use crate::git::{Commit, GitCommitRepository, GitRepository};
use crate::gitea::pull_request::GiteaPullRequester;
use crate::gitea::PullRequest;
use crate::report::markdown::MarkdownReport;
use crate::utils::{MultiProgressNew, ProgressStyleTemplate};
use chrono::{DateTime, FixedOffset};
use clap::Parser;
use futures::{future, FutureExt};
use gitea_sdk::{Auth, Client};
use indicatif::{MultiProgress, ProgressBar, TermLike};
use itertools::Itertools;
use model::{Repository, Result, Sprint, User};
use std::error::Error;
use std::future::Future;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser, Debug, Clone)]
struct Args {
    #[arg(long = "repos", default_value = "repositories.json")]
    repos_path: String,
    #[arg(long = "sprints", default_value = "sprints.json")]
    sprints_path: String,
    #[arg(long = "users", default_value = "users.json")]
    users_path: String,
    #[arg(long = "cache_path", default_value = "repos")]
    repos_cache_path: String,
    #[arg(long = "gitea_url")]
    gitea_url: String,
    #[arg(long = "gitea_token")]
    gitea_token: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    run(&args).await.unwrap()
}

async fn run(args: &Args) -> Result<()> {
    let (users, sprints, repos) = tokio::spawn(parse_configs(args.clone())).await?;

    let min_since = calc_min_since(&sprints);
    let data_analysis = {
        let analyzer = DataAnalysis::new(users.clone(), sprints.clone(), repos.clone());
        Arc::new(Mutex::new(analyzer))
    };

    for repo in &repos {
        let (commits, pull_requests) = repo_fetch(repo, args, &min_since).await;
        data_analysis
            .lock()
            .await
            .insert_commits(repo, commits.clone());
        data_analysis
            .lock()
            .await
            .insert_pull_request(repo, pull_requests.clone());
    }

    let analyze = data_analysis.lock().await.analyze_sprints();
    for team in find_teams(&users) {
        analyze.report_create(&team);
    }

    Ok(())
}

async fn parse_configs(args: Args) -> (Vec<User>, Vec<Sprint>, Vec<Repository>) {
    async fn parse_config<T, F>(path: &str, pb: &ProgressBar, parser: F) -> Vec<T>
    where
        F: FnOnce(&str) -> Result<Vec<T>>,
    {
        pb.set_message(format!("Read file `{}` ...", path));
        let vec = parser(path).unwrap();
        pb.finish_with_message(format!(
            "✅ Completed parsing file `{}` (find {} elements)",
            path,
            vec.len()
        ));
        vec
    }

    let multi_progress = MultiProgress::default();
    let users_pb = multi_progress.add_with_style(
        ProgressBar::no_length(),
        ProgressStyleTemplate::only_message(),
    );
    let sprints_pb = multi_progress.add_with_style(
        ProgressBar::no_length(),
        ProgressStyleTemplate::only_message(),
    );
    let repos_pb = multi_progress.add_with_style(
        ProgressBar::no_length(),
        ProgressStyleTemplate::only_message(),
    );

    futures::join!(
        parse_config(&args.users_path, &users_pb, User::from_config),
        parse_config(&args.sprints_path, &sprints_pb, Sprint::from_config),
        parse_config(&args.repos_path, &repos_pb, Repository::from_config),
    )
}

async fn repo_fetch(
    repo: &Repository,
    args: &Args,
    min_since: &DateTime<FixedOffset>,
) -> (Vec<Commit>, Vec<PullRequest>) {
    let mut multi_progress = MultiProgress::default();
    multi_progress.println(&format!("# {}", repo.name)).unwrap();

    let repo_pb = multi_progress.add_with_style(
        ProgressBar::no_length(),
        ProgressStyleTemplate::percent_bar(),
    );
    repo_pb.set_message("Fetching...");
    let commit_pb = multi_progress.add_with_style(
        ProgressBar::new_spinner(),
        ProgressStyleTemplate::only_message(),
    );
    commit_pb.set_message("Waiting clone");
    let pull_request_pb = multi_progress.add_with_style(
        ProgressBar::new_spinner(),
        ProgressStyleTemplate::only_message(),
    );
    pull_request_pb.set_message("Waiting Gitea");

    let gitea_client = Client::new(&args.gitea_url, Auth::Token(&args.gitea_token));
    let repos_cache_path = args.repos_cache_path.to_string();
    let (commits, pull_requests) = future::join(
        tokio::spawn(git_fetch(
            repo.clone(),
            repos_cache_path,
            min_since.clone(),
            repo_pb,
            commit_pb,
        )),
        tokio::spawn(gitea_fetch(
            gitea_client,
            repo.clone(),
            min_since.clone(),
            pull_request_pb,
        )),
    )
    .await;
    (commits.unwrap(), pull_requests.unwrap())
}

async fn git_fetch(
    repo: Repository,
    repo_dir_path: String,
    min_since: DateTime<FixedOffset>,
    fetch_pb: ProgressBar,
    commits_pb: ProgressBar,
) -> Vec<Commit> {
    let mut one_call_progress = false;
    let progress_bg = fetch_pb.clone();
    let progress = |is_clone: bool| {
        move |current, total| {
            one_call_progress = true;
            progress_bg.set_message(if is_clone { "Cloning" } else { "Pulling" });
            progress_bg.set_position(current as u64);
            progress_bg.set_length(total as u64);
        }
    };

    let is_clone = !repo.repo_exists(&repo_dir_path);
    fetch_pb.set_message(if is_clone {
        "Cloning ..."
    } else {
        "Pulling ..."
    });
    let git_repo = if is_clone {
        repo.repo_clone(&repo_dir_path, Box::new(progress(is_clone)))
            .unwrap()
    } else {
        repo.repo_pull(&repo_dir_path, Box::new(progress(is_clone)))
            .unwrap()
    };

    fetch_pb.reset();
    fetch_pb.set_style(ProgressStyleTemplate::only_message());
    if one_call_progress {
        fetch_pb.finish_with_message(if is_clone {
            "✅ Cloned"
        } else {
            "✅ Fetched"
        });
    } else {
        fetch_pb.finish_with_message(if is_clone {
            "✅ Cloned"
        } else {
            "Already up to date"
        });
    }

    commits_pb.set_message("Read git history ...");
    let commits = git_repo.get_commits(&min_since).unwrap();
    commits_pb.finish_with_message(format!(
        "✅ Completed read git history (find {} commits)",
        commits.len()
    ));
    commits
}

async fn gitea_fetch(
    client: Client,
    repo: Repository,
    min_since: DateTime<FixedOffset>,
    pb: ProgressBar,
) -> Vec<PullRequest> {
    let progress_pb = pb.clone();
    let progress = move |page: i64| {
        progress_pb.set_message(format!("Fetch pull requests (#{} page) ...", page + 1));
    };
    let pull_requests = repo
        .fetch_pull_request(&client, &min_since, Box::new(progress))
        .await
        .unwrap();
    pb.finish_with_message(format!(
        "✅ Completed fetch pull requests (find {} pull requests)",
        pull_requests.len()
    ));
    pull_requests
}

fn calc_min_since(sprints: &Vec<Sprint>) -> DateTime<FixedOffset> {
    sprints
        .iter()
        .min_by(|s1, s2| s1.since.cmp(&s2.since))
        .unwrap()
        .since
}

fn find_teams<'a>(users: &'a Vec<User>) -> Vec<String> {
    users
        .iter()
        .flat_map(|u| u.teams.clone())
        .unique()
        .collect::<Vec<String>>()
}
