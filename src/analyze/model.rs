use crate::git::Commit;
use crate::gitea::PullRequest;
use crate::model::{Repository, Sprint, User};
use std::collections::HashMap;

pub type RepoCommits = HashMap<Repository, Vec<Commit>>;
pub type RepoPullRequests = HashMap<Repository, Vec<PullRequest>>;

#[derive(Debug, Clone)]
pub struct DataAnalysis {
    pub users: Vec<User>,
    pub sprints: Vec<Sprint>,
    pub repos: Vec<Repository>,
    pub commits: RepoCommits,
    pub pull_requests: RepoPullRequests,
}

impl DataAnalysis {
    pub fn new(users: Vec<User>, sprints: Vec<Sprint>, repos: Vec<Repository>) -> Self {
        Self {
            users,
            sprints,
            repos,
            commits: HashMap::new(),
            pull_requests: HashMap::new(),
        }
    }

    pub fn insert_commits(&mut self, repository: &Repository, commits: Vec<Commit>) {
        self.commits.insert(repository.clone(), commits);
    }

    pub fn insert_pull_request(
        &mut self,
        repository: &Repository,
        pull_requests: Vec<PullRequest>,
    ) {
        self.pull_requests.insert(repository.clone(), pull_requests);
    }
}

pub type SprintsAnalyzed = Vec<(Sprint, UsersAnalyzed)>;
pub type UsersAnalyzed = Vec<(User, UserDataAnalyzed)>;

#[derive(Debug, Clone)]
pub struct UserDataAnalyzed {
    pub commits: CommitAnalyzed,
    pub pull_requests: PullRequestAnalyzed,
}

impl UserDataAnalyzed {
    pub fn new(commits: CommitAnalyzed, pull_requests: PullRequestAnalyzed) -> Self {
        Self { commits, pull_requests, }
    }
}

#[derive(Debug, Clone)]
pub struct CommitAnalyzed {
    pub files_changed: usize,
    pub change_lines: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub commits: usize,
}

impl CommitAnalyzed {
    pub fn new(
        files_changed: usize,
        change_lines: usize,
        insertions: usize,
        deletions: usize,
        commits: usize,
    ) -> Self {
        Self {
            files_changed,
            change_lines,
            insertions,
            deletions,
            commits,
        }
    }

    pub fn default() -> Self {
        Self::new(0, 0, 0, 0, 0)
    }

    pub fn from_commits(commits: Vec<&Commit>) -> Self {
        commits.iter().fold(Self::default(), |mut acc, c| {
            acc.files_changed += c.files_changed;
            acc.change_lines += c.insertions + c.deletions;
            acc.insertions += c.insertions;
            acc.deletions += c.deletions;
            acc.commits += 1;
            acc
        })
    }
}

#[derive(Debug, Clone)]
pub struct PullRequestAnalyzed {
    pub create_pull_requests: usize,
    pub merged_pull_requests: usize,
    pub closed_pull_requests: usize,
    pub received_discussions: usize,
    pub approver_assigned: usize,
    pub approver_conducted: usize,
    pub approver_added_discussions: usize,
}

impl PullRequestAnalyzed {
    pub fn new(
        create_pull_requests: usize,
        merged_pull_requests: usize,
        closed_pull_requests: usize,
        received_discussions: usize,
        approver_assigned: usize,
        approver_conducted: usize,
        approver_added_discussions: usize,
    ) -> Self {
        Self {
            create_pull_requests,
            merged_pull_requests,
            closed_pull_requests,
            received_discussions,
            approver_assigned,
            approver_conducted,
            approver_added_discussions,
        }
    }

    pub fn default() -> Self {
        Self::new(0, 0, 0, 0, 0, 0, 0)
    }
}
