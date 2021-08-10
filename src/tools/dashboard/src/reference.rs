// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0 OR MIT
//! Utilities to extract examples from
//! [The Rust Reference](https://doc.rust-lang.org/nightly/reference),
//! run them through RMC, and display their results.

use crate::{
    dashboard,
    litani::Litani,
    util::{self, FailStep, TestProps},
};
use pulldown_cmark::{Parser, Tag};
use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::{Debug, Formatter, Result, Write},
    fs::{self, File},
    hash::Hash,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use walkdir::WalkDir;

/// Parses the chapter/section hierarchy in the markdown file specified by
/// `summary_path` and returns a mapping from markdown files containing rust
/// code to corresponding directories where the extracted rust code should
/// reside.
fn parse_hierarchy(summary_path: &Path) -> HashMap<PathBuf, PathBuf> {
    let summary_dir = summary_path.parent().unwrap().to_path_buf();
    let start = "# The Rust Reference\n\n[Introduction](introduction.md)";
    let summary = fs::read_to_string(summary_path).unwrap();
    assert!(summary.starts_with(start), "Error: The start of the summary file changed.");
    // Skip the title and introduction.
    let n = Parser::new(start).count();
    let parser = Parser::new(&summary).skip(n);
    // Set "ref" as the root of the hierarchical path.
    let mut hierarchy: PathBuf = ["src", "test", "ref"].iter().collect();
    let mut map = HashMap::new();
    // Introduction is a especial case, so handle it separately.
    map.insert(summary_dir.join("introduction.md"), hierarchy.join("Introduction"));
    for event in parser {
        match event {
            pulldown_cmark::Event::End(Tag::Item) => {
                // Pop the current chapter/section from the hierarchy once
                // we are done processing it and its subsections.
                hierarchy.pop();
            }
            pulldown_cmark::Event::End(Tag::Link(_, path, _)) => {
                // At the start of the link tag, the hierarchy does not yet
                // contain the title of the current chapter/section. So, we wait
                // for the end of the link tag before adding the path and
                // hierarchy of the current chapter/section to the map.
                let mut full_path = summary_dir.clone();
                full_path.extend(path.split('/'));
                map.insert(full_path, hierarchy.clone());
            }
            pulldown_cmark::Event::Text(text) => {
                // Add the current chapter/section title to the hierarchy.
                hierarchy.push(text.to_string());
            }
            _ => (),
        }
    }
    map
}

/// The data structure represents the "full" path to examples in the Rust books.
#[derive(PartialEq, Eq, Hash)]
struct Example {
    /// Path to the markdown file containing the example.
    path: PathBuf,
    /// Line number of the code block introducing the example.
    line: usize,
}

impl Example {
    /// Creates a new [`Example`] instance representing "full" path to the
    /// Rust example.
    fn new(path: PathBuf, line: usize) -> Example {
        Example { path, line }
    }
}

impl Debug for Example {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_fmt(format_args!("{}:{}", self.path.to_str().unwrap(), self.line))
    }
}

/// Extracts examples from the markdown files specified by each key in the given
/// `map` and saves them in the directory specified by the corresponding value.
/// Returns a mapping from the original location of **_each_** example to the
/// path it was extracted to.
fn extract_examples(par_map: HashMap<PathBuf, PathBuf>) -> HashMap<Example, PathBuf> {
    let mut full_map = HashMap::new();
    for (par_from, par_to) in par_map {
        let pairs = extract(&par_from, &par_to);
        for (key, val) in pairs {
            full_map.insert(key, val);
        }
    }
    full_map
}

/// Extracts examples from the markdown files specified by `par_from` and saves
/// them in the directory specified by `par_to`. Returns a mapping from the
/// original location of **_each_** example to the path it was extracted to.
fn extract(par_from: &Path, par_to: &Path) -> Vec<(Example, PathBuf)> {
    let build_dir = &env::var("BUILD_DIR").unwrap();
    let triple = &env::var("TRIPLE").unwrap();
    // Create a temporary directory to save the files generated by `rustdoc`.
    let gen_dir: PathBuf = [build_dir, triple, "dashboard", "ref"].iter().collect();
    fs::create_dir_all(&gen_dir).unwrap();
    let mut cmd = Command::new("rustdoc");
    cmd.args([
        "+nightly",
        "--test",
        "-Z",
        "unstable-options",
        par_from.to_str().unwrap(),
        "--test-builder",
        &["src", "tools", "dashboard", "print.sh"].iter().collect::<PathBuf>().to_str().unwrap(),
        "--persist-doctests",
        gen_dir.to_str().unwrap(),
        "--no-run",
    ]);
    cmd.stdout(Stdio::null());
    cmd.spawn().unwrap().wait().unwrap();
    // Mapping from path and line number of rust example to where it was extracted to.
    let mut pairs = Vec::new();

    for dir in gen_dir.read_dir().unwrap() {
        // Some directories do not contain tests because the markdown file
        // instructs `rustdoc` to "ignore" those tests.
        let dir = dir.unwrap().path();
        if let Some(from) = dir.read_dir().unwrap().next() {
            // The path to each example extracted by `rustdoc` has the form:
            // <from> = `<gen_dir>/<par_from>_<line>_<test-num>/rust_out`
            // where occurrences of '/', '-', and '.' in <par_from> are replaced
            // by '_'. We copy the file in this path to a new path of the form:
            // <to> = `<par_to>/<line>.rs`
            //  We omit <test-num> because all tests have the same number, 0.
            let from = from.unwrap().path();
            let path_line_test = dir.file_name().unwrap().to_str().unwrap();
            let splits: Vec<_> = path_line_test.rsplitn(3, '_').collect();
            let line: usize = splits[1].parse().unwrap();
            let to = par_to.join(format!("{}.rs", line));
            fs::create_dir_all(par_to).unwrap();
            fs::copy(&from, &to).unwrap();
            pairs.push((Example::new(par_from.to_path_buf(), line), to));
        }
    }
    // Delete the temporary directory.
    fs::remove_dir_all(gen_dir).unwrap();
    pairs
}

