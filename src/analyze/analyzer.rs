use crate::analyze::{
    CommitAnalyzed, DataAnalysis, PullRequestAnalyzed, SprintsAnalyzed, UserDataAnalyzed,
    UsersAnalyzed,
};
use crate::git::Commit;
use crate::gitea::{GiteaPullRequest, PullRequest};
use crate::model::{Sprint, User};
use chrono::DateTime;
use gitea_sdk::model::reviews::ReviewStateType;

pub trait Analyzer {
    fn analyze_sprints(&self) -> SprintsAnalyzed;
}

impl Analyzer for DataAnalysis {
    fn analyze_sprints(&self) -> SprintsAnalyzed {
        let mut sprints_analyzed: SprintsAnalyzed = vec![];
        for sprint in &self.sprints {
            let mut users_analyzed: UsersAnalyzed = vec![];
            for user in &self.users {
                let commits = self.commits_from_user_in_sprint(&user, &sprint);
                let commits_analysis = CommitAnalyzed::from_commits(commits);

                let mut pull_request_analysis = PullRequestAnalyzed::default();
                let pull_requests = self.pull_requests_from_user_in_sprint(&user, &sprint);
                for pull_request in &pull_requests {
                    pull_request_analysis.analyze_request(&pull_request.request, &sprint);
                    pull_request_analysis.analyze_received_discussion(&pull_request, &sprint);
                }

                let pull_requests = self.pull_requests_closed_in_sprint(&sprint);
                for pull_request in &pull_requests {
                    pull_request_analysis.analyze_reviews(&pull_request, &sprint, &user);
                }

                users_analyzed.push((
                    user.clone(),
                    UserDataAnalyzed::new(commits_analysis, pull_request_analysis),
                ));
            }
            sprints_analyzed.push((sprint.clone(), users_analyzed));
        }
        sprints_analyzed
    }
}

trait DataAnalysisExtension {
    fn commits_from_user_in_sprint<'a>(&'a self, user: &User, sprint: &Sprint) -> Vec<&'a Commit>;

    fn pull_requests_from_user_in_sprint<'a>(
        &'a self,
        user: &User,
        sprint: &Sprint,
    ) -> Vec<&'a PullRequest>;

    fn pull_requests_closed_in_sprint<'a>(&'a self, sprint: &Sprint) -> Vec<&'a PullRequest>;
}

impl DataAnalysisExtension for DataAnalysis {
    fn commits_from_user_in_sprint<'a>(&'a self, user: &User, sprint: &Sprint) -> Vec<&'a Commit> {
        self.commits
            .iter()
            .flat_map(|(_, commits)| commits)
            .filter(|commit| user.emails.contains(&commit.email))
            .filter(|commit| commit.datetime >= sprint.since && commit.datetime <= sprint.until)
            .collect::<Vec<_>>()
    }

    fn pull_requests_from_user_in_sprint<'a>(
        &'a self,
        user: &User,
        sprint: &Sprint,
    ) -> Vec<&'a PullRequest> {
        self.pull_requests
            .iter()
            .flat_map(|(_, pull_requests)| pull_requests)
            .filter(|pull_request| user.emails.contains(&pull_request.request.user.email))
            .filter(|pull_request| {
                let pull_request = pull_request.request.clone();
                datetime_include_sprint(&Some(pull_request.created_at.to_string()), sprint)
                    || datetime_include_sprint(&pull_request.merged_at, sprint)
                    || datetime_include_sprint(&pull_request.closed_at, sprint)
            })
            .collect::<Vec<_>>()
    }

    fn pull_requests_closed_in_sprint<'a>(&'a self, sprint: &Sprint) -> Vec<&'a PullRequest> {
        self.pull_requests
            .iter()
            .flat_map(|(_, pull_requests)| pull_requests)
            .filter(|pull_request| {
                let pull_request = pull_request.request.clone();
                datetime_include_sprint(&pull_request.closed_at, sprint)
            })
            .collect::<Vec<_>>()
    }
}

trait PullRequestAnalyzer {
    fn analyze_request(&mut self, pull_request: &GiteaPullRequest, sprint: &Sprint);
    fn analyze_received_discussion(&mut self, pull_request: &PullRequest, sprint: &Sprint);
    fn analyze_reviews(&mut self, pull_request: &PullRequest, sprint: &Sprint, user: &User);
}

impl PullRequestAnalyzer for PullRequestAnalyzed {
    fn analyze_request(&mut self, pull_request: &GiteaPullRequest, sprint: &Sprint) {
        if datetime_include_sprint(&Some(pull_request.created_at.to_string()), sprint) {
            self.create_pull_requests += 1;
        }
        if datetime_include_sprint(&pull_request.merged_at, sprint)
            && datetime_include_sprint(&pull_request.closed_at, sprint)
        {
            self.merged_pull_requests += 1;
        }
        if !datetime_include_sprint(&pull_request.merged_at, sprint)
            && datetime_include_sprint(&pull_request.closed_at, sprint)
        {
            self.closed_pull_requests += 1;
        }
    }

    fn analyze_received_discussion(&mut self, pull_request: &PullRequest, sprint: &Sprint) {
        if datetime_include_sprint(&pull_request.request.closed_at, sprint) {
            for review in &pull_request.reviews {
                self.received_discussions += review.comments_count as usize
            }
        }
    }

    fn analyze_reviews(&mut self, pull_request: &PullRequest, sprint: &Sprint, user: &User) {
        let mut assigned = false;
        let mut conducted = false;
        let mut added_discussions = 0;
        if datetime_include_sprint(&pull_request.request.closed_at, sprint) {
            for review in &pull_request.reviews {
                if let Some(review_user) = &review.user {
                    if user.emails.contains(&review_user.email) {
                        assigned = true;
                        match review.state {
                            ReviewStateType::Approved => {
                                conducted = true;
                            }
                            ReviewStateType::Pending => {}
                            ReviewStateType::Comment => {}
                            ReviewStateType::RequestChanges => {
                                conducted = true;
                            }
                            ReviewStateType::RequestReview => {}
                            ReviewStateType::Unknown => {}
                        }
                        added_discussions += review.comments_count as usize
                    }
                }
            }
        }
        self.approver_assigned += if assigned { 1 } else { 0 };
        self.approver_conducted += if conducted { 1 } else { 0 };
        self.approver_added_discussions += added_discussions
    }
}

fn datetime_include_sprint(datetime: &Option<String>, sprint: &Sprint) -> bool {
    let Some(datetime) = datetime else {
        return false;
    };
    let datetime = DateTime::parse_from_rfc3339(&datetime).unwrap();
    datetime >= sprint.since && datetime <= sprint.until
}
