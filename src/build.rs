use std::{fs, path};
use std::io::{Read, Write};

use karin_js::output;
use karinc::{parser::ast, hir::id::*, input::*};

#[derive(Clone, Debug, PartialEq)]
pub struct Dir {
    pub name: String,
    pub files: Vec<File>,
    pub subdirs: Vec<Dir>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct File {
    pub path: Box<path::Path>,
    pub name: String,
}

pub fn build_input_tree(paths: Vec<&String>) -> InputTree {
    let mut hakos = Vec::new();
    let mut hako_id_counter = 0;
    // 一番最初に指定された hako を main hako として扱う
    let mut main_hako_name = None;
    for each_path in paths {
        let dir = get_dir(&each_path);
        let id = {
            let new_id = hako_id_counter;
            hako_id_counter += 1;
            new_id
        };
        let name = dir.name.clone();
        if let None = main_hako_name {
            main_hako_name = Some(name.clone());
        }
        let new_hako = conv_dir_to_hako(HakoId::new(id), name, &dir);
        hakos.push(new_hako);
    }

    InputTree { hakos, main_hako_name: main_hako_name.expect("expected at least one module") }
}

// hako のルートディレクトリを InputHako に変換する
pub fn conv_dir_to_hako(id: HakoId, name: String, hako_dir: &Dir) -> InputHako {
    let mut mod_id_counter = 0;
    let parent_mod_path = vec![hako_dir.name.clone()];
    let mods = conv_dir_to_mods(0 /* fix */, &mut mod_id_counter, &parent_mod_path, hako_dir);
    InputHako { id, name, mods }
}

// ディレクトリ内のファイルリストをモジュールリストに変換する (サブモジュール含む)
pub fn conv_dir_to_mods(hako_id: usize, mod_id_counter: &mut usize, parent_mod_path: &Vec<String>, mod_dir: &Dir) -> Vec<InputMod> {
    let mut mods = Vec::new();
    for mod_file in &mod_dir.files {
        let new_mod = conv_file_to_mod(hako_id, mod_id_counter, &parent_mod_path, mod_dir, mod_file);
        mods.push(new_mod);
    }
    mods
}

// ファイルをモジュールに変換する (サブモジュール含む)
pub fn conv_file_to_mod(hako_id: usize, mod_id_counter: &mut usize, parent_mod_path: &Vec<String>, parent_dir: &Dir, mod_file: &File) -> InputMod {
    let mod_id = {
        let new_id = *mod_id_counter;
        *mod_id_counter += 1;
        new_id
    };
    let mod_path = {
        let mut mod_path = parent_mod_path.clone();
        mod_path.push(mod_file.name.clone());
        mod_path
    };
    let source = read_file_content(&mod_file.path);
    let submods = {
        match get_submod_dirs(&parent_dir, &mod_file.name) {
            Some(submod_dir) => conv_dir_to_mods(hako_id, mod_id_counter, &mod_path, submod_dir),
            None => Vec::new(),
        }
    };
    InputMod {
        id: ModId::new(hako_id, mod_id),
        path: ast::Path::from(mod_path),
        source,
        submods,
    }
}

// 同一ディレクトリ内にサブファイル名と一致するサブディレクトリ名があればサブディレクトリを返す
// 返却したサブディレクトリはサブモジュールの取得に利用される
pub fn get_submod_dirs<'a>(parent_dir: &'a Dir, filename: &'a str) -> Option<&'a Dir> {
    for subdir in &parent_dir.subdirs {
        // ファイル名と一致するサブディレクトリ名が存在する場合はサブモジュールのディレクトリとして認識する
        if subdir.name == filename {
            return Some(subdir);
        }
    }
    None
}

pub fn get_dir(dirpath: &str) -> Dir {
    let dirpath = path::Path::new(dirpath).to_path_buf().canonicalize().unwrap();
    let dirname = dirpath.file_stem().unwrap().to_str().unwrap().to_string();
    let read_dir = fs::read_dir(dirpath).unwrap();
    let mut files = Vec::new();
    let mut subdirs = Vec::new();
    for entry in read_dir {
        let path = entry.unwrap().path();
        let path_str = path.to_str().unwrap().to_string();
        if path.is_dir() {
            let new_subdir = get_dir(&path_str);
            subdirs.push(new_subdir);
        } else {
            if let Some(fileext) = path.extension() {
                if fileext == "kr" {
                    let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
                    let new_file = File { path: path.into_boxed_path(), name: filename };
                    files.push(new_file);
                }
            }
        }
    }
    Dir { name: dirname, files, subdirs }
}

pub fn read_file_content(filepath: &path::Path) -> String {
    let mut file = fs::File::open(filepath).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content
}

pub fn write_output_file(file: &output::OutputFile) {
    match &file.source {
        Some(code) if !code.source.is_empty() => {
            let outpath = format!("./{}.{}", file.name, file.ext);
            write_file_content(&path::Path::new(&outpath), &code.source);
            println!("successfully compiled!");
        },
        _ => println!("there is no generated code"),
    }
}

pub fn write_file_content(filepath: &path::Path, content: &str) {
    let mut file = fs::File::create(filepath).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}
