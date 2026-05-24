use regex::Regex;
use std::cmp::Ordering;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

const DEBIAN_KERNEL_PKG_REGEX: &str = r"linux-image-([0-9]+\.[0-9]+\.[0-9]+)";
const KERNEL_VERSION_REGEX: &str =
    r"BOOT_IMAGE=(?:\([a-z0-9]+,[a-z0-9]+\))?/vmlinuz-([0-9]+\.[0-9]+\.[0-9]+(-[0-9]+)?)";
const UBUNTU_KERNEL_PKG_REGEX: &str = r"linux-image-([0 - 9] + \.[0 - 9] + \.[0 -9] + - [0 - 9] +)";

pub struct KernelInfo {
    pub latest: String,
    pub running: String,
}

fn get_os_id() -> Result<String, String> {
    let file = File::open("/etc/os-release");
    let reader = BufReader::new(file.unwrap());

    for line in reader.lines() {
        let line = line.unwrap();
        if !line.starts_with("ID=") {
            continue;
        }

        let fields = line.split("=").collect::<Vec<&str>>();
        if fields.len() != 2 {
            return Err(String::from(format!(
                "Unable to parse OS ID line: {}",
                line
            )));
        }

        return Ok(fields.get(1).unwrap().to_string());
    }

    Err(String::from("Unable to determine OS ID"))
}

fn apt_kernel_packages(regex_str: &str) -> Result<Vec<String>, String> {
    let re = Regex::new(regex_str).unwrap();
    let cmd = Command::new("dpkg")
        .arg("--list")
        .output()
        .expect("failed to execute dpkg");
    let output = String::from_utf8(cmd.stdout).unwrap();

    let mut pkgs = vec![];
    for line in output.lines().skip(5) {
        let fields = line.split(" ").collect::<Vec<&str>>();
        if fields.len() != 2 {
            return Err(String::from(format!(
                "Unable to packages from line {}",
                line
            )));
        };

        let package = fields.get(1).unwrap();
        let matches = re.find(package);
        if matches.is_some() {
            pkgs.push(package.to_string());
        }
    }

    Ok(pkgs)
}
fn dnf_kernel_packages() -> Result<Vec<String>, String> {
    let cmd = Command::new("dnf")
        .args(vec!["list", "--installed", "kernel"])
        .output()
        .expect("failed to execute dnf");
    let output = String::from_utf8_lossy(&cmd.stdout);

    let by_space = Regex::new(r"\s+").unwrap();
    let mut pkgs = vec![];
    for line in output.lines().skip(1) {
        if line.is_empty() {
            continue;
        }

        let fields = by_space.split(line).collect::<Vec<&str>>();
        if fields.len() != 3 {
            return Err(format!("Unable to determine version from line {}", line));
        }

        let version = fields.get(1).unwrap();
        let fields = version.rsplitn(2, ".").collect::<Vec<&str>>();
        let pkg = fields.get(1).unwrap();
        pkgs.push(pkg.to_string());
    }

    Ok(pkgs)
}

fn compare_versions(a: &String, b: &String) -> Ordering {
    let by_delim = if a.contains("-") { r"\.|-" } else { r"\." };
    let re = Regex::new(&by_delim).unwrap();

    let a_fields = re.split(a).collect::<Vec<&str>>();
    let b_fields = re.split(b).collect::<Vec<&str>>();
    let field_len = a_fields.len();

    for (index, field) in a_fields.iter().enumerate() {
        let b_field = b_fields.get(index).unwrap();

        let a_num = field.parse::<usize>().unwrap();
        let b_num = b_field.parse::<usize>().unwrap();

        let cmp = a_num.cmp(&b_num);
        if index == field_len - 1 {
            return cmp;
        } else if a_num != b_num {
            return cmp;
        }
    }

    Ordering::Equal
}

fn get_running_kernel() -> Result<String, String> {
    let cmd_fs = fs::read_to_string("/proc/cmdline").unwrap();
    let cmdline = cmd_fs.trim();
    let re = Regex::new(KERNEL_VERSION_REGEX).unwrap();
    match re.captures(cmdline) {
        Some(m) => Ok(m.get(1).unwrap().as_str().to_string()),
        None => Err(String::from("Unable to determine running kernel version")),
    }
}

pub fn kernel_info() -> Result<KernelInfo, String> {
    let os_id = match get_os_id() {
        Ok(os_id) => os_id,
        Err(why) => return Err(why),
    };

    let kernel_packages = match os_id.as_str() {
        "debian" => apt_kernel_packages(DEBIAN_KERNEL_PKG_REGEX),
        "fedora" => dnf_kernel_packages(),
        "ubuntu" => apt_kernel_packages(UBUNTU_KERNEL_PKG_REGEX),
        _ => Err(format!("Unknown OS ID {}", os_id)),
    };

    let mut pkgs = kernel_packages?;
    pkgs.sort_by(compare_versions);
    let most_recent = pkgs.last().unwrap();

    let running_kernel = match get_running_kernel() {
        Ok(running_kernel) => running_kernel,
        Err(why) => return Err(why),
    };

    Ok(KernelInfo{latest: most_recent.clone(), running: running_kernel})
}
