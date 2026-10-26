//! The branch's keeper: reading a branch at one tip, the files at that tip,
//! and moving the branch from the tip read to the next (78 to 80; C.3).
//!
//! A branch lies on a remote, reached from a bare cache the caller names,
//! or in a repository with no remote. Reading a remote fetches its tip into
//! the cache's `refs/weir/<branch>` and changes nothing on the branch (I7);
//! a tip once read stays readable whole from the cache however the branch
//! moves after (80).

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::git::Git;

/// Files by path, each with its full text: those at a tip, or those a
/// write adds or replaces.
pub type Files = BTreeMap<String, String>;

/// Where a branch lies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Place {
    /// On a remote at `url`, reached from the bare cache at `cache`, where
    /// commits are made and the lock lies (C.3).
    Remote { cache: PathBuf, url: String },
    /// In the repository whose git directory is `git_dir`, with no remote
    /// (48).
    Local { git_dir: PathBuf },
}

/// A shared branch: its name and where it lies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    name: String,
    place: Place,
}

impl Branch {
    /// The branch `name` on the remote at `url`, through the bare cache at
    /// `cache`, made when it is missing.
    pub fn remote(
        name: impl Into<String>,
        cache: impl Into<PathBuf>,
        url: impl Into<String>,
    ) -> Branch {
        Branch {
            name: name.into(),
            place: Place::Remote {
                cache: cache.into(),
                url: url.into(),
            },
        }
    }

    /// The branch `name` of the repository whose git directory is
    /// `git_dir`, which has no remote.
    pub fn local(name: impl Into<String>, git_dir: impl Into<PathBuf>) -> Branch {
        Branch {
            name: name.into(),
            place: Place::Local {
                git_dir: git_dir.into(),
            },
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn place(&self) -> &Place {
        &self.place
    }

    /// The git directory commits are made in and the lock lies in: the
    /// cache, or the repository's own.
    pub fn git_dir(&self) -> &Path {
        match &self.place {
            Place::Remote { cache, .. } => cache,
            Place::Local { git_dir } => git_dir,
        }
    }

    pub(crate) fn head_ref(&self) -> String {
        format!("refs/heads/{}", self.name)
    }

    /// The cache's record of the remote's tip.
    fn cache_ref(&self) -> String {
        format!("refs/weir/{}", self.name)
    }

    /// Whether the name and the address are ones git can be handed safely:
    /// `Err` names the field and why.
    pub(crate) fn check(&self) -> Result<(), (&'static str, String)> {
        check_name(&self.name).map_err(|reason| ("branch", reason))?;
        if let Place::Remote { url, .. } = &self.place {
            check_url(url).map_err(|reason| ("url", reason))?;
        }
        Ok(())
    }
}

/// A branch's name, as a ref under `refs/heads/` and a refspec's side may
/// hold it.
fn check_name(name: &str) -> Result<(), String> {
    let refused = |why: &str| Err(format!("the branch `{name}` {why}"));
    if name.is_empty() {
        return Err("the branch is named by nothing".to_string());
    }
    if name.starts_with('-') {
        return refused("begins with `-`");
    }
    if name
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || "~^:?*[\\".contains(c))
    {
        return refused("holds a character no branch's name holds");
    }
    if name.contains("..") || name.contains("@{") || name == "@" {
        return refused("holds `..` or `@{`, or is `@`");
    }
    if name
        .split('/')
        .any(|segment| segment.is_empty() || segment.starts_with('.') || segment.ends_with(".lock"))
    {
        return refused("has an empty segment, one beginning with `.` or one ending `.lock`");
    }
    if name.ends_with('.') {
        return refused("ends with `.`");
    }
    Ok(())
}

/// A remote's address: never one holding a password, which would reach a
/// command line (71; core.3). The refusal names the address without it.
pub(crate) fn check_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("the remote's address is empty".to_string());
    }
    if url.starts_with('-') || url.contains(['\n', '\r', '\0']) {
        return Err("the remote's address begins with `-` or holds a line break".to_string());
    }
    if let Some((scheme, rest)) = url.split_once("://") {
        let end = rest.find('/').unwrap_or(rest.len());
        let (authority, path) = rest.split_at(end);
        if let Some((userinfo, host)) = authority.rsplit_once('@') {
            if let Some((user, _)) = userinfo.split_once(':') {
                return Err(format!(
                    "the remote's address `{scheme}://{user}@{host}{path}` holds a password: \
                     a remote is reached through git's credential helpers"
                ));
            }
        }
    }
    Ok(())
}

