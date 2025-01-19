use chrono::{DateTime, FixedOffset, Utc};
use futures::pending;
use git2::{DiffFindOptions, DiffFormat, DiffOptions, DiffStats, Error, Repository};
use std::env::home_dir;

#[derive(Debug, Clone)]
pub struct Commit {
    pub email: String,
    pub message: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub datetime: DateTime<Utc>,
}

impl Commit {
    fn new(
        email: impl ToString,
        message: impl ToString,
        files_changed: usize,
        insertions: usize,
        deletions: usize,
        datetime: DateTime<Utc>,
    ) -> Self {
        Self {
            email: email.to_string(),
            message: message.to_string(),
            files_changed,
            insertions,
            deletions,
            datetime,
        }
    }
}

pub trait GitCommitRepository {
    fn get_commits(&self, since: &DateTime<FixedOffset>) -> Result<Vec<Commit>, Error>;
}

impl GitCommitRepository for Repository {
    fn get_commits(&self, since: &DateTime<FixedOffset>) -> Result<Vec<Commit>, Error> {
        let git_commits = get_commits(self, since)?;
        let commits = git_commits
            .iter()
            .map(|git_commit| {
                let git_diff = get_commit_stats_for_commit(self, &git_commit).unwrap();
                git_commit_to_commit(&git_commit, &git_diff).unwrap()
            })
            .collect::<Vec<_>>();
        Ok(commits)
    }
}

fn get_commits<'a>(
    repo: &'a Repository,
    since: &'a DateTime<FixedOffset>,
) -> Result<Vec<git2::Commit<'a>>, Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    let commits = revwalk
        .filter_map(|id| repo.find_commit(id.ok()?).ok())
        .filter(|commit| {
            let datetime = DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp_opt(commit.time().seconds(), 0).unwrap(),
                Utc,
            );
            datetime >= *since
        })
        .collect();
    Ok(commits)
}

fn get_commit_stats_for_commit<'a>(
    repo: &'a Repository,
    commit: &git2::Commit<'a>,
) -> Result<DiffStats, Error> {
    let mut diff_options = DiffOptions::new();
    diff_options.patience(true);
    diff_options.include_untracked(true);
    diff_options.include_typechange(true);
    diff_options.include_ignored(true);
    let mut diff_find_options = DiffFindOptions::new();
    diff_find_options.renames(true);
    let old_tree = if commit.parent_count() > 0 {
        let parent = commit.parent(0)?;
        Some(parent.tree()?)
    } else {
        None
    };
    let mut diff = repo.diff_tree_to_tree(
        old_tree.as_ref(),
        Some(&commit.tree()?),
        Some(&mut diff_options),
    )?;
    diff.find_similar(Some(&mut diff_find_options))?;
    diff.stats().map_err(Into::into)
}

fn git_commit_to_commit(
    git_commit: &git2::Commit<'_>,
    git_diff: &DiffStats,
) -> Result<Commit, Error> {
    let author = git_commit.author();
    let message = git_commit.message();
    let datetime = DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp_opt(git_commit.time().seconds(), 0).unwrap(),
        Utc,
    );
    Ok(Commit::new(
        author.email().unwrap_or("").to_string(),
        message.unwrap_or("").to_string(),
        git_diff.files_changed(),
        git_diff.insertions(),
        git_diff.deletions(),
        datetime,
    ))
}
