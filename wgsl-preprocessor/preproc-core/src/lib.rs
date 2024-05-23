use std::{collections::HashMap, ffi::OsStr, fs, io::{self, BufRead}, path::{Path, PathBuf}};

pub const PREFIX: &'static str = "//!";

#[derive(Debug, PartialEq, Eq)]
pub struct MapEntry {
    pub filename: Box<OsStr>,
    pub source_start: usize,
    pub dest_start: usize,
    pub length: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SourceMap(pub Vec<MapEntry>);

impl SourceMap {
    pub fn map(&self, file: &OsStr, line: usize) -> Vec<usize> {
        let mut out = vec![];

        for entry in &self.0 {
            if &*entry.filename == file && entry.source_start <= line && line < entry.source_start + entry.length {
                out.push(line - entry.source_start + entry.dest_start);
            }
        }
        
        out
    }

    pub fn unmap<'a>(&'a self, line: usize) -> Option<(&'a OsStr, usize)> {
        let mut delta = (self.0.len()/2) as isize;
        let mut i = delta;

        loop {
            let entry = self.0.get(i as usize)?;

            if line < entry.dest_start {
                delta = (-delta.abs() / 2).min(-1);
            } else if line >= entry.dest_start + entry.length {
                delta = (delta.abs() / 2).max(1);
            } else {
                return Some((&*entry.filename, line - entry.dest_start + entry.source_start));
            }

            i += delta;

            if i < 0 || i >= self.0.len() as isize {
                return None;
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PreprocessError {
    #[error("`path` is not a file")]
    NotAFile(PathBuf),
    #[error("unknown preprocessor command '{command}' ({file}:{line})")]
    UnknownCommand {
        file: PathBuf,
        line: usize,
        command: String,
    },
    #[error("preprocessor prefix without command ({file}:{line})")]
    NoCommand {
        file: PathBuf,
        line: usize,
    },
    #[error("failed to parse preprocess command arguments ({file}:{line})")]
    ArgParse {
        file: PathBuf,
        line: usize,
    },
    #[error("invalid argument '{arg}' ({reason}) ({file}:{line})")]
    InvalidArgument {
        file: PathBuf,
        line: usize,
        arg: String,
        reason: &'static str,
    },
    #[error("unexpected preprocessor command '{command}' ({file}:{line})")]
    UnexpectedCommand {
        file: PathBuf,
        line: usize,
        command: String,
    },
    #[error(transparent)]
    IoError(#[from] io::Error),
}

#[derive(PartialEq, Eq)]
enum CommentMode {
    None,
    SingleLine,
    Multiline,
}

fn apply_consts(line: String, consts: &HashMap<String, String>, comment_mode: &mut CommentMode) -> String {
    let mut new_line: String = String::new();
            
    let line = line + "\n";
    let mut line_chars = line.chars();
    let mut current_token = String::new();

    let mut prev_char = '\0';
    
    while let Some(c) = line_chars.next() {
        // multiline comment end
        if *comment_mode == CommentMode::Multiline && prev_char == '*' && c == '/' {
            *comment_mode = CommentMode::None;
        }

        if *comment_mode == CommentMode::None {
            // single line comment start
            if prev_char == '/' && c == '/' {
                *comment_mode = CommentMode::SingleLine;
            } else if prev_char == '/' && c == '*' { // multiline comment start
                *comment_mode = CommentMode::Multiline;
            }
        }

        if *comment_mode == CommentMode::None && current_token.is_empty() && (c.is_alphabetic() || c == '_') {
            current_token.push(c);
        } else if *comment_mode == CommentMode::None && !current_token.is_empty() && (c.is_alphanumeric() || c == '_') {
            current_token.push(c);
        } else if *comment_mode == CommentMode::None && !current_token.is_empty() {
            if let Some(value) = consts.get(&current_token) {
                new_line += value;
            } else {
                new_line += &current_token;
            }
            new_line.push(c);
            current_token = String::new();
        } else {
            new_line.push(c);
        }
        prev_char = c;
    }
    
    if *comment_mode == CommentMode::SingleLine {
        *comment_mode = CommentMode::None;
    }

    new_line
}

fn _preprocess(root: impl AsRef<Path>, path: impl AsRef<Path>, mut consts: HashMap<String, String>) -> Result<(String, SourceMap), PreprocessError> {
    let mut source_map = SourceMap(Vec::new());
    let mut out = String::new();

    let filepath = root.as_ref().join(path.as_ref());
    let file = io::BufReader::new(fs::File::open(&filepath)?);

    let mut dest_line = 0;

    source_map.0.push(MapEntry {
        filename: filepath.as_os_str().into(),
        source_start: 0,
        dest_start: dest_line,
        length: 0,
    });

    let mut comment_mode = CommentMode::None;
    let mut if_stack = vec![];

    for (line_num, line) in file.lines().enumerate() {
        let line = line?;
        let line = if line.trim_start().starts_with("//!") {
            line.trim().to_owned()
        } else {
            line
        };
        let line = apply_consts(line, &consts, &mut comment_mode);

        if !line.starts_with("//!") || comment_mode != CommentMode::None {
            if if_stack.last().map(|v| *v).unwrap_or(true) {
                out += &line;

                source_map.0.last_mut().expect("unreachable").length += 1;
            
                dest_line += 1;
            }

            continue;
        }

        let mut i = 0;
        for c in line[3..].chars() {
            if !c.is_alphanumeric() {
                break;
            }
            i += 1;
        }

        let command = line[3..3+i].trim();
        let args = line[3+i..].trim();

        match command {
            "include" => {
                if if_stack.last().map(|v| *v).unwrap_or(true) {
                    #[derive(serde::Deserialize)]
                    #[serde(untagged)]
                    enum IncludeArgs {
                        File((String,)),
                        Consts(String, HashMap<String, String>)
                    }

                    let args = ron::from_str::<IncludeArgs>(args).map_err(|_| PreprocessError::ArgParse { file: filepath.clone(), line: line_num })?;
                    let (path, mut arg_consts) = match args {
                        IncludeArgs::File((path,)) => (path, HashMap::new()),
                        IncludeArgs::Consts(path, arg_consts) => (path, arg_consts),
                    };
                    let path = root.as_ref().join(path);
                    for (key, value) in &consts {
                        if !arg_consts.contains_key(key) {
                            arg_consts.insert(key.clone(), value.clone());
                        }
                    }

                    let root = path.parent().unwrap_or(Path::new(""));
                    let file = path.file_name().ok_or(PreprocessError::ArgParse { file: filepath.clone(), line: line_num })?;
                    
                    let (include, mut include_map) = _preprocess(root, file, arg_consts)?;
                    out += &include;

                    for m in &mut include_map.0 {
                        m.dest_start += dest_line;
                    }

                    source_map.0.extend(include_map.0.into_iter());

                    dest_line += include.chars().filter(|c| *c == '\n').count();
                }
            }
            "define" => {
                if if_stack.last().map(|v| *v).unwrap_or(true) {
                    let (name, value) = ron::from_str::<(String, String)>(args).map_err(|_| PreprocessError::ArgParse { file: filepath.clone(), line: line_num })?;
                    
                    if name.len() == 0
                        || {let c = name.chars().next().expect("unreachable"); !c.is_alphabetic() && c != '_'}
                        || name.chars().filter(|c| !c.is_alphanumeric() && *c != '_').count() > 0 {
                        return Err(PreprocessError::InvalidArgument {
                            file: filepath.clone(),
                            line: line_num,
                            arg: name,
                            reason: "macro variable should only contain alphanumeric characters and underscores",
                        });
                    }
                    if value.contains('\n') {
                        return Err(PreprocessError::InvalidArgument {
                            file: filepath.clone(),
                            line: line_num,
                            arg: name,
                            reason: "macro variable value should not contain new lines",
                        });
                    }
                    
                    consts.insert(name, value);
                }
            }
            "ifdef" => {
                if if_stack.last().map(|v| *v).unwrap_or(true) {
                    let (name,) = ron::from_str::<(String,)>(args).map_err(|_| PreprocessError::ArgParse { file: filepath.clone(), line: line_num })?;
                    if_stack.push(consts.contains_key(&name));
                }
            }
            "ifndef" => {
                if if_stack.last().map(|v| *v).unwrap_or(true) {
                    let (name,) = ron::from_str::<(String,)>(args).map_err(|_| PreprocessError::ArgParse { file: filepath.clone(), line: line_num })?;
                    if_stack.push(!consts.contains_key(&name));
                }
            }
            "ifeq" => {
                if if_stack.last().map(|v| *v).unwrap_or(true) {
                    let (name, value) = ron::from_str::<(String, String)>(args).map_err(|_| PreprocessError::ArgParse { file: filepath.clone(), line: line_num })?;
                    if_stack.push(
                        consts.get(&name).map(|v| *v == value)
                        .ok_or(PreprocessError::InvalidArgument {
                            file: filepath.clone(),
                            line: line_num,
                            arg: name,
                            reason: "undefined macro variable",
                        })?
                    );
                }
            }
            "ifneq" => {
                if if_stack.last().map(|v| *v).unwrap_or(true) {
                    let (name, value) = ron::from_str::<(String, String)>(args).map_err(|_| PreprocessError::ArgParse { file: filepath.clone(), line: line_num })?;
                    if_stack.push(
                        consts.get(&name).map(|v| *v != value)
                        .ok_or(PreprocessError::InvalidArgument {
                            file: filepath.clone(),
                            line: line_num,
                            arg: name,
                            reason: "undefined macro variable",
                        })?
                    );
                }
            }
            "else" => {
                if args != "" {
                    return Err(PreprocessError::InvalidArgument {
                        file: filepath.clone(),
                        line: line_num,
                        arg: args.into(),
                        reason: "unexpected argument",
                    });
                }
                if if_stack.len() == 0 {
                    return Err(PreprocessError::UnexpectedCommand {
                        file: filepath.clone(),
                        line: line_num,
                        command: command.into(),
                    });
                }

                let idx = if_stack.len()-1;
                let last = if_stack[idx];
                if_stack[idx] = !last;
            }
            "endif" => {
                if args != "" {
                    return Err(PreprocessError::InvalidArgument {
                        file: filepath.clone(),
                        line: line_num,
                        arg: args.into(),
                        reason: "unexpected argument",
                    });
                }
                if if_stack.len() == 0 {
                    return Err(PreprocessError::UnexpectedCommand {
                        file: filepath.clone(),
                        line: line_num,
                        command: command.into(),
                    });
                }
                
                if_stack.pop().expect("unreachable");
            }
            "" => return Err(PreprocessError::NoCommand {
                file: filepath,
                line: line_num,
            }),
            _ => return Err(PreprocessError::UnknownCommand {
                file: filepath,
                line: line_num,
                command: command.into(),
            }),
        }

        source_map.0.push(MapEntry {
            filename: filepath.as_os_str().into(),
            source_start: line_num + 1,
            dest_start: dest_line,
            length: 0,
        });
    }

    source_map.0 = source_map.0.into_iter().filter(|e| e.length > 0).collect();

    Ok((out, source_map))
}

/// Returns preprocessed wgsl file.
pub fn preprocess(path: impl AsRef<Path>) -> Result<(String, SourceMap), PreprocessError> {

    let path = path.as_ref();

    let root = path.parent().unwrap_or(Path::new(""));
    let file = path.file_name().ok_or(io::Error::new(io::ErrorKind::Unsupported, "not a file"))?;

    _preprocess(root, file, HashMap::new())
}

/// Returns preprocessed wgsl file, given some macro constants.
pub fn preprocess_with(path: impl AsRef<Path>, consts: HashMap<String, String>) -> Result<(String, SourceMap), PreprocessError> {

    let path = path.as_ref();

    let root = path.parent().unwrap_or(Path::new(""));
    let file = path.file_name().ok_or(io::Error::new(io::ErrorKind::Unsupported, "not a file"))?;

    _preprocess(root, file, consts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_source_map() {
        let test_path = OsStr::new("../tests/test.wgsl");
        let incl_path = OsStr::new("../tests/include/include.wgsl");
        let sub_incl_path = OsStr::new("../tests/include/sub_include.wgsl");
        
        let (contents, map) = preprocess(test_path).unwrap();

        assert_eq!(contents, "i0\ni1\ni2\ns0\ns1\ni3\ni4\ni5\ni6\n0\n1\n2\n3\ni0\ni1\ni2\ns0\ns1\ni3\ni4\ni5\ni6\n4\n5\n6\n7\ni0\ni1\ni2\ns0\ns1\ni3\ni4\ni5\ni6\n8\n9\n10\ni0\ni1\ni2\ns0\ns1\ni3\ni4\ni5\ni6\n");
        assert_eq!(map, SourceMap(
            vec![
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 0,
                    dest_start: 0,
                    length: 3,
                },
                MapEntry {
                    filename: sub_incl_path.into(),
                    source_start: 0,
                    dest_start: 3,
                    length: 2,
                },
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 4,
                    dest_start: 5,
                    length: 4,
                },
                MapEntry {
                    filename: test_path.into(),
                    source_start: 1,
                    dest_start: 9,
                    length: 4,
                },
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 0,
                    dest_start: 13,
                    length: 3,
                },
                MapEntry {
                    filename: sub_incl_path.into(),
                    source_start: 0,
                    dest_start: 16,
                    length: 2,
                },
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 4,
                    dest_start: 18,
                    length: 4,
                },
                MapEntry {
                    filename: test_path.into(),
                    source_start: 6,
                    dest_start: 22,
                    length: 4,
                },
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 0,
                    dest_start: 26,
                    length: 3,
                },
                MapEntry {
                    filename: sub_incl_path.into(),
                    source_start: 0,
                    dest_start: 29,
                    length: 2,
                },
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 4,
                    dest_start: 31,
                    length: 4,
                },
                MapEntry {
                    filename: test_path.into(),
                    source_start: 11,
                    dest_start: 35,
                    length: 3,
                },
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 0,
                    dest_start: 38,
                    length: 3,
                },
                MapEntry {
                    filename: sub_incl_path.into(),
                    source_start: 0,
                    dest_start: 41,
                    length: 2,
                },
                MapEntry {
                    filename: incl_path.into(),
                    source_start: 4,
                    dest_start: 43,
                    length: 4,
                },
            ],
        ));

