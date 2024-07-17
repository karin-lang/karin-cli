mod build;

use std::{env, path};

use build::*;
use karin_js::{Compiler, option::*};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.iter().skip(1).next() {
        Some(cmd) => match cmd.as_str() {
            "build" => run_build(),
            _ => println!("unknown command `{}`", cmd),
        },
        None => println!("please specify any command"),
    }
}

fn run_build() {
    let mut paths = Vec::new();
    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.iter().skip(2);
    while let Some(each_arg) = args_iter.next() {
        let check_path = path::Path::new(each_arg);
        if !check_path.is_dir() {
            panic!("invalid directory: {each_arg}");
        }
        paths.push(each_arg);
    }

    let input = build_input_tree(paths);
    let options = CompilerOptions {
        output_root_name: "output".to_string(),
    };
    let output = Compiler::compile(&input, &options);
    println!("{:?}", output.logs);
    write_output_file(&output.file);
}