/// Returns a set of paths to the config files for examples in the Rust books.
fn get_config_paths() -> HashSet<PathBuf> {
    let config_dir: PathBuf = ["src", "tools", "dashboard", "configs"].iter().collect();
    let mut config_paths = HashSet::new();
    for entry in WalkDir::new(config_dir) {
        let entry = entry.unwrap().into_path();
        if entry.is_file() {
            config_paths.insert(entry);
        }
    }
    config_paths
}

/// Prepends the given `props` to the test file in `props.test`.
fn prepend_props(props: &TestProps) {
    let code = fs::read_to_string(&props.path).unwrap();
    let code = format!("{}{}", props, code);
    fs::write(&props.path, code).unwrap();
}

/// Pretty prints the `paths` set.
fn paths_to_string(paths: HashSet<PathBuf>) -> String {
    let mut f = String::new();
    for path in paths {
        f.write_fmt(format_args!("    {:?}\n", path.to_str().unwrap())).unwrap();
    }
    f
}

/// Pre-processes the examples in `map` before running them with `compiletest`.
fn preprocess_examples(map: &HashMap<Example, PathBuf>) {
    let config_dir: PathBuf = ["src", "tools", "dashboard", "configs"].iter().collect();
    let test_dir: PathBuf = ["src", "test"].iter().collect();
    let mut config_paths = get_config_paths();
    // Copy compiler annotations specified in the original markdown code blocks
    // and custom configurations under the `config` directory.
    for (from, to) in map.iter() {
        // Path `to` has the following form:
        // `src/test/ref/<hierarchy>/<line-num>.rs`
        // If it has a custom props file, the path to the props file will have
        // the following form:
        // `src/tools/dashboard/configs/ref/<hierarchy>/<line-num>.props`
        // where <hierarchy> and <line-num> are the same for both paths.
        let mut props_path = config_dir.join(to.strip_prefix(&test_dir).unwrap());
        props_path.set_extension("props");
        let mut props = if props_path.exists() {
            config_paths.remove(&props_path);
            // Parse the properties in the file. The format follows the same
            // conventions for the headers in RMC regressions.
            let mut props = util::parse_test_header(&props_path);
            // `util::parse_test_header` thinks `props_path` is the path to the
            // test. That is not the case, `to` is the actual path to the
            // test/example.
            props.path = to.clone();
            props
        } else {
            TestProps::new(to.clone(), None, Vec::new(), Vec::new())
        };
        let file = File::open(&from.path).unwrap();
        // Skip to the first line of the example code block.
        // Line numbers in files start with 1 but `nth(...)` starts with 0.
        // Subtract 1 to account for the difference.
        let line = BufReader::new(file).lines().nth(from.line - 1).unwrap().unwrap();
        if !line.contains("edition2015") {
            props.rustc_args.push(String::from("--edition"));
            props.rustc_args.push(String::from("2018"));
        }
        // Most examples with `compile_fail` annotation fail because of check
        // errors. This heuristic can be overridden by manually specifying the
        // fail step in the corresponding config file.
        if props.fail_step.is_none() && line.contains("compile_fail") {
            props.fail_step = Some(FailStep::Check);
        }
        // RMC should catch run-time errors.
        if props.fail_step.is_none() && line.contains("should_panic") {
            props.fail_step = Some(FailStep::Verification);
        }
        // Prepend those properties to test/example file.
        prepend_props(&props);
    }
    if !config_paths.is_empty() {
        panic!(
            "Error: The examples corresponding to the following config files \
             were not encountered in the pre-processing step:\n{}This is most \
             likely because the line numbers of the config files are not in \
             sync with the line numbers of the corresponding code blocks in \
             the latest versions of the Rust books. Please update the line \
             numbers of the config files and rerun the program.",
            paths_to_string(config_paths)
        );
    }
    // TODO: Add support for manually adding assertions (see issue #324).
}

