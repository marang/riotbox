//! Explicit, source-free Session metadata migration for selected legacy captures.
use riotbox_app::jam_app::migrate_legacy_capture_identities;
use riotbox_core::ids::CaptureId;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (session, ids, accept) = parse_args(std::env::args().skip(1))?;
    let entries = migrate_legacy_capture_identities(&session, &ids, accept)?;
    println!("{}", serde_json::to_string_pretty(&entries)?);
    eprintln!(
        "{}; adopted identities protect current bytes only, not historical authenticity",
        if accept {
            "migration completed"
        } else {
            "preview only; use --accept-current-content to adopt"
        }
    );
    Ok(())
}

fn parse_args(
    mut args: impl Iterator<Item = String>,
) -> Result<(PathBuf, Vec<CaptureId>, bool), Box<dyn std::error::Error>> {
    let session = PathBuf::from(args.next().ok_or("usage: capture_identity_migrate SESSION --capture ID [--capture ID ...] [--accept-current-content]")?);
    let mut ids = Vec::new();
    let mut accept = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--capture" => ids.push(CaptureId::from(
                args.next().ok_or("--capture requires an ID")?,
            )),
            "--accept-current-content" if !accept => accept = true,
            _ => return Err(format!("unknown or repeated argument: {arg}").into()),
        }
    }
    if ids.is_empty() {
        return Err("select at least one --capture ID".into());
    }
    Ok((session, ids, accept))
}

#[cfg(test)]
mod tests {
    use super::parse_args;

    #[test]
    fn preview_default_and_explicit_acceptance() {
        let args = ["session.json", "--capture", "cap-01"];
        let (_, ids, accept) = parse_args(args.into_iter().map(String::from)).unwrap();
        assert_eq!(ids.len(), 1);
        assert!(!accept);
        let (_, ids, accept) = parse_args(
            args.into_iter()
                .chain(["--capture", "cap-02", "--accept-current-content"])
                .map(String::from),
        )
        .unwrap();
        assert_eq!(ids.len(), 2);
        assert!(accept);
    }

    #[test]
    fn malformed_arguments_do_not_reach_migration() {
        for args in [
            vec![],
            vec!["session.json"],
            vec!["session.json", "--capture"],
            vec!["session.json", "--unknown"],
            vec![
                "session.json",
                "--accept-current-content",
                "--accept-current-content",
            ],
        ] {
            assert!(parse_args(args.into_iter().map(String::from)).is_err());
        }
    }
}
