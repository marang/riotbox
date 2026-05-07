#[derive(Debug, PartialEq, Eq)]
struct Args {
    observer_path: PathBuf,
    manifest_path: PathBuf,
    output_path: Option<PathBuf>,
    require_evidence: bool,
    json_output: bool,
    show_help: bool,
}

impl Args {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut observer_path = None;
        let mut manifest_path = None;
        let mut output_path = None;
        let mut require_evidence = false;
        let mut json_output = false;
        let mut show_help = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--help" | "-h" => show_help = true,
                "--require-evidence" => require_evidence = true,
                "--json" => json_output = true,
                "--observer" => {
                    observer_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--observer requires a path".to_string())?,
                    ));
                }
                "--manifest" => {
                    manifest_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--manifest requires a path".to_string())?,
                    ));
                }
                "--output" => {
                    output_path = Some(PathBuf::from(
                        args.next()
                            .ok_or_else(|| "--output requires a path".to_string())?,
                    ));
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        if show_help {
            return Ok(Self {
                observer_path: PathBuf::new(),
                manifest_path: PathBuf::new(),
                output_path,
                require_evidence,
                json_output,
                show_help,
            });
        }

        Ok(Self {
            observer_path: observer_path.ok_or_else(|| "--observer is required".to_string())?,
            manifest_path: manifest_path.ok_or_else(|| "--manifest is required".to_string())?,
            output_path,
            require_evidence,
            json_output,
            show_help,
        })
    }
}

fn print_help() {
    println!(
        "Usage: observer_audio_correlate --observer PATH --manifest PATH [--output PATH] [--json]\n\
         \n\
         Reads a riotbox-app observer NDJSON file and an audio QA manifest.json,\n\
         then emits a compact Markdown correlation summary, or JSON with --json.\n\
         This is local-first QA bookkeeping, not a live host-session monitor.\n\
         \n\
         Pass --require-evidence to fail when the observer stream is malformed,\n\
         the manifest envelope is unstable, committed control-path evidence is\n\
         missing, or passing output-path manifest evidence is missing."
    );
}
