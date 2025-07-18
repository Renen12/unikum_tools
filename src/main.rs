pub mod server;

use std::{
    env::args,
    fs,
    process::{Command, exit},
};

use serde_json::Value;
pub fn return_server_values(
    jsessionid: &String,
    unihzsessid: &String,
    shibsession_name: &String,
    shibsession_value: &String,
    pid: &String,
) -> String {
    let out = Command::new("pwsh")
        .args([
            "-NoProfile",
            "-Command",
            &format!(
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; ./return_json_posts.ps1 '{}' '{}' '{}' '{}' '{}'",
                jsessionid,
                unihzsessid,
                shibsession_name,
                shibsession_value,
                pid
            ),
        ])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e)).unwrap();
    let out_readable = String::from_utf8_lossy(&out.stdout).to_string();
    out_readable
}
#[allow(unused)]
pub fn return_server_values_messages(bearer: &String, pid: &String) -> String {
    let mut cmd = &fs::read_to_string("curl_messages.txt").unwrap();
    let binding = cmd.replace("_REPLACE", &bearer);
    cmd = &binding;
    let binding = cmd.replace("_PID", pid);
    cmd = &binding;
    let out = Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map_err(|e| println!("Cannot run cURL: {}", e))
        .unwrap();
    let string = String::from_utf8_lossy(&out.stdout).to_string();
    string
}
use crate::server::server;
static HELP_MESSAGE: &'static str = "
--help — display this message\n 
\t--shibsession_name=[your shibsession cookie's name]\n
\t--shibsession_value=[your shibsession cookie's value]\n
\t--jsessionid=[your jsessionid]\n
 \t--unihzsessid=[your unihzsessid]\n
  \t--pid=[the pid of the blog page, e.g 11349120275]\n
  \t --html — output these into respective HTML files\n
  \t --dry — show raw data
  \t --server — run a server on port 7951 that returns data from Unikum
  Instructions on how to obtain these is in the README.md file. This must be run in a directory with the proper return_json.ps1 file.
  \n Such a file is provided in the repository.";
#[allow(unused)]
fn main() {
    let mut is_server = false;
    let args: Vec<_> = args().collect();
    let mut jsessionid = String::new();
    let mut dry = false;
    let mut html = false;
    let mut unihzsessid = String::new();
    let mut shibsession_name = String::new();
    let mut shibsession_value = String::new();
    let mut pid = String::new();
    let pwsh_contents = fs::read_to_string("return_json_posts.ps1").unwrap();
    for arg in args {
        let split_equals: Vec<&str> = arg.split("=").collect();
        if arg == "--dry" {
            dry = true;
        }
        if arg == "--html" {
            html = true;
        }
        if arg.contains("--jsessionid") {
            jsessionid = split_equals[1].to_owned();
        }
        if arg.contains("--shibsession_name") {
            shibsession_name = split_equals[1].to_owned();
        }
        if arg.contains("--shibsession_value") {
            shibsession_value = split_equals[1].to_owned();
        }
        if arg.contains("--unihzsessid") {
            unihzsessid = split_equals[1].to_owned();
        }
        if arg.contains("--pid") {
            pid = split_equals[1].to_owned();
        }
        if arg == "--help" {
            print!("{}", HELP_MESSAGE);
            exit(0);
        }
        if arg == "--server" {
            is_server = true;
            server();
        }
    }
    if !jsessionid.is_empty()
        || !unihzsessid.is_empty()
        || !pid.is_empty()
        || !shibsession_name.is_empty()
        || !shibsession_value.is_empty()
    {
        let out = Command::new("pwsh")
            .args([
                "return_json.ps1",
                &jsessionid,
                &unihzsessid,
                &shibsession_name,
                &shibsession_value,
                &pid,
            ])
            .output()
            .unwrap();
        let out_readable = String::from_utf8(out.stdout).unwrap();
        if dry {
            println!("{}", out_readable);
        } else {
            let json: Value = match serde_json::from_str(&out_readable) {
                Ok(v) => v,
                Err(_) => {
                    eprintln!("Cannot read JSON from Unikum.");
                    exit(1);
                }
            };
            let main_list = match json["list"].as_array() {
                Some(v) => v,
                None => {
                    eprintln!("Cannot turn the main JSON list into an array");
                    exit(1);
                }
            };
            let mut item = 1;
            for blog in main_list {
                let mut content = String::from(blog["contentHTML"].as_str().unwrap());
                content.push_str(
                    "<script>// Injected by the rust wrapper script — to fix the links
                for (let x of document.querySelectorAll(\"a\")) {
                    if (x.href.includes(\"/unikum/content\")) {
                        x.href = \"https://start.unikum.net\" + x.getAttribute(\"data-file-url\");
                }
                }</script>",
                );
                fs::write(format!("{item}.html"), content).unwrap();
                item += 1;
            }
        }
    } else if is_server {
    } else {
        eprintln!("Not enough arguments");
        exit(1);
    }
}