/// A path a write may add or a reader may ask for: relative, its segments
/// non-empty and none `.`, `..` or `.git`, holding no NUL or line break.
pub(crate) fn check_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("is empty".to_string());
    }
    if path.contains(['\0', '\n', '\r']) {
        return Err("holds a NUL or a line break".to_string());
    }
    if path.starts_with('/') {
        return Err("is not relative".to_string());
    }
    for segment in path.split('/') {
        if segment.is_empty() {
            return Err("has an empty segment".to_string());
        }
        if segment == "." || segment == ".." {
            return Err(format!("has the segment `{segment}`"));
        }
        if segment.eq_ignore_ascii_case(".git") {
            return Err("has the segment `.git`".to_string());
        }
    }
    Ok(())
}

/// A step of the protocol, named when it fails.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Read,
    Lock,
    Check,
    Commit,
    Move,
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Step::Read => "read",
            Step::Lock => "lock",
            Step::Check => "check",
            Step::Commit => "commit",
            Step::Move => "move",
        })
    }
}

/// A step that failed, with git's, `recfix`'s or the lock's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub step: Step,
    pub said: String,
}

impl Failure {
    pub(crate) fn new(step: Step, said: impl Into<String>) -> Failure {
        Failure {
            step,
            said: said.into(),
        }
    }
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the {} failed: {}", self.step, self.said)
    }
}

impl std::error::Error for Failure {}

/// A file that could not be read at a tip, naming its path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TipError {
    pub path: String,
    pub reason: String,
}

