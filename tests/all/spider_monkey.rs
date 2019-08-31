#![cfg(test)]
use flate2::read::GzDecoder;
use ressa::{Error, Builder};
use std::path::Path;
use walkdir::WalkDir;
#[cfg(windows)]
static ESPARSE: &str = "node_modules/.bin/esparse.cmd";
#[cfg(not(windows))]
static ESPARSE: &str = "node_modules/.bin/esparse";

#[test]
fn moz_central() {
    let moz_central_path = Path::new("./moz-central");
    if !moz_central_path.exists() {
        get_moz_central_test_files(&moz_central_path);
    }
    let failures = walk(&moz_central_path);
    let fail_count = failures
        .iter()
        .filter(|(_, white_list)| !white_list)
        .count();
    for (msg, _) in failures.iter().filter(|(_, white_list)| *white_list) {
        eprintln!("W-{}", msg);
    }
    if fail_count > 0 {
        eprintln!("----------");
        eprintln!("FAILURES");
        eprintln!("----------");
        for (msg, _) in failures.iter().filter(|(_, white_list)| !white_list) {
            eprintln!("{}", msg);
        }
        panic!("Failed to parse {} moz_central files", fail_count);
    }
}

fn walk(path: &Path) -> Vec<(String, bool)> {
    let mut ret = Vec::new();
    for file_path in WalkDir::new(path).into_iter() {
        let file_path = file_path.unwrap();
        if file_path.path().is_file() {
            let test = if let Some(ext) = file_path.path().extension() {
                ext == "js"
            } else {
                false
            };
            if !test {
                continue;
            }
            if let Err(e) = run(&file_path.path()) {
                let loc = match &e {
                    Error::InvalidGetterParams(ref pos)
                    | Error::InvalidSetterParams(ref pos)
                    | Error::NonStrictFeatureInStrictContext(ref pos, _)
                    | Error::OperationError(ref pos, _)
                    | Error::Redecl(ref pos, _)
                    | Error::UnableToReinterpret(ref pos, _, _)
                    | Error::UnexpectedToken(ref pos, _) => format!(
                        "{}:{}:{}",
                        &file_path.path().to_str().unwrap(),
                        pos.line,
                        pos.column
                    ),
                    _ => format!("{}", file_path.path().display()),
                };
                let mut msg = format!("Parse Failure {}\n\t{}", e, loc);
                let white_list = match ::std::process::Command::new(
                    ESPARSE,
                )
                .arg(file_path.path())
                .output()
                {
                    Ok(op) => {
                        if !op.status.success() {
                            let mut msg2 = format!(
                                "esparse failure: \nstderr: {:?}",
                                String::from_utf8_lossy(&op.stderr)
                            );
                            msg2.push_str(&format!(
                                "stdout: {:?}",
                                String::from_utf8_lossy(&op.stdout)
                            ));
                            Some(msg2)
                        } else {
                            let name = file_path.file_name();
                            let mut out_path =
                                Path::new("failures")
                                    .join(name);
                            out_path.set_extension("json");
                            ::std::fs::write(
                                &out_path,
                                String::from_utf8_lossy(&op.stdout).to_string(),
                            )
                            .unwrap();
                            None
                        }
                    }
                    Err(e) => {
                        panic!("failed to exec esparse {}", e);
                    }
                };
                let white_list = if let Some(msg2) = white_list {
                    msg.push_str(&format!("\n{}", msg2));
                    true
                } else {
                    false
                };
                ret.push((msg, white_list));
            }
        } 
    }
    ret
}

fn run(file: &Path) -> Result<(), Error> {
    let mut contents = ::std::fs::read_to_string(file)?;
    if contents.starts_with("|") {
        // bad comment
        contents = format!("//{}", contents);
    }
    if let Some(first) = contents.lines().next() {
        if first.contains("error:InternalError")
        /*--> in last line*/
        {
            contents = contents.replace("-->", "//");
        }
    }
    let module = contents.starts_with("// |jit-test| module");
    let mut b = Builder::new();
    let parser = b.js(&contents).module(module).build()?;
    for part in parser {
        let _part = part?;
    }
    Ok(())
}

fn get_moz_central_test_files(path: &Path) {
    let mut response = reqwest::get(
        "https://hg.mozilla.org/mozilla-central/archive/tip.tar.gz/js/src/jit-test/tests/",
    )
    .expect("Failed to get zip of moz-central");
    let mut buf = Vec::new();
    response
        .copy_to(&mut buf)
        .expect("failed to copy to BzDecoder");
    let gz = GzDecoder::new(buf.as_slice());
    let mut t = tar::Archive::new(gz);
    t.unpack(path).expect("Failed to unpack gz");
}
