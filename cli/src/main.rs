use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use tesseract_rs::{TessPageSegMode, TesseractAPI};

const USAGE: &str = "\
tesseract-rs — OCR from the command line

Usage:
  tesseract-rs [OPTIONS] <IMAGE>

Options:
  -l, --lang <LANG>     Language(s) to use (default: eng)
  -p, --psm <MODE>      Page segmentation mode 0-13 (default: 3 = auto)
  -t, --tessdata <DIR>  Tessdata directory (default: TESSDATA_PREFIX, then
                        `tesseract --print-tessdata-dir`, then the
                        compiled-in default)
  -o, --output <FMT>    Output format: txt, hocr, tsv (default: txt)
  -v, --version         Print version and exit
  -h, --help            Print this help and exit
";

struct Options {
    image: PathBuf,
    lang: String,
    psm: Option<i32>,
    tessdata: Option<PathBuf>,
    output: OutputFormat,
}

enum OutputFormat {
    Txt,
    Hocr,
    Tsv,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let opts = match parse_args(&args) {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("error: {err}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match run(&opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut image = None;
    let mut lang = "eng".to_string();
    let mut psm = None;
    let mut tessdata = None;
    let mut output = OutputFormat::Txt;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-h" | "--help" => {
                print!("{USAGE}");
                std::process::exit(0);
            }
            "-v" | "--version" => {
                println!("tesseract-rs {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "-l" | "--lang" => {
                i += 1;
                lang = args.get(i).ok_or("--lang requires a value")?.clone();
            }
            "-p" | "--psm" => {
                i += 1;
                psm = Some(parse_psm(args.get(i).ok_or("--psm requires a value")?)?);
            }
            "-t" | "--tessdata" => {
                i += 1;
                tessdata = Some(PathBuf::from(
                    args.get(i).ok_or("--tessdata requires a value")?,
                ));
            }
            "-o" | "--output" => {
                i += 1;
                output = parse_output(args.get(i).ok_or("--output requires a value")?)?;
            }
            _ if arg.starts_with("--lang=") => lang = arg["--lang=".len()..].to_string(),
            _ if arg.starts_with("--psm=") => psm = Some(parse_psm(&arg["--psm=".len()..])?),
            _ if arg.starts_with("--tessdata=") => {
                tessdata = Some(PathBuf::from(&arg["--tessdata=".len()..]))
            }
            _ if arg.starts_with("--output=") => output = parse_output(&arg["--output=".len()..])?,
            _ if arg.starts_with('-') && arg.len() > 1 => {
                return Err(format!("unknown option: {arg}"));
            }
            _ => {
                if image.is_some() {
                    return Err("only one image file is supported".to_string());
                }
                image = Some(PathBuf::from(arg));
            }
        }
        i += 1;
    }

    Ok(Options {
        image: image.ok_or("missing IMAGE argument (see --help)")?,
        lang,
        psm,
        tessdata,
        output,
    })
}

fn parse_psm(value: &str) -> Result<i32, String> {
    let psm = value
        .parse::<i32>()
        .map_err(|_| format!("invalid --psm value: {value} (expected 0-13)"))?;
    if !(0..=13).contains(&psm) {
        return Err(format!("invalid --psm value: {psm} (expected 0-13)"));
    }
    Ok(psm)
}

fn parse_output(value: &str) -> Result<OutputFormat, String> {
    match value {
        "txt" => Ok(OutputFormat::Txt),
        "hocr" => Ok(OutputFormat::Hocr),
        "tsv" => Ok(OutputFormat::Tsv),
        other => Err(format!(
            "unknown output format: {other} (expected txt, hocr or tsv)"
        )),
    }
}

fn run(opts: &Options) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(&opts.image)?.to_luma8();
    let (width, height) = img.dimensions();

    let tessdata = match opts.tessdata.clone().or_else(resolve_tessdata) {
        Some(dir) => dir,
        None => {
            return Err(
                "unable to find a tessdata directory; pass --tessdata <DIR> or set TESSDATA_PREFIX"
                    .into(),
            );
        }
    };

    let api = TesseractAPI::new();
    api.init(tessdata.to_str().unwrap_or(""), &opts.lang)?;

    if let Some(psm) = opts.psm {
        api.set_page_seg_mode(TessPageSegMode::from_int(psm))?;
    }

    api.set_image(img.as_raw(), width as i32, height as i32, 1, width as i32)?;

    let text = match opts.output {
        OutputFormat::Txt => api.get_utf8_text()?,
        OutputFormat::Hocr => api.get_hocr_text(0)?,
        OutputFormat::Tsv => api.get_tsv_text(0)?,
    };
    print!("{text}");

    Ok(())
}

/// Resolve the tessdata directory, in order:
/// 1. `TESSDATA_PREFIX` env var
/// 2. `tesseract --print-tessdata-dir` (supported by some builds)
/// 3. Common locations (bundled-build dirs, Homebrew, distro defaults),
///    validated by the presence of `eng.traineddata`
fn resolve_tessdata() -> Option<PathBuf> {
    if let Ok(dir) = env::var("TESSDATA_PREFIX") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }

    if let Ok(output) = Command::new("tesseract")
        .arg("--print-tessdata-dir")
        .output()
    {
        if output.status.success() {
            let dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if is_absolute_path(&dir) {
                return Some(PathBuf::from(dir));
            }
        }
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    if cfg!(target_os = "macos") {
        if let Ok(home) = env::var("HOME") {
            candidates.push(
                PathBuf::from(home).join("Library/Application Support/tesseract-rs/tessdata"),
            );
        }
        candidates.push(PathBuf::from("/opt/homebrew/share/tessdata"));
        candidates.push(PathBuf::from("/usr/local/share/tessdata"));
        candidates.push(PathBuf::from("/usr/share/tessdata"));
    } else if cfg!(target_os = "linux") || cfg!(target_os = "freebsd") {
        if let Ok(home) = env::var("HOME") {
            candidates.push(PathBuf::from(home).join(".tesseract-rs/tessdata"));
        }
        candidates.push(PathBuf::from("/usr/share/tessdata"));
        candidates.push(PathBuf::from("/usr/share/tesseract-ocr/5/tessdata"));
        candidates.push(PathBuf::from("/usr/share/tesseract-ocr/4.00/tessdata"));
    } else if cfg!(target_os = "windows") {
        if let Ok(appdata) = env::var("APPDATA") {
            candidates.push(PathBuf::from(appdata).join("tesseract-rs/tessdata"));
        }
    }

    candidates
        .into_iter()
        .find(|dir| dir.join("eng.traineddata").exists())
}

fn is_absolute_path(value: &str) -> bool {
    value.starts_with('/')
        || (cfg!(target_os = "windows") && value.len() > 2 && value.as_bytes()[1] == b':')
}
