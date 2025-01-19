use crate::model;
use git2::build::CheckoutBuilder;
use git2::{
    build::RepoBuilder, Config, Error, FetchOptions, RemoteCallbacks, Repository, ResetType,
};
use git2_credentials::CredentialHandler;
use model::Repository as Repo;
use std::path::Path;

pub type PercentProgress<'a> = Box<dyn FnMut(usize, usize) + 'a>;

pub trait GitRepository {
    fn repo_exists(&self, dir_path: &str) -> bool;
    fn repo_clone<'a>(&self, dir_path: &str, cb: PercentProgress<'a>) -> Result<Repository, Error>;
    fn repo_pull<'a>(&self, dir_path: &str, cb: PercentProgress<'a>) -> Result<Repository, Error>;
}

impl GitRepository for Repo {
    fn repo_exists(&self, dir_path: &str) -> bool {
        let path = Path::new(dir_path).join(&self.name);
        path.exists()
    }

    fn repo_clone<'a>(
        &self,
        dir_path: &str,
        mut cb: PercentProgress<'a>,
    ) -> Result<Repository, Error> {
        let git_config = Config::open_default()?;
        let mut credential_handler = CredentialHandler::new(git_config);

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |url, username, allowed| {
            credential_handler.try_next_credential(url, username, allowed)
        });
        callbacks.transfer_progress(|stats| {
            let current = 100.0 / stats.total_objects() as f64 * stats.received_objects() as f64;
            cb(current as usize, 100);
            true
        });

        let mut options = FetchOptions::new();
        options.remote_callbacks(callbacks);

        let path = Path::new(dir_path).join(&self.name);
        RepoBuilder::new()
            .fetch_options(options)
            .branch(&self.branch)
            .clone(&self.ssh, &path)
    }

    fn repo_pull<'a>(
        &self,
        dir_path: &str,
        mut cb: PercentProgress<'a>,
    ) -> Result<Repository, Error> {
        let git_config = Config::open_default()?;
        let mut credential_handler = CredentialHandler::new(git_config);

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |url, username, allowed| {
            credential_handler.try_next_credential(url, username, allowed)
        });
        callbacks.transfer_progress(|stats| {
            let current = 100.0 / stats.total_objects() as f64 * stats.received_objects() as f64;
            cb(current as usize, 100);
            true
        });

        let mut options = FetchOptions::new();
        options.remote_callbacks(callbacks);

        let path = Path::new(dir_path).join(&self.name);
        let repo = Repository::open(path)?;
        let mut remote = repo.find_remote("origin")?;
        reset(&repo, &self.branch)?;
        checkout(&repo, &self.branch)?;
        remote.fetch(&[&self.branch], Some(&mut options), None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
        merge(&repo, &self.branch, fetch_commit)?;

        let path = Path::new(dir_path).join(&self.name);
        Repository::open(path)
    }
}

fn reset(repo: &Repository, branch: &str) -> Result<(), Error> {
    let rev = repo.revparse_single(branch)?;
    repo.reset(&rev, ResetType::Hard, None)
}

fn checkout(repo: &Repository, branch: &str) -> Result<(), Error> {
    let (object, _) = repo.revparse_ext(branch)?;
    repo.checkout_tree(&object, None)
}

fn merge<'a>(
    repo: &'a Repository,
    branch: &str,
    fetch_commit: git2::AnnotatedCommit<'a>,
) -> Result<(), Error> {
    let analysis = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.0.is_fast_forward() {
        let refname = format!("refs/heads/{}", branch);
        match repo.find_reference(&refname) {
            Ok(mut r) => {
                fast_forward(repo, &mut r, &fetch_commit)?;
            }
            Err(_) => {
                repo.reference(
                    &refname,
                    fetch_commit.id(),
                    true,
                    &format!("Setting {} to {}", branch, fetch_commit.id()),
                )?;
                repo.set_head(&refname)?;
                repo.checkout_head(Some(
                    CheckoutBuilder::default()
                        .allow_conflicts(true)
                        .conflict_style_merge(true)
                        .force(),
                ))?;
            }
        };
    } else if analysis.0.is_normal() {
        let head_commit = repo.reference_to_annotated_commit(&repo.head()?)?;
        normal_merge(&repo, &head_commit, &fetch_commit)?;
    }
    Ok(())
}

fn fast_forward(
    repo: &Repository,
    lb: &mut git2::Reference,
    rc: &git2::AnnotatedCommit,
) -> Result<(), Error> {
    let name = match lb.name() {
        Some(s) => s.to_string(),
        None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
    };
    let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
    lb.set_target(rc.id(), &msg)?;
    repo.set_head(&name)?;
    repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
    Ok(())
}

fn normal_merge(
    repo: &Repository,
    local: &git2::AnnotatedCommit,
    remote: &git2::AnnotatedCommit,
) -> Result<(), Error> {
    let local_tree = repo.find_commit(local.id())?.tree()?;
    let remote_tree = repo.find_commit(remote.id())?.tree()?;
    let ancestor = repo
        .find_commit(repo.merge_base(local.id(), remote.id())?)?
        .tree()?;
    let mut idx = repo.merge_trees(&ancestor, &local_tree, &remote_tree, None)?;

    if idx.has_conflicts() {
        repo.checkout_index(Some(&mut idx), None)?;
        return Ok(());
    }
    let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
    let msg = format!("Merge: {} into {}", remote.id(), local.id());
    let sig = repo.signature()?;
    let local_commit = repo.find_commit(local.id())?;
    let remote_commit = repo.find_commit(remote.id())?;
    let _merge_commit = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        &msg,
        &result_tree,
        &[&local_commit, &remote_commit],
    )?;
    repo.checkout_head(None)?;
    Ok(())
}