        // test map
        assert_eq!(map.map(test_path, 0), []);
        assert_eq!(map.map(test_path, 1), [9]);
        assert_eq!(map.map(test_path, 2), [10]);
        assert_eq!(map.map(test_path, 3), [11]);
        assert_eq!(map.map(test_path, 4), [12]);
        assert_eq!(map.map(test_path, 5), []);
        assert_eq!(map.map(test_path, 6), [22]);
        assert_eq!(map.map(test_path, 7), [23]);
        assert_eq!(map.map(test_path, 8), [24]);
        assert_eq!(map.map(test_path, 9), [25]);
        assert_eq!(map.map(test_path, 10), []);
        assert_eq!(map.map(test_path, 11), [35]);
        assert_eq!(map.map(test_path, 12), [36]);
        assert_eq!(map.map(test_path, 13), [37]);
        assert_eq!(map.map(test_path, 14), []);
        assert_eq!(map.map(test_path, 15), []);

        assert_eq!(map.map(incl_path, 0), [0, 13, 26, 38]);
        assert_eq!(map.map(incl_path, 1), [1, 14, 27, 39]);
        assert_eq!(map.map(incl_path, 2), [2, 15, 28, 40]);
        assert_eq!(map.map(incl_path, 3), []);
        assert_eq!(map.map(incl_path, 4), [5, 18, 31, 43]);
        assert_eq!(map.map(incl_path, 5), [6, 19, 32, 44]);
        assert_eq!(map.map(incl_path, 6), [7, 20, 33, 45]);
        assert_eq!(map.map(incl_path, 7), [8, 21, 34, 46]);
        assert_eq!(map.map(incl_path, 8), []);