impl fmt::Display for TipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` {}", self.path, self.reason)
    }
}

impl std::error::Error for TipError {}

/// The commit a branch was read at, whose files read as they stood there
/// however the branch moves after (78, 80).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tip {
    commit: String,
    git_dir: PathBuf,
}

impl Tip {
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// The text of the file at `path` at this tip, or none when it holds
    /// no file there.
    pub fn file(&self, path: &str) -> Result<Option<String>, TipError> {
        check_path(path).map_err(|reason| TipError {
            path: path.to_string(),
            reason,
        })?;
        let object = format!("{}:{path}", self.commit);
        let mut texts = self.cat(&[(path.to_string(), object)])?;
        Ok(texts.pop().and_then(|(_, text)| text))
    }

    /// Every file at this tip whose path begins with `prefix`, the empty
    /// prefix naming them all, each with its text.
    pub fn files(&self, prefix: &str) -> Result<Files, TipError> {
        let git = Git::new(&self.git_dir);
        let listed = git
            .run(&["ls-tree", "-r", "-z", "--full-tree", &self.commit], None)
            .map_err(|said| self.unread(prefix, said))?;
        if !listed.ok() {
            return Err(self.unread(prefix, listed.said()));
        }
        let mut wanted = Vec::new();
        for entry in listed.out.split(|&byte| byte == 0) {
            if entry.is_empty() {
                continue;
            }
            // `<mode> <type> <object>\t<path>`
            let tab = entry
                .iter()
                .position(|&byte| byte == b'\t')
                .ok_or_else(|| self.unread(prefix, "git listed an entry with no path"))?;
            let (head, path) = (&entry[..tab], &entry[tab + 1..]);
            let path = String::from_utf8(path.to_vec()).map_err(|_| TipError {
                path: String::from_utf8_lossy(path).into_owned(),
                reason: "is a path that is not UTF-8".to_string(),
            })?;
            let head = String::from_utf8_lossy(head);
            let mut fields = head.split(' ');
            let (_, kind, object) = (fields.next(), fields.next(), fields.next());
            if kind == Some("blob") && path.starts_with(prefix) {
                let object = object.unwrap_or_default().to_string();
                wanted.push((path, object));
            }
        }
        let texts = self.cat(&wanted)?;
        Ok(texts
            .into_iter()
            .filter_map(|(path, text)| text.map(|text| (path, text)))
            .collect())
    }

    fn unread(&self, path: &str, reason: impl Into<String>) -> TipError {
        TipError {
            path: path.to_string(),
            reason: format!("could not be read at {}: {}", self.commit, reason.into()),
        }
    }

    /// The text of each object, by the path it was asked for: none for an
    /// object the tip does not hold, through one `cat-file --batch`.
    fn cat(&self, objects: &[(String, String)]) -> Result<Vec<(String, Option<String>)>, TipError> {
        if objects.is_empty() {
            return Ok(Vec::new());
        }
        let mut input = Vec::new();
        for (_, object) in objects {
            input.extend_from_slice(object.as_bytes());
            input.push(b'\n');
        }
        let first = &objects[0].0;
        let ran = Git::new(&self.git_dir)
            .run(&["cat-file", "--batch"], Some(&input))
            .map_err(|said| self.unread(first, said))?;
        if !ran.ok() {
            return Err(self.unread(first, ran.said()));
        }
        let mut out = ran.out.as_slice();
        let mut texts = Vec::new();
        for (path, _) in objects {
            let end = out
                .iter()
                .position(|&byte| byte == b'\n')
                .ok_or_else(|| self.unread(path, "git's answer ended early"))?;
            let header = String::from_utf8_lossy(&out[..end]).into_owned();
            out = &out[end + 1..];
            if header.ends_with(" missing") {
                texts.push((path.clone(), None));
                continue;
            }
            let mut fields = header.split(' ');
            let (_, kind, size) = (fields.next(), fields.next(), fields.next());
            let size: usize = size
                .and_then(|size| size.parse().ok())
                .ok_or_else(|| self.unread(path, format!("git answered `{header}`")))?;
            if out.len() < size + 1 {
                return Err(self.unread(path, "git's answer ended early"));
            }
            let body = &out[..size];
            out = &out[size + 1..];
            if kind != Some("blob") {
                return Err(TipError {
                    path: path.clone(),
                    reason: format!("is a folder, not a file, at {}", self.commit),
                });
            }
            let text = String::from_utf8(body.to_vec()).map_err(|_| TipError {
                path: path.clone(),
                reason: format!("is not UTF-8 at {}", self.commit),
            })?;
            texts.push((path.clone(), Some(text)));
        }
        Ok(texts)
    }
}

/// Reads `branch`: the commit it stands at, or none when it does not exist
/// (78). A remote's tip is fetched into the cache, made with
/// `git init --bare` when it is missing; nothing on the branch changes.
pub fn read(branch: &Branch) -> Result<Option<Tip>, Failure> {
    branch
        .check()
        .map_err(|(_, reason)| Failure::new(Step::Read, reason))?;
    if let Place::Remote { cache, .. } = &branch.place {
        std::fs::create_dir_all(cache).map_err(|error| {
            Failure::new(
                Step::Read,
                format!("the cache {} could not be made: {error}", cache.display()),
            )
        })?;
    }
    read_tip(branch)
}

/// [`read`], the name and address checked and the cache's folder made.
pub(crate) fn read_tip(branch: &Branch) -> Result<Option<Tip>, Failure> {
    let failed = |said: String| Failure::new(Step::Read, said);
    let commit = match &branch.place {
        Place::Local { git_dir } => {
            let ran = Git::new(git_dir)
                .run(
                    &["rev-parse", "--verify", "--quiet", &branch.head_ref()],
                    None,
                )
                .map_err(failed)?;
            match ran.code {
                Some(0) => Some(ran.text()),
                Some(1) if ran.out.is_empty() => None,
                _ => return Err(failed(ran.said())),
            }
        }
        Place::Remote { cache, url } => {
            make_cache(cache)?;
            let git = Git::new(cache);
            let refspec = format!("+{}:{}", branch.head_ref(), branch.cache_ref());
            let fetched = git
                .run(
                    &[
                        "fetch",
                        "--quiet",
                        "--no-tags",
                        "--no-write-fetch-head",
                        "--",
                        url,
                        &refspec,
                    ],
                    None,
                )
                .map_err(failed)?;
            if fetched.ok() {
                Some(
                    git.text(&["rev-parse", "--verify", &branch.cache_ref()], None)
                        .map_err(failed)?,
                )
            } else {
                let listed = git
                    .run(
                        &["ls-remote", "--exit-code", "--", url, &branch.head_ref()],
                        None,
                    )
                    .map_err(failed)?;
                match listed.code {
                    Some(2) => None,
                    Some(0) => return Err(failed(fetched.said())),
                    _ => return Err(failed(listed.said())),
                }
            }
        }
    };
    Ok(commit.map(|commit| Tip {
        commit,
        git_dir: branch.git_dir().to_path_buf(),
    }))
}

/// Makes the bare cache in its folder when it holds no repository yet.
fn make_cache(cache: &Path) -> Result<(), Failure> {
    if cache.join("HEAD").is_file() {
        return Ok(());
    }
    let made = Git::new(cache)
        .run(&["init", "--quiet", "--bare"], None)
        .map_err(|said| Failure::new(Step::Read, said))?;
    // Another reader on this machine may have made it meanwhile.
    if made.ok() || cache.join("HEAD").is_file() {
        Ok(())
    } else {
        Err(Failure::new(
            Step::Read,
            format!(
                "the cache {} could not be made: {}",
                cache.display(),
                made.said()
            ),
        ))
    }
}

/// How a move ended: the branch moved, or refused it with git's words.
pub(crate) enum Moved {
    Moved,
    Refused(String),
}

/// Moves `branch` to `commit` only while it stands at `tip`, none meaning
/// it must not exist: on a remote by a push with the tip as the lease's
/// expected value, never with force; with no remote by `update-ref` from
/// the tip, forty zeros for none (44, 48, 79; C.3 step 4; core.27).
pub(crate) fn move_to(
    branch: &Branch,
    tip: Option<&Tip>,
    commit: &str,
    what: &str,
) -> Result<Moved, Failure> {
    let failed = |said: String| Failure::new(Step::Move, said);
    let head = branch.head_ref();
    match &branch.place {
        Place::Remote { cache, url } => {
            let git = Git::new(cache);
            let lease = format!(
                "--force-with-lease={head}:{}",
                tip.map(Tip::commit).unwrap_or_default()
            );
            let pushed = git
                .run(
                    &[
                        "push",
                        "--porcelain",
                        "--quiet",
                        &lease,
                        "--",
                        url,
                        &format!("{commit}:{head}"),
                    ],
                    None,
                )
                .map_err(failed)?;
            if !pushed.ok() {
                return Ok(Moved::Refused(pushed.said()));
            }
            // The cache's own record of the remote's tip, not the branch:
            // it follows the push, as a fetch would set it.
            git.text(&["update-ref", &branch.cache_ref(), commit], None)
                .map_err(failed)?;
            Ok(Moved::Moved)
        }
        Place::Local { git_dir } => {
            let zeros = "0".repeat(40);
            let old = tip.map(Tip::commit).unwrap_or(&zeros);
            let moved = Git::new(git_dir)
                .run(&["update-ref", "-m", what, &head, commit, old], None)
                .map_err(failed)?;
            if moved.ok() {
                Ok(Moved::Moved)
            } else {
                Ok(Moved::Refused(moved.said()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_password_in_the_address_is_refused_without_it() {
        let refused = check_url("https://bot:s3cret@example.org/inbox.git").unwrap_err();
        assert!(
            refused.contains("https://bot@example.org/inbox.git"),
            "{refused}"
        );
        assert!(!refused.contains("s3cret"), "{refused}");
        assert!(check_url("https://bot@example.org/inbox.git").is_ok());
        assert!(check_url("git@example.org:inbox.git").is_ok());
        assert!(check_url("/srv/inbox.git").is_ok());
        assert!(check_url("--upload-pack=x").is_err());
    }

    #[test]
    fn a_path_a_write_may_add() {
        assert!(check_path("inbox/moves/moves.rec").is_ok());
        for path in [
            "",
            "/inbox/a",
            "inbox//a",
            "inbox/",
            "./a",
            "a/../b",
            ".git/config",
            "a/.GIT/x",
            "a\nb",
            "a\0b",
        ] {
            assert!(check_path(path).is_err(), "{path:?} taken");
        }
    }

    #[test]
    fn a_branch_name_git_can_be_handed() {
        assert!(check_name("inbox").is_ok());
        assert!(check_name("weir/inbox").is_ok());
        for name in [
            "", "-x", "a:b", "a b", "a..b", "a/", "/a", ".a", "a.lock", "a@{1", "@", "a.",
        ] {
            assert!(check_name(name).is_err(), "{name:?} taken");
        }
    }
}
