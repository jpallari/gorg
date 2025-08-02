use anyhow::{Result, bail};

enum UrlScheme {
    Ssh,
    Git,
    Rsync,
    File,
    Http,
    Https,
}

const URL_SCHEME_MAX_LEN: usize = 5 + 3;

impl UrlScheme {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "ssh" => Some(Self::Ssh),
            "git" => Some(Self::Git),
            "rsync" => Some(Self::Rsync),
            "file" => Some(Self::File),
            "http" => Some(Self::Http),
            "https" => Some(Self::Https),
            _ => None,
        }
    }
}

fn join_strs(from: &[String], to: String, separator: char) -> String {
    let mut to = from
        .iter()
        .filter(|part| !part.trim().is_empty())
        .fold(to, |mut a, b| {
            a.push_str(&b);
            a.push(separator);
            a
        });
    _ = to.pop();
    to
}

fn set_git_suffix(target: &mut String) {
    if !target.ends_with(".git") {
        target.push_str(".git")
    }
}

fn total_length(strs: &[String]) -> usize {
    strs.iter().map(|p| p.len()).sum()
}

fn starts_with_scheme(s: &str) -> bool {
    s.split_once(':')
        .and_then(|(maybe_scheme, _)| UrlScheme::from_str(maybe_scheme))
        .is_some()
}

fn buffer_for_remote_parts(remote: &[String]) -> String {
    String::with_capacity(URL_SCHEME_MAX_LEN + total_length(remote) + 4)
}

pub fn from_parts(remote: &[String]) -> Result<String> {
    // parse first part to URL and append other non-empty parts on top with slash separation
    // check if first part is ssh, git, rsync, file, http, or https
    //   if yes, use that as the url scheme and next part as the host
    //   if not, assume https and use the first part as host
    match remote.len() {
        0 => bail!("Not enough parameters to build a remote URL"),
        1 => {
            // Only one part so let's use that as the URL
            return Ok(remote[0].clone());
        }
        _ => {}
    }

    // Is it a file URL?
    if remote[0].starts_with("/") || remote[0].starts_with("~") {
        bail!("File URLs are not supported");
    }

    // Parse scheme
    let Some(scheme) = UrlScheme::from_str(&remote[0]) else {
        let mut url = buffer_for_remote_parts(remote);
        if starts_with_scheme(&remote[0]) {
            // First part includes the scheme
            url.push_str(&remote[0]);
        } else {
            // No scheme set so let's assume HTTPS
            url.push_str("https://");
            url.push_str(&remote[0]);
        }

        url.push('/');
        url = join_strs(&remote[1..], url, '/');
        set_git_suffix(&mut url);
        return Ok(url);
    };

    let mut url = buffer_for_remote_parts(remote);
    url.push_str(&remote[0]);
    url.push_str("://");

    match scheme {
        UrlScheme::Ssh | UrlScheme::Rsync => {
            // For SSH/RSYNC, add default git user when no user is set.
            if remote[1].contains("@") {
                url.push_str(&remote[1]);
            } else {
                url.push_str("git@");
                url.push_str(&remote[1]);
            }
        }
        UrlScheme::File => {
            bail!("File URLs are not supported");
        }
        _ => {
            url.push_str(&remote[1]);
        }
    }
    url.push('/');
    url = join_strs(&remote[2..], url, '/');
    set_git_suffix(&mut url);
    Ok(url)
}

fn left_of(s: &str, c: char) -> &str {
    s.split_once(c).map(|(l, _)| l).unwrap_or(s)
}

fn right_of(s: &str, c: char) -> &str {
    s.split_once(c).map(|(_, r)| r).unwrap_or(s)
}

