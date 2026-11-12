//! Build and package a Rust client-side Wasm library without foreign bindgen tools.
use ocamljsify::{AsyncRuntime, Backend, Error, Options};
use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};
fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find(|v| v[0] == name).map(|v| v[1].clone())
}
// Restore generated structural object fields after the compiler's syntax printer.
// Ordinary source strings and native record fields are left untouched.
fn rescript_object_fields(source: &str) -> Result<String, Error> {
    const PREFIX: &str = "__ocamljs_property_";
    let bytes = source.as_bytes();
    let mut output = String::new();
    let (mut at, mut copied) = (0, 0);
    let mut previous = None;
    while at < bytes.len() {
        if bytes[at..].starts_with(b"/*") {
            let mut depth = 1;
            at += 2;
            while at < bytes.len() && depth > 0 {
                if bytes[at..].starts_with(b"/*") {
                    depth += 1;
                    at += 2;
                } else if bytes[at..].starts_with(b"*/") {
                    depth -= 1;
                    at += 2;
                } else {
                    at += 1;
                }
            }
            continue;
        }
        if bytes[at..].starts_with(b"//") {
            while at < bytes.len() && bytes[at] != b'\n' {
                at += 1;
            }
            continue;
        }
        if matches!(bytes[at], b'"' | b'`') {
            let quote = bytes[at];
            let start = at;
            at += 1;
            while at < bytes.len() {
                if bytes[at] == b'\\' {
                    at = (at + 2).min(bytes.len());
                } else if bytes[at] == quote {
                    at += 1;
                    break;
                } else {
                    at += 1;
                }
            }
            output.push_str(&source[copied..start]);
            let token = &source[start..at];
            let marker = quote == b'"'
                && token.ends_with('"')
                && matches!(previous, Some(b'{' | b','))
                && source[at..].trim_start().starts_with(':');
            let hex = marker
                .then(|| token[1..token.len() - 1].strip_prefix(PREFIX))
                .flatten();
            if let Some(hex) = hex.filter(|hex| {
                hex.len() % 2 == 0 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
            }) {
                let decoded = (0..hex.len())
                    .step_by(2)
                    .map(|index| u8::from_str_radix(&hex[index..index + 2], 16))
                    .collect::<Result<Vec<_>, _>>()?;
                output.push_str(&serde_json::to_string(&String::from_utf8(decoded)?)?);
            } else {
                output.push_str(token);
            }
            copied = at;
            previous = Some(quote);
            continue;
        }
        if !bytes[at].is_ascii_whitespace() {
            previous = Some(bytes[at]);
        }
        at += 1;
    }
    output.push_str(&source[copied..]);
    Ok(output)
}

