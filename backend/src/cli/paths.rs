use std::env;
use std::path::PathBuf;

pub fn home_dir() -> Option<PathBuf> {
    resolve_home_dir_from_values(
        env::var_os("HOME").map(PathBuf::from),
        env::var_os("USERPROFILE").map(PathBuf::from),
        env::var("HOMEDRIVE").ok(),
        env::var("HOMEPATH").ok(),
    )
}

fn resolve_home_dir_from_values(
    home: Option<PathBuf>,
    userprofile: Option<PathBuf>,
    homedrive: Option<String>,
    homepath: Option<String>,
) -> Option<PathBuf> {
    home.filter(|path| !path.as_os_str().is_empty())
        .or_else(|| userprofile.filter(|path| !path.as_os_str().is_empty()))
        .or_else(|| match (homedrive, homepath) {
            (Some(drive), Some(path)) if !drive.is_empty() && !path.is_empty() => {
                Some(PathBuf::from(format!("{drive}{path}")))
            }
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_home_dir_prefers_home_then_userprofile_then_drive_pair() {
        assert_eq!(
            resolve_home_dir_from_values(
                Some(PathBuf::from("/home/alice")),
                Some(PathBuf::from("/Users/alice")),
                Some("C:".to_string()),
                Some("\\Users\\alice".to_string()),
            ),
            Some(PathBuf::from("/home/alice"))
        );
        assert_eq!(
            resolve_home_dir_from_values(
                None,
                Some(PathBuf::from("C:\\Users\\alice")),
                Some("D:".to_string()),
                Some("\\Users\\fallback".to_string()),
            ),
            Some(PathBuf::from("C:\\Users\\alice"))
        );
        assert_eq!(
            resolve_home_dir_from_values(
                None,
                None,
                Some("C:".to_string()),
                Some("\\Users\\alice".to_string()),
            ),
            Some(PathBuf::from("C:\\Users\\alice"))
        );
    }

    #[test]
    fn resolve_home_dir_ignores_empty_values() {
        assert_eq!(
            resolve_home_dir_from_values(
                Some(PathBuf::new()),
                Some(PathBuf::new()),
                Some(String::new()),
                Some(String::new()),
            ),
            None
        );
    }
}