pub fn to_path(url: &str) -> Result<Vec<&str>> {
    let url = url.trim();
    if url.is_empty() {
        bail!("Empty URL cannot be converted to a path");
    }

    let Some((url_left, url_right)) = url.split_once(':') else {
        bail!("Unsupported URL: {url}");
    };

    let mut path: Vec<&str> = Vec::new();
    let host_part: &str;
    let path_part: &str;

    match UrlScheme::from_str(url_left) {
        Some(UrlScheme::File) => bail!("File URLs are unsupported: {url}"),
        Some(_) => {
            // Starts with a URL scheme
            let Some(url_right) = url_right.strip_prefix("//") else {
                bail!("Invalid URL: {url}");
            };
            let Some((url_left, url_right)) = url_right.split_once('/') else {
                bail!("Invalid URL: {url}");
            };
            host_part = left_of(right_of(url_left, '@'), ':');
            path_part = url_right;
        }
        None => {
            // No URL scheme found => Handle as SSH URL
            host_part = right_of(url_left, '@');
            path_part = url_right;
        }
    }

    // Add the host part
    path.push(host_part);

    // Add the rest of the parts to the list
    let mut parts = path_part.split('/').map(|p| p.trim()).peekable();
    while let Some(part) = parts.next() {
        let mut part = part.strip_prefix('~').unwrap_or(part);
        if parts.peek() == None {
            part = part.strip_suffix(".git").unwrap_or(part);
        }
        if !part.is_empty() {
            path.push(part);
        }
    }

    if path.len() <= 1 {
        bail!("Not enough parts in URL to convert it to a path");
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_parts_single_https() {
        let url = "https://github.com/jpallari/gorg.git";
        let parts = vec![url.to_string()];
        assert_eq!(from_parts(&parts).unwrap(), url);
    }

    #[test]
    fn from_parts_no_scheme() {
        let url = "https://github.com/jpallari/gorg.git";
        let parts = vec![
            "github.com".to_string(),
            "jpallari".to_string(),
            "gorg".to_string(),
        ];
        assert_eq!(from_parts(&parts).unwrap(), url);
    }

    #[test]
    fn from_parts_trailing_git() {
        let url = "https://github.com/jpallari/gorg.git";
        let parts = vec![
            "github.com".to_string(),
            "jpallari".to_string(),
            "gorg.git".to_string(),
        ];
        assert_eq!(from_parts(&parts).unwrap(), url);
    }

    #[test]
    fn from_parts_with_scheme_ssh_default_user() {
        let url = "ssh://git@github.com/jpallari/gorg.git";
        let parts = vec![
            "ssh".to_string(),
            "github.com".to_string(),
            "jpallari".to_string(),
            "gorg".to_string(),
        ];
        assert_eq!(from_parts(&parts).unwrap(), url);
    }

    #[test]
    fn from_parts_with_scheme_ssh_user() {
        let url = "ssh://user@github.com/jpallari/gorg.git";
        let parts = vec![
            "ssh".to_string(),
            "user@github.com".to_string(),
            "jpallari".to_string(),
            "gorg".to_string(),
        ];
        assert_eq!(from_parts(&parts).unwrap(), url);
    }

    #[test]
    fn from_parts_with_scheme_http() {
        let url = "http://github.com/jpallari/gorg.git";
        let parts = vec![
            "http".to_string(),
            "github.com".to_string(),
            "jpallari".to_string(),
            "gorg".to_string(),
        ];
        assert_eq!(from_parts(&parts).unwrap(), url);
    }

    #[test]
    fn from_parts_fail_on_no_parts() {
        assert_eq!(from_parts(&Vec::new()).is_err(), true);
    }

    #[test]
    fn from_parts_invalid() {
        assert_eq!(
            from_parts(&vec!["file".to_string(), "path/to/repo".to_string(),]).is_err(),
            true
        );
        assert_eq!(
            from_parts(&vec!["file".to_string(), "/path/to/repo".to_string(),]).is_err(),
            true
        );
        assert_eq!(
            from_parts(&vec!["file".to_string(), "~/path/to/repo".to_string(),]).is_err(),
            true
        );
        assert_eq!(
            from_parts(&vec!["/".to_string(), "path/to/repo".to_string(),]).is_err(),
            true
        );
        assert_eq!(
            from_parts(&vec!["~".to_string(), "path/to/repo".to_string(),]).is_err(),
            true
        );
    }

    #[test]
    fn to_path_empty() {
        assert_eq!(to_path("").is_err(), true);
    }

    #[test]
    fn to_path_invalid_url() {
        assert_eq!(to_path("https://").is_err(), true);
        assert_eq!(to_path("file:///path/to/repo").is_err(), true);
        assert_eq!(to_path("/path/to/repo").is_err(), true);
        assert_eq!(to_path("~/path/to/repo").is_err(), true);
    }

    #[test]
    fn to_path_https() {
        let url = "https://github.com/jpallari/gorg.git";
        let path = vec!["github.com", "jpallari", "gorg"];
        assert_eq!(to_path(url).unwrap(), path);
    }

    #[test]
    fn to_path_ssh() {
        let url = "ssh://git@github.com/jpallari/gorg.git";
        let path = vec!["github.com", "jpallari", "gorg"];
        assert_eq!(to_path(url).unwrap(), path);
    }

    #[test]
    fn to_path_ssh_with_port() {
        let url = "ssh://git@github.com:2022/jpallari/gorg.git";
        let path = vec!["github.com", "jpallari", "gorg"];
        assert_eq!(to_path(url).unwrap(), path);
    }

    #[test]
    fn to_path_ssh_implied() {
        let url = "git@github.com:jpallari/gorg.git";
        let path = vec!["github.com", "jpallari", "gorg"];
        assert_eq!(to_path(url).unwrap(), path);
    }

    #[test]
    fn to_path_ssh_with_user_home() {
        let url = "git@host.xyz:~user/repo.git";
        let path = vec!["host.xyz", "user", "repo"];
        assert_eq!(to_path(url).unwrap(), path);
    }

    #[test]
    fn to_path_ssh_with_home_in_path() {
        let url = "ssh://git@host.xyz:~/user/repo.git";
        let path = vec!["host.xyz", "user", "repo"];
        assert_eq!(to_path(url).unwrap(), path);
    }
}