fn format_rescript(folder: &Path, args: &[String]) -> Result<(), Error> {
    let cwd = std::env::current_dir()?;
    let executable = if cfg!(windows) { "bsc.cmd" } else { "bsc" };
    let compiler = option(args, "--rescript-compiler")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("BSC").map(PathBuf::from))
        .unwrap_or_else(|| {
            cwd.ancestors()
                .map(|dir| dir.join("node_modules/.bin").join(executable))
                .find(|path| path.is_file())
                .unwrap_or_else(|| PathBuf::from(executable))
        });
    // Use the language compiler's own syntax printer, so this stays aligned
    // with the selected ReScript version rather than maintaining a second parser.
    for (input, output) in [
        ("Bindings.ml", "Bindings.res"),
        ("Bindings.mli", "Bindings.resi"),
    ] {
        let formatted = Command::new(&compiler)
            .arg("-format")
            .arg(folder.join(input))
            .output()
            .map_err(|e| {
                format!(
                    "ReScript formatting requires bsc; set BSC or --rescript-compiler PATH: {e}"
                )
            })?;
        if !formatted.status.success() {
            return Err(String::from_utf8_lossy(&formatted.stderr)
                .into_owned()
                .into());
        }
        std::fs::write(
            folder.join(output),
            rescript_object_fields(&String::from_utf8(formatted.stdout)?)?,
        )?;
    }
    Ok(())
}
fn run() -> Result<(), Error> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args.iter().any(|a| a == "--help") {
        println!(
            "ocamljs_pack build --manifest-path Cargo.toml [--package NAME] [--backend melange|js_of_ocaml|rescript|all] [--async-runtime promise|lwt] [--out-dir pkg] [--name NAME] [--dev | --profile PROFILE] [--paranoid] [--features FEATURES] [--rescript-compiler PATH]\nocamljs_pack interface --wasm input.wasm --out-dir api\nocamljs_pack bindings --interface api/interface.json --name NAME --out-dir bindings\nocamljs_pack package --wasm input.wasm --runtime bindings/bindings.runtime.mjs --out-dir pkg\nocamljs_pack generate --wasm input.wasm [--schema schema.json] --out-dir pkg [--backend BACKEND] [--async-runtime promise|lwt] [--dev] [--paranoid]"
        );
        return Ok(());
    }
    if matches!(args[0].as_str(), "interface" | "bindings" | "package") {
        let allowed: &[&str] = match args[0].as_str() {
            "interface" => &["--wasm", "--out-dir"],
            "bindings" => &["--interface", "--name", "--out-dir"],
            "package" => &["--wasm", "--runtime", "--out-dir"],
            _ => unreachable!(),
        };
        let mut flags = args[1..].chunks_exact(2);
        for pair in &mut flags {
            if !allowed.contains(&pair[0].as_str()) || pair[1].starts_with("--") {
                return Err(format!("invalid {} option: {}", args[0], pair[0]).into());
            }
        }
        if !flags.remainder().is_empty() {
            return Err("every option requires a value".into());
        }
        let out = PathBuf::from(option(&args, "--out-dir").ok_or("--out-dir is required")?);
        std::fs::create_dir_all(&out)?;
        match args[0].as_str() {
            "interface" => {
                let wasm = std::fs::read(option(&args, "--wasm").ok_or("--wasm is required")?)?;
                std::fs::write(
                    out.join("interface.json"),
                    ocamljsify::binding_input(&wasm)?,
                )?;
                std::fs::write(out.join("schema.json"), ocamljsify::schema_sidecar(&wasm)?)?;
            }
            "bindings" => {
                let input =
                    std::fs::read(option(&args, "--interface").ok_or("--interface is required")?)?;
                ocamljsify::generate_bindings(
                    &input,
                    &out,
                    &Options {
                        name: option(&args, "--name").unwrap_or("bindings".into()),
                        backend: Backend::Melange,
                        async_runtime: AsyncRuntime::Promise,
                        debug: false,
                        paranoid: false,
                    },
                )?;
            }
            "package" => {
                let wasm = std::fs::read(option(&args, "--wasm").ok_or("--wasm is required")?)?;
                let runtime = std::fs::read_to_string(
                    option(&args, "--runtime").ok_or("--runtime is required")?,
                )?;
                std::fs::write(
                    out.join("backend.wasm"),
                    ocamljsify::package_runtime(&wasm, &runtime, false)?,
                )?;
            }
            _ => unreachable!(),
        }
        return Ok(());
    }
    for arg in args.iter().filter(|arg| arg.starts_with("--")) {
        if !matches!(
            arg.as_str(),
            "--out-dir"
                | "--wasm"
                | "--schema"
                | "--name"
                | "--manifest-path"
                | "--package"
                | "--backend"
                | "--async-runtime"
                | "--profile"
                | "--features"
                | "--rescript-compiler"
                | "--paranoid"
                | "--dev"
        ) {
            return Err(format!("unknown option {arg}").into());
        }
    }
    let paranoid = args.iter().any(|arg| arg == "--paranoid")
        || std::env::var("OCAMLJS_PARANOID").as_deref() == Ok("1");
    let out = PathBuf::from(option(&args, "--out-dir").unwrap_or("pkg".into()));
    let mode = &args[0];
    let (wasm, name) = if mode == "generate" {
        let p = option(&args, "--wasm").ok_or("generate requires --wasm")?;
        (
            std::fs::read(&p)?,
            option(&args, "--name").unwrap_or("bindings".into()),
        )
    } else if mode == "build" {
        let manifest =
            PathBuf::from(option(&args, "--manifest-path").unwrap_or("Cargo.toml".into()))
                .canonicalize()?;
        let cargo = std::env::var_os("CARGO").unwrap_or("cargo".into());
        let output = Command::new(&cargo)
            .args([
                "metadata",
                "--no-deps",
                "--format-version",
                "1",
                "--manifest-path",
            ])
            .arg(&manifest)
            .output()?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
        }
        let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        let package = option(&args, "--package");
        let pkg = metadata["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| {
                package.as_ref().map_or_else(
                    || Path::new(p["manifest_path"].as_str().unwrap()) == manifest,
                    |n| p["name"] == *n,
                )
            })
            .ok_or("select a library package with --package")?;
        let target = pkg["targets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| {
                t["crate_types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|k| k == "cdylib")
            })
            .ok_or("add crate-type = [\"cdylib\", \"rlib\"] to [lib]")?;
        let target_name = target["name"].as_str().unwrap();
        let name = option(&args, "--name").unwrap_or(target_name.into());
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("--name must be a JavaScript identifier".into());
        }
        let dev = args.iter().any(|a| a == "--dev");
        let mut build = Command::new(&cargo);
        build
            .args([
                "build",
                "--locked",
                "--lib",
                "--target",
                "wasm32-unknown-unknown",
                "--manifest-path",
            ])
            .arg(&manifest)
            .args(["--package", pkg["name"].as_str().unwrap()]);
        let profile = option(&args, "--profile");
        if dev && profile.is_some() {
            return Err("--dev and --profile cannot be combined".into());
        }
        if args.iter().any(|arg| arg == "--profile")
            && profile.as_ref().is_none_or(|value| value.starts_with('-'))
        {
            return Err("--profile requires a Cargo profile name".into());
        }
        let profile = profile
            .as_deref()
            .unwrap_or(if dev { "dev" } else { "release" });
        build.args(["--profile", profile]);
        let mut features = option(&args, "--features").unwrap_or_default();
        if paranoid {
            if !features.is_empty() {
                features.push(',');
            }
            features.push_str("ocamljs_bindgen/paranoid");
        }
        if !features.is_empty() {
            build.args(["--features", &features]);
        }
        let status = build.status()?;
        if !status.success() {
            return Err("Rust Wasm build failed".into());
        }
        let artifact = Path::new(metadata["target_directory"].as_str().unwrap())
            .join("wasm32-unknown-unknown")
            .join(match profile {
                "dev" | "test" => "debug",
                "release" | "bench" => "release",
                custom => custom,
            })
            .join(format!("{target_name}.wasm"));
        (std::fs::read(artifact)?, name)
    } else {
        return Err(format!("unknown command {mode}").into());
    };
    let wasm = if let Some(sidecar) = option(&args, "--schema") {
        if mode != "generate" {
            return Err("--schema is only supported by generate".into());
        }
        ocamljsify::with_schema(&wasm, &std::fs::read(sidecar)?)?
    } else {
        wasm
    };
    let schema = ocamljsify::read(&wasm)?;
    let backend = option(&args, "--backend").unwrap_or("melange".into());
    let backends = match backend.as_str() {
        "all" => vec![
            ("melange", Backend::Melange),
            ("js_of_ocaml", Backend::JsOfOcaml),
            ("rescript", Backend::Rescript),
        ],
        "melange" => vec![("melange", Backend::Melange)],
        "js_of_ocaml" | "jsoo" => vec![("js_of_ocaml", Backend::JsOfOcaml)],
        "rescript" => vec![("rescript", Backend::Rescript)],
        _ => return Err("unknown backend".into()),
    };
    let async_runtime = match option(&args, "--async-runtime").as_deref() {
        None if args.iter().any(|arg| arg == "--async-runtime") => {
            return Err("--async-runtime requires promise or lwt".into());
        }
        None | Some("promise") => AsyncRuntime::Promise,
        Some("lwt") => AsyncRuntime::Lwt,
        Some(_) => return Err("--async-runtime must be promise or lwt".into()),
    };
    if async_runtime == AsyncRuntime::Lwt
        && !backends
            .iter()
            .any(|(_, b)| matches!(b, Backend::JsOfOcaml))
    {
        return Err("Lwt generation requires the js_of_ocaml backend".into());
    }
    for (label, backend) in backends {
        let folder = if option(&args, "--backend").as_deref() == Some("all") {
            out.join(label)
        } else {
            out.clone()
        };
        ocamljsify::generate(
            &schema,
            &wasm,
            &folder,
            &Options {
                name: name.clone(),
                backend,
                async_runtime: if matches!(backend, Backend::JsOfOcaml) {
                    async_runtime
                } else {
                    AsyncRuntime::Promise
                },
                debug: args.iter().any(|a| a == "--dev"),
                paranoid,
            },
        )?;
        if matches!(backend, Backend::Rescript) {
            format_rescript(&folder, &args)?;
        }
        println!(
            "generated {label} bindings in {} (schema {})",
            folder.display(),
            &schema.fingerprint[..12]
        );
    }
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ocamljs_pack: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod rescript_property_tests {
    use super::rescript_object_fields;
    #[test]
    fn restores_object_tokens_without_touching_comments_or_string_contents() {
        let source = r#"/* "__ocamljs_property_74797065": /* nested */ docs */
let text = "escaped \"__ocamljs_property_30\": text"
let quoted = `"__ocamljs_property_31": template`
// "__ocamljs_property_32": line comment
let annotated = "__ocamljs_property_33": string
type value = {"__ocamljs_property_74797065": string, "__ocamljs_property_30": bool}
"#;
        let expected = source.replace(r#"type value = {"__ocamljs_property_74797065": string, "__ocamljs_property_30": bool}"#, r#"type value = {"type": string, "0": bool}"#);
        assert_eq!(rescript_object_fields(source).unwrap(), expected);
    }
}
