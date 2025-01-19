use crate::model::{Repository, Sprint};
use chrono::{DateTime, FixedOffset};
use gitea_sdk::error::Result;
use gitea_sdk::model::issues::State;
use gitea_sdk::Client;

pub type GiteaPullRequest = gitea_sdk::model::pulls::PullRequest;
pub type GiteaPullReview = gitea_sdk::model::reviews::PullReview;

#[derive(Debug, Clone)]
pub struct PullRequest {
    pub request: GiteaPullRequest,
    pub reviews: Vec<GiteaPullReview>,
}

impl PullRequest {
    fn new(request: GiteaPullRequest, reviews: Vec<GiteaPullReview>) -> Self {
        Self { request, reviews }
    }
}

pub type PercentProgress<'a> = Box<dyn FnMut(i64) + Send +'a>;

pub trait GiteaPullRequester {
    async fn fetch_pull_request<'a>(
        &self,
        client: &Client,
        since: &DateTime<FixedOffset>,
        cb: PercentProgress<'a>,
    ) -> Result<Vec<PullRequest>>;
}

impl GiteaPullRequester for Repository {
    async fn fetch_pull_request<'a>(
        &self,
        client: &Client,
        since: &DateTime<FixedOffset>,
        mut cb: PercentProgress<'a>,
    ) -> Result<Vec<PullRequest>> {
        fn datetime_more_since(datetime: &Option<String>, since: &DateTime<FixedOffset>) -> bool {
            let Some(datetime) = datetime else {
                return false;
            };
            let datetime = DateTime::parse_from_rfc3339(&datetime).unwrap();
            datetime >= *since
        }

        let mut page = 1;
        let mut pull_requests: Vec<PullRequest> = vec![];
        let pulls = client.pulls(&self.owner, &self.name);

        loop {
            cb(page);
            let gitea_pull_requests = pulls
                .list()
                .limit(20)
                .page(page)
                .state(State::All)
                .send(&client)
                .await?;
            let gitea_pull_requests = gitea_pull_requests
                .iter()
                .filter(|pull_request| {
                    datetime_more_since(&Some(pull_request.created_at.to_string()), since)
                        || datetime_more_since(&pull_request.merged_at, since)
                        || datetime_more_since(&pull_request.closed_at, since)
                })
                .collect::<Vec<_>>();
            if gitea_pull_requests.is_empty() {
                break;
            }

            let gitea_reviews = futures::future::join_all(
                gitea_pull_requests
                    .iter()
                    .map(|pr| async { pulls.reviews().get(pr.number).send(&client).await.unwrap() })
                    .collect::<Vec<_>>(),
            )
            .await;

            for index in 0..gitea_pull_requests.len() {
                pull_requests.push(PullRequest::new(
                    gitea_pull_requests[index].clone(),
                    gitea_reviews[index].clone(),
                ))
            }
            page += 1;
        }
        Ok(pull_requests)
    }
}
