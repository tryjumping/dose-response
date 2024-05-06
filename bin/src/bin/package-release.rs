use std::{
    env,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use flate2::{write::GzEncoder, Compression};

use walkdir::WalkDir;

use zip::write::{SimpleFileOptions, ZipWriter};

const APP_NAME: &str = "dose-response";
const OUT_DIR: &str = "target/publish";

/// Return a given environment variable or the provided default value.
fn getenv(env_name: &str, default: &str) -> String {
    env::var(env_name).unwrap_or(String::from(default))
}

/// Convert the given text into the target platform's line endings.
///
/// That's `CRLF` or `\r\n` on Windows and `LF` (`\n`) on unix (linux
/// & macos).
fn platform_line_ending(source: &str) -> String {
    let line_ending = if cfg!(unix) { "\n" } else { "\r\n" };
    source.replace("\n", line_ending)
}

fn file_name_from_path(path: &Path) -> anyhow::Result<String> {
    use std::ffi::OsStr;
    path.file_name()
        .map(OsStr::to_str)
        .flatten()
        .map(String::from)
        .ok_or(anyhow::anyhow!(
            "Failed to get file name from path: {}",
            path.display()
        ))
}

fn publish_text_file(source_path: &Path, destination_path: Option<PathBuf>) -> anyhow::Result<()> {
    let source_filename = file_name_from_path(source_path)?;
    let out_dir = Path::new(OUT_DIR);
    let destination_path = destination_path.unwrap_or(out_dir.join(source_filename));
    println!(
        "Processing text file: `{}` to: `{}`",
        source_path.display(),
        destination_path.display()
    );
    let mut source_file = File::open(source_path)?;
    let mut contents = String::new();
    let source_size = source_file.metadata()?.len();
    let read_size = source_file.read_to_string(&mut contents)? as u64;
    assert_eq!(read_size, source_size);

    let contents = platform_line_ending(&contents);

    let mut destination_file = File::create_new(destination_path)?;
    destination_file.write_all(contents.as_bytes())?;

    Ok(())
}

fn publish_bin_file(source_path: &Path, destination_path: Option<PathBuf>) -> anyhow::Result<()> {
    let source_filename = file_name_from_path(source_path)?;
    let out_dir = Path::new(OUT_DIR);
    let destination_path = destination_path.unwrap_or(out_dir.join(&source_filename));
    let destination_path = if destination_path.exists() && destination_path.is_dir() {
        destination_path.join(&source_filename)
    } else {
        destination_path
    };

    println!(
        "Copying file: `{}` to: `{}`",
        source_path.display(),
        destination_path.display()
    );

    fs::copy(source_path, destination_path)?;

    Ok(())
}

fn publish_text(contents: &str, destination_path: &Path) -> anyhow::Result<()> {
    let destination_path = Path::new(OUT_DIR).join(destination_path);
    println!("Creating text file: `{}`", destination_path.display());
    fs::write(destination_path, platform_line_ending(contents))?;

    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum OS {
    Linux,
    Windows,
    MacOs,
}

fn main() -> anyhow::Result<()> {
    let bucket_name = getenv("BUCKET_NAME", "missing-bucket");
    let region_name = getenv("AWS_REGION", "missing-region");
    let aws_access_key_id = getenv("AWS_ACCESS_KEY_ID", "");
    let aws_secret_access_key = getenv("AWS_SECRET_ACCESS_KEY", "");
    let target_triple = current_platform::CURRENT_PLATFORM;
    let commit_hash = getenv("GITHUB_SHA", "master");
    let run_id = getenv("GITHUB_RUN_ID", "RUN-ID");
    let run_number = getenv("GITHUB_RUN_NUMBER", "RUN-NUMBER");

    // NOTE: ref format examples:
    // Pull request: refs/pull/13/merge
    // Push to master: refs/heads/master
    // Tag/release: refs/tags/v2.0.0-ci-16
    let github_ref = getenv("GITHUB_REF", "refs/tags/v0.0.1-test");

    println!("Target triple: {target_triple}");
    println!("Commit hash: {commit_hash}");
    println!("Run ID: {run_id}");
    println!("Run number: {run_number}");
    println!("Github ref: {github_ref}");

    let os = match target_triple {
        "x86_64-unknown-linux-gnu" => OS::Linux,
        "x86_64-pc-windows-msvc" => OS::Windows,
        "x86_64-apple-darwin" => OS::MacOs,
        "aarch64-apple-darwin" => OS::MacOs,
        _ => unimplemented!(),
    };

    let upload_release;
    let release_destination;
    let release_version;

    if github_ref == "refs/heads/master" || github_ref == "refs/heads/main" {
        println!("This is a `latest`: the job merging into the main branch.");
        upload_release = true;
        release_destination = "nightlies";
        let today = chrono::Utc::now().format("%F");
        release_version = format!("{today}-{run_number}");
    } else if github_ref.starts_with("refs/tags/v") {
        let tag_name = github_ref.replace("refs/tags/", "");
        println!("This is a release tag: {tag_name}");
        upload_release = true;
        release_destination = "releases";
        release_version = tag_name;
    } else {
        println!("The ref `{github_ref}` is neither a tag nor push to the main branch.");
        upload_release = false;
        release_destination = "unexpected";
        release_version = "unexpected".to_string();
    }

    let exe_name = {
        match os {
            OS::Windows => format!("{APP_NAME}.exe"),
            _ => APP_NAME.to_string(),
        }
    };

    let debug_script = match os {
        OS::Windows => "Debug.bat",
        _ => "debug.sh",
    };

    let full_version = format!("{APP_NAME}-{release_version}-{target_triple}");

    println!("Operating system: {:?}", os);
    println!("Upload the release: {upload_release}");
    println!("Release destination: {release_destination}");
    println!("Executable file name: {exe_name}");
    println!("Debug script file name: {debug_script}");
    println!("Full version: {full_version}");

    let target_release_dir = Path::new("target/release");
    let out_dir = Path::new(OUT_DIR);

    // NOTE: need to remove any traces of the previous directory so
    // the output is not contaminated by previous runs:
    let _ = fs::remove_dir_all(out_dir);
    fs::create_dir_all(out_dir)?;
    fs::create_dir_all(out_dir.join("icons"))?;

    publish_text_file(Path::new("README.md"), Some(out_dir.join("README.txt")))?;
    publish_text_file(Path::new("COPYING.txt"), Some(out_dir.join("LICENSE.txt")))?;
    publish_text_file(Path::new("third-party-licenses.html"), None)?;

    publish_bin_file(Path::new(debug_script), None)?;

    let version_contents = format!(
        "Version: {release_version}\nFull Version: {full_version}\nCommit: {commit_hash}\n"
    );
    publish_text(&version_contents, Path::new("VERSION.txt"))?;

    publish_bin_file(&target_release_dir.join(&exe_name), None)?;

    // NOTE: Add game icons
    for entry in WalkDir::new("assets") {
        let entry = entry?;
        let file_name = file_name_from_path(entry.path())?;
        if file_name.starts_with("icon") {
            publish_bin_file(entry.path(), Some(out_dir.join("icons")))?;
        }
    }

    let archive_file_name;
    let archive_path;
    let archive_directory_name = &format!("Dose Response {release_version}");

    // NOTE: Build the archive
    match os {
        OS::Linux => {
            archive_file_name = format!("{full_version}.tar.gz");
            archive_path = format!("target/{archive_file_name}");
            let tar_gz = File::create(&archive_path)?;
            let enc = GzEncoder::new(tar_gz, Compression::default());
            let mut tar = tar::Builder::new(enc);

            tar.append_dir_all(archive_directory_name, OUT_DIR)?;
        }

        OS::Windows | OS::MacOs => {
            archive_file_name = format!("{full_version}.zip");
            archive_path = format!("target/{archive_file_name}");
            let zip_file = File::create(&archive_path)?;
            let mut zip = ZipWriter::new(zip_file);

            zip.add_directory(archive_directory_name, SimpleFileOptions::default())?;
            zip.add_directory_from_path(
                Path::new(archive_directory_name).join("icons"),
                SimpleFileOptions::default(),
            )?;

            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            for entry in WalkDir::new(OUT_DIR) {
                let entry = entry?;
                if entry.file_type().is_file() {
                    let p = entry.path().strip_prefix(OUT_DIR)?;
                    let destination_path = Path::new(archive_directory_name).join(p);
                    println!(
                        "{} -> {}",
                        entry.path().display(),
                        destination_path.display()
                    );

                    // Get the file's permission since we need to set
                    // that explicitly for zip.
                    //
                    // Without this, the executable bit won't be set properly.
                    #[cfg(unix)]
                    let options = {
                        use std::os::unix::fs::PermissionsExt;
                        let permissions = entry.path().metadata()?.permissions().mode();
                        options.unix_permissions(permissions)
                    };

                    zip.start_file_from_path(destination_path, options)?;
                    let mut contents = vec![];
                    let mut f = File::open(entry.path())?;
                    f.read_to_end(&mut contents)?;
                    zip.write_all(&contents)?;
                }
            }

            zip.finish()?;
        }
    };

    println!("Archive path: {archive_path}");

    if upload_release {
        if aws_access_key_id.is_empty() || aws_secret_access_key.is_empty() {
            println!("AWS credentials not provided, skipping release upload.");
        } else {
            use rusty_s3::{actions, Bucket, Credentials, S3Action, UrlStyle};
            println!("Uploading the release to S3...");
            let endpoint = format!("https://s3.{region_name}.amazonaws.com/").parse()?;
            let path_style = UrlStyle::VirtualHost;
            let bucket = Bucket::new(endpoint, path_style, bucket_name, region_name)?;
            let creds = Credentials::new(aws_access_key_id, aws_secret_access_key);

            let object_name =
                format!("/{release_destination}/{release_version}/{archive_file_name}");
            println!("Uploading to: {object_name}");

            let action = actions::PutObject::new(&bucket, Some(&creds), &object_name);
            let ten_minutes = std::time::Duration::from_secs(600);
            let signed_url = action.sign(ten_minutes);

            let archive_file = File::open(archive_path)?;
            let client = reqwest::blocking::Client::new()
                .put(signed_url)
                .body(archive_file);
            let res = client.send()?;

            if res.status() == 200 {
                println!("Release archive uploaded successfully.");
            } else {
                dbg!(res);
                anyhow::bail!("Failed to upload the release.");
            }
        }
    } else {
        println!("Release upload not requested, we're done here.");
    }

    Ok(())
}
