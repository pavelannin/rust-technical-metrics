use crate::model::{Sprint, User};

pub struct Report {
    pub sprints: Vec<SprintReport>,
}

pub struct SprintReport {
    pub sprint: Sprint,
    pub users: Vec<UserReport>,
}

pub struct UserReport {
    pub user: User,
    pub commits: CommitReport,
    pub pull_requests: PullRequestReport,
}

pub struct CommitReport {
    pub files_changed: usize,
    pub change_lines: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub commits: usize,
}

pub struct PullRequestReport {
    pub create_pull_requests: usize,
    pub merged_pull_requests: usize,
    pub closed_pull_requests: usize,
    pub got_discussions: usize,
    pub approver_assigned: usize,
    pub approver_conducted: usize,
    pub approver_added_discussions: usize,
}