        assert_eq!(map.map(sub_incl_path, 0), [3, 16, 29, 41]);
        assert_eq!(map.map(sub_incl_path, 1), [4, 17, 30, 42]);
        assert_eq!(map.map(sub_incl_path, 2), []);

        // test unmap
        // test.wgsl
        {
            // include/include.wgsl
            {
                assert_eq!(map.unmap(0), Some((incl_path.into(), 0)));
                assert_eq!(map.unmap(1), Some((incl_path.into(), 1)));
                assert_eq!(map.unmap(2), Some((incl_path.into(), 2)));
                // include/sub_include.wgsl
                {
                    assert_eq!(map.unmap(3), Some((sub_incl_path.into(), 0)));
                    assert_eq!(map.unmap(4), Some((sub_incl_path.into(), 1)));
                }
                assert_eq!(map.unmap(5), Some((incl_path.into(), 4)));
                assert_eq!(map.unmap(6), Some((incl_path.into(), 5)));
                assert_eq!(map.unmap(7), Some((incl_path.into(), 6)));
                assert_eq!(map.unmap(8), Some((incl_path.into(), 7)));
            }
            assert_eq!(map.unmap(9), Some((test_path.into(), 1)));
            assert_eq!(map.unmap(10), Some((test_path.into(), 2)));
            assert_eq!(map.unmap(11), Some((test_path.into(), 3)));
            assert_eq!(map.unmap(12), Some((test_path.into(), 4)));
            // include/include.wgsl
            {
                assert_eq!(map.unmap(13), Some((incl_path.into(), 0)));
                assert_eq!(map.unmap(14), Some((incl_path.into(), 1)));
                assert_eq!(map.unmap(15), Some((incl_path.into(), 2)));
                // include/sub_include.wgsl
                {
                    assert_eq!(map.unmap(16), Some((sub_incl_path.into(), 0)));
                    assert_eq!(map.unmap(17), Some((sub_incl_path.into(), 1)));
                }
                assert_eq!(map.unmap(18), Some((incl_path.into(), 4)));
                assert_eq!(map.unmap(19), Some((incl_path.into(), 5)));
                assert_eq!(map.unmap(20), Some((incl_path.into(), 6)));
                assert_eq!(map.unmap(21), Some((incl_path.into(), 7)));
            }
            assert_eq!(map.unmap(22), Some((test_path.into(), 6)));
            assert_eq!(map.unmap(23), Some((test_path.into(), 7)));
            assert_eq!(map.unmap(24), Some((test_path.into(), 8)));
            assert_eq!(map.unmap(25), Some((test_path.into(), 9)));
            // include/include.wgsl
            {
                assert_eq!(map.unmap(26), Some((incl_path.into(), 0)));
                assert_eq!(map.unmap(27), Some((incl_path.into(), 1)));
                assert_eq!(map.unmap(28), Some((incl_path.into(), 2)));
                // include/sub_include.wgsl
                {
                    assert_eq!(map.unmap(29), Some((sub_incl_path.into(), 0)));
                    assert_eq!(map.unmap(30), Some((sub_incl_path.into(), 1)));
                }
                assert_eq!(map.unmap(31), Some((incl_path.into(), 4)));
                assert_eq!(map.unmap(32), Some((incl_path.into(), 5)));
                assert_eq!(map.unmap(33), Some((incl_path.into(), 6)));
                assert_eq!(map.unmap(34), Some((incl_path.into(), 7)));
            }
            assert_eq!(map.unmap(35), Some((test_path.into(), 11)));
            assert_eq!(map.unmap(36), Some((test_path.into(), 12)));
            assert_eq!(map.unmap(37), Some((test_path.into(), 13)));
            // include/include.wgsl
            {
                assert_eq!(map.unmap(38), Some((incl_path.into(), 0)));
                assert_eq!(map.unmap(39), Some((incl_path.into(), 1)));
                assert_eq!(map.unmap(40), Some((incl_path.into(), 2)));
                // include/sub_include.wgsl
                {
                    assert_eq!(map.unmap(41), Some((sub_incl_path.into(), 0)));
                    assert_eq!(map.unmap(42), Some((sub_incl_path.into(), 1)));
                }
                assert_eq!(map.unmap(43), Some((incl_path.into(), 4)));
                assert_eq!(map.unmap(44), Some((incl_path.into(), 5)));
                assert_eq!(map.unmap(45), Some((incl_path.into(), 6)));
                assert_eq!(map.unmap(46), Some((incl_path.into(), 7)));
            }
        }
    }

    #[test]
    fn define() {
        let (contents, _) = preprocess("../tests/define.wgsl").unwrap();
        assert_eq!(contents, "B // ABC should be B\n/*\nABC, should be B\n*/\nB C C // ABC B C, should be B C C\n/* ABC B C, should be B C C\n*/\nABC C C // ABC B C, should be A C C\n/*\nABC B C, should be A C C*/\n");
    }

    #[test]
    fn comments() {
        let (contents, _) = preprocess("../tests/comments.wgsl").unwrap();
        assert_eq!(contents, "i0\ni1\ni2\ns0\ns1\ni3\ni4\ni5\ni6\n// !include(\"include/include.wgsl\")\n///!include(\"include/include.wgsl\")\n/* //!include(\"include/include.wgsl\")\n//!include(\"include/include.wgsl\")\n");
    }

    #[test]
    fn test_if() {
        let (contents, _) = preprocess("../tests/if.wgsl").unwrap();
        assert_eq!(contents, "    TEST_A // include\n    TEST_D // include\n    TEST_E // include\n    TEST_H // include\n        TEST_I // include\n        TEST_L // include\n        TEST_M // include\n        TEST_O // include\nd0\nd\nd1\n");
    }

    #[test]
    fn source_map() {
        let source_file = OsStr::new("../tests/source_map.wgsl");
        let sub_incl_file = OsStr::new("../tests/include/sub_include.wgsl");
        let def_incl_file = OsStr::new("../tests/include/def_include.wgsl");
        let (contents, map) = preprocess(source_file).unwrap();
        
        assert_eq!(contents, "s0\ns1\n    TEST_H // include\n        TEST_O // include\ns0\ns1\nd0\nd\nd1\n");
        assert_eq!(map, SourceMap(
            vec![
                MapEntry {
                    filename: sub_incl_file.into(),
                    source_start: 0,
                    dest_start: 0,
                    length: 2,
                },
                MapEntry {
                    filename: source_file.into(),
                    source_start: 6,
                    dest_start: 2,
                    length: 1,
                },
                MapEntry {
                    filename: source_file.into(),
                    source_start: 8,
                    dest_start: 3,
                    length: 1,
                },
                MapEntry {
                    filename: sub_incl_file.into(),
                    source_start: 0,
                    dest_start: 4,
                    length: 2,
                },
                MapEntry {
                    filename: def_incl_file.into(),
                    source_start: 0,
                    dest_start: 6,
                    length: 3,
                },
            ],
        ));

        // test map
        assert_eq!(map.map(source_file, 0), []);
        assert_eq!(map.map(source_file, 1), []);
        assert_eq!(map.map(source_file, 2), []);
        assert_eq!(map.map(source_file, 3), []);
        assert_eq!(map.map(source_file, 4), []);
        assert_eq!(map.map(source_file, 5), []);
        assert_eq!(map.map(source_file, 6), [2]);
        assert_eq!(map.map(source_file, 7), []);
        assert_eq!(map.map(source_file, 8), [3]);
        assert_eq!(map.map(source_file, 9), []);
        assert_eq!(map.map(source_file, 10), []);
        assert_eq!(map.map(source_file, 11), []);
        assert_eq!(map.map(source_file, 12), []);
        assert_eq!(map.map(source_file, 13), []);
        assert_eq!(map.map(source_file, 14), []);
        assert_eq!(map.map(source_file, 15), []);
        assert_eq!(map.map(source_file, 16), []);

        assert_eq!(map.map(sub_incl_file, 0), [0, 4]);
        assert_eq!(map.map(sub_incl_file, 1), [1, 5]);

        assert_eq!(map.map(def_incl_file, 0), [6]);
        assert_eq!(map.map(def_incl_file, 1), [7]);
        assert_eq!(map.map(def_incl_file, 2), [8]);
        
        println!("{contents}");

        for i in 0..9 {
            println!("{i} {:?}", map.unmap(i));
        }

        // test unmap
        // include/sub_include.wgsl
        {
            assert_eq!(map.unmap(0), Some((sub_incl_file, 0)));
            assert_eq!(map.unmap(1), Some((sub_incl_file, 1)));
        }
        // source_map.wgsl
        assert_eq!(map.unmap(2), Some((source_file, 6)));
        assert_eq!(map.unmap(3), Some((source_file, 8)));
        // include/sub_include.wgsl
        {
            assert_eq!(map.unmap(4), Some((sub_incl_file, 0)));
            assert_eq!(map.unmap(5), Some((sub_incl_file, 1)));
        }
        // include/def_include.wgsl
        {
            assert_eq!(map.unmap(6), Some((def_incl_file, 0)));
            assert_eq!(map.unmap(7), Some((def_incl_file, 1)));
            assert_eq!(map.unmap(8), Some((def_incl_file, 2)));
        }
    }

    #[test]
    fn args() {
        let mut map = HashMap::new();
        map.insert(String::from("DEF"), String::from("abc"));
        
        let (contents, _) = preprocess_with("../tests/args.wgsl", map).unwrap();
        
        assert_eq!(contents, "d0\nabc\nd1\nd0\ndef\nd1\n");
    }
}