/// Runs `compiletest` on the `suite` and logs the results to `log_path`.
fn run_examples(suite: &str, log_path: &Path) {
    // Before executing this program, `cargo` populates the environment with
    // build configs. `x.py` respects those configs, causing a recompilation
    // of `rustc`. This is not a desired behavior, so we remove those configs.
    let filtered_env: HashMap<String, String> = env::vars()
        .filter(|&(ref k, _)| {
            !(k.contains("CARGO") || k.contains("LD_LIBRARY_PATH") || k.contains("RUST"))
        })
        .collect();
    let mut cmd = Command::new([".", "x.py"].iter().collect::<PathBuf>());
    cmd.args([
        "test",
        suite,
        "-i",
        "--stage",
        "1",
        "--test-args",
        "--logfile",
        "--test-args",
        log_path.to_str().unwrap(),
    ]);
    cmd.env_clear().envs(filtered_env);
    cmd.stdout(Stdio::null());
    cmd.spawn().unwrap().wait().unwrap();
}

/// Creates a new [`Tree`] from `path`, and a test `result`.
fn tree_from_path(mut path: Vec<String>, result: bool) -> dashboard::Tree {
    assert!(path.len() > 0, "Error: `path` must contain at least 1 element.");
    let mut tree = dashboard::Tree::new(
        dashboard::Node::new(
            path.pop().unwrap(),
            if result { 1 } else { 0 },
            if result { 0 } else { 1 },
        ),
        vec![],
    );
    for _ in 0..path.len() {
        tree = dashboard::Tree::new(
            dashboard::Node::new(path.pop().unwrap(), tree.data.num_pass, tree.data.num_fail),
            vec![tree],
        );
    }
    tree
}

/// Parses and generates a dashboard from the log output of `compiletest` in
/// `path`.
fn parse_log(path: &Path) -> dashboard::Tree {
    let file = fs::File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut tests = dashboard::Tree::new(dashboard::Node::new(String::from("ref"), 0, 0), vec![]);
    for line in reader.lines() {
        let (ns, l) = parse_log_line(&line.unwrap());
        tests = dashboard::Tree::merge(tests, tree_from_path(ns, l)).unwrap();
    }
    tests
}

/// Parses a line in the log output of `compiletest` and returns a pair containing
/// the path to a test and its result.
fn parse_log_line(line: &str) -> (Vec<String>, bool) {
    // Each line has the format `<result> [rmc] <path>`. Extract <result> and
    // <path>.
    let splits: Vec<_> = line.split(" [rmc] ").map(String::from).collect();
    let l = if splits[0].as_str() == "ok" { true } else { false };
    let mut ns: Vec<_> = splits[1].split(&['/', '.'][..]).map(String::from).collect();
    // Remove unnecessary `.rs` suffix.
    ns.pop();
    (ns, l)
}

/// Display the dashboard in the terminal.
fn display_dashboard(dashboard: dashboard::Tree) {
    println!(
        "# of tests: {}\t✔️ {}\t❌ {}",
        dashboard.data.num_pass + dashboard.data.num_fail,
        dashboard.data.num_pass,
        dashboard.data.num_fail
    );
    println!("{}", dashboard);
}

/// Runs examples using Litani build.
fn litani_run_tests() {
    let output_prefix: PathBuf = ["build", "output"].iter().collect();
    let output_symlink: PathBuf = output_prefix.join("latest");
    let ref_dir: PathBuf = ["src", "test", "ref"].iter().collect();
    util::add_rmc_and_litani_to_path();
    let mut litani = Litani::init("RMC", &output_prefix, &output_symlink);
    // Run all tests under the `src/test/ref` directory.
    for entry in WalkDir::new(ref_dir) {
        let entry = entry.unwrap().into_path();
        if entry.is_file() {
            let test_props = util::parse_test_header(&entry);
            util::add_test_pipeline(&mut litani, &test_props);
        }
    }
    litani.run_build();
}

/// Extracts examples from The Rust Reference, run them through RMC, and
/// displays their results in a terminal dashboard.
pub fn display_reference_dashboard() {
    let summary_path: PathBuf = ["src", "doc", "reference", "src", "SUMMARY.md"].iter().collect();
    let build_dir = &env::var("BUILD_DIR").unwrap();
    let triple = &env::var("TRIPLE").unwrap();
    let log_path: PathBuf = [build_dir, triple, "dashboard", "ref.log"].iter().collect();
    // Parse the chapter/section hierarchy from the table of contents in The
    // Rust Reference.
    let map = parse_hierarchy(&summary_path);
    // Extract examples from The Rust Reference, organize them following the
    // partial hierarchy in map, and return the full hierarchy map.
    let map = extract_examples(map);
    // Pre-process the examples before running them through `compiletest`.
    preprocess_examples(&map);
    // Run `compiletest` on the reference examples.
    run_examples("ref", &log_path);
    // Parse `compiletest` log file.
    let dashboard = parse_log(&log_path);
    // Display the reference dashboard.
    display_dashboard(dashboard);
    // Generate Litani's dashboard.
    litani_run_tests();
}
