// use std::env;
use std::fs::File;
use std::io::{self, Read, BufReader};
use std::collections::HashMap;
use std::io::BufRead;
use sotn_str::{decode_menu_string, encode_menu_string, Platform};
// use std::io::Seek;

#[allow(dead_code)]
fn utf8_to_byte_literals_escaped(input: &str) -> String {
    let out = utf8_to_byte_literals(input);
    let out2 = format!("\"{}\"", out);
    out2
}

fn utf8_to_byte_literals(input_str: &str) -> String {
    let value = input_str
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(input_str);
    let bytes = encode_menu_string(value, Platform::Psx)
        .unwrap_or_else(|error| panic!("cannot encode _S({input_str:?}): {error}"));
    let out = bytes.iter()
        .map(|&val| format!("\\x{:02X}", val))
        .collect::<String>();
    format!("\"{}\"", out)
}

fn utf8_to_byte_literals_psp(input_str: &str) -> String {
    let value = input_str
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(input_str);
    let bytes = encode_menu_string(value, Platform::Psp)
        .unwrap_or_else(|error| panic!("cannot encode PSP _S({input_str:?}): {error}"));
    let out = bytes.iter()
        .map(|&val| format!("\\x{:02X}", val))
        .collect::<String>();
    format!("\"{}\"", out)
}


// fn parse(filename: &str, str_offset: &str) -> io::Result<()> {
//     let offset = usize::from_str_radix(str_offset, 16).unwrap();
//     let file = File::open(filename)?;
//     let mut reader = BufReader::new(file);

//     reader.seek(io::SeekFrom::Start(offset as u64))?;

//     let mut r = String::new();
//     loop {
//         let mut buffer = [0];
//         match reader.read_exact(&mut buffer) {
//             Ok(_) => {
//                 let ch = buffer[0];
//                 if ch == 0xFF {
//                     break;
//                 }
//                 r.push((ch + 0x20) as char);
//             }
//             Err(_) => break,
//         }
//     }

//     println!(r#"_S("{}")"#, r);
//     Ok(())
// }

// slower regex version

// lazy_static! {
//     static ref RE_S: Regex = Regex::new(r"_S\(([^()]*|(?:[^()]*\([^()]*\)[^()]*)*)\)").unwrap();
//     static ref RE_S2: Regex = Regex::new(r"_S2\((.*?)\)").unwrap();
//     static ref RE_S2_HD: Regex = Regex::new(r"_S2_HD\((.*?)\)").unwrap();
// }

// fn process_macro_with_transform(line: &str, re: &Regex, transform: impl Fn(&str) -> Vec<u8>) -> String {
//     re.replace_all(line, |match_: &regex::Captures| {
//         let s = match_.get(1).map_or("", |m| m.as_str());
//         // println!("{}", s);
//         let s = s.replace(r#"\\"#, "\"");
//         let out = transform(&s);
//         let escaped = out.iter()
//             .map(|&c| format!("\\x{:02X}", c))
//             .collect::<String>();
//         format!("\"{}\"", escaped)
//     }).to_string()
// }

// fn process_s_macro(line: &str) -> String {
//     process_macro_with_transform(line, &RE_S, utf8_to_byte_literals)
// }

// fn process_s2_macro(line: &str) -> String {
//     process_macro_with_transform(line,&RE_S2, alt_utf8_to_byte_literals)
// }

// fn process_s2_hd_macro(line: &str) -> String {
//     process_macro_with_transform(line,&RE_S2_HD, alt_hd_utf8_to_byte_literals)
// }
fn process_macro(line: &str, macro_name: &str, transform: impl Fn(&str) -> String) -> String {
    let mut result = String::new();
    let mut last_end = 0;
    while let Some((start, end, content)) = find_macro_content(&line, macro_name, last_end) {
        if start > end {
            panic!("Invalid range: start index {} cannot be greater than end index {}", start, end);
        }
 
        // Append the portion before the macro
        result.push_str(&line[last_end..start-macro_name.len()-1]);

        let processed_content = transform(&content.replace(r#"\\"#, "\""));

        // Append the transformed macro content
        result.push_str(&processed_content);

        // Update last_end to continue from the end of the replaced macro content
        last_end = end + 1;
    }

    // Append any remaining part of the line after the last macro
    result.push_str(&line[last_end..]);
    result
}

fn find_macro_content(line: &str, macro_name: &str, last_end: usize) -> Option<(usize, usize, String)> {
    let start_pattern = format!("{}(", macro_name);
    if let Some(start) = line[last_end..].find(&start_pattern) {
        let start_idx = start + last_end + start_pattern.len();
        let mut balance = 1;
        let mut end_idx = start_idx;

        while end_idx < line.len() {
            match line[end_idx..].chars().next() {
                Some('(') => balance += 1,
                Some(')') => {
                    balance -= 1;
                    if balance == 0 {
                        return Some((start_idx, end_idx, line[start_idx..end_idx].to_string()));
                    }
                }
                _ => {}
            }
            end_idx += line[end_idx..].chars().next().unwrap_or_default().len_utf8();
        }
    }
    None
}

fn process_s_macro(line: &str, psp: bool) -> String {
    if psp
    {
        process_macro(line, "_S", utf8_to_byte_literals_psp)
    }
    else {
        process_macro(line, "_S", utf8_to_byte_literals)

    }
}

fn process_s2_macro(line: &str) -> String {
    process_macro(line, "_S2", alt_utf8_to_byte_literals)
}

fn process_s2_hd_macro(line: &str) -> String {
    process_macro(line, "_S2_HD", alt_hd_utf8_to_byte_literals)
}

fn process_se_macro(line: &str) -> String {
    process_macro(line, "_SE", s3_utf8_to_byte_literals)
}



fn do_sub(line: &str, psp: bool) -> String {
    let mut processed = process_s_macro(line, psp);
    processed = process_s2_macro(&processed);
    processed = process_s2_hd_macro(&processed);
    processed = process_se_macro(&processed);
    processed
}

fn process(filename: Option<String>, psp: bool) -> io::Result<()> {
    let reader: Box<dyn Read> = match filename {
        Some(file) => Box::new(BufReader::new(File::open(file)?)),
        None => Box::new(io::stdin()),
    };

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut output = String::new();

    while reader.read_line(&mut line)? > 0 {
        output.push_str(&do_sub(&line, psp));
        line.clear();
    }
    
    print!("{}", output);

    Ok(())
}

use lazy_static::lazy_static;

lazy_static! {
    static ref ALT_UTF8_TO_INDEX: HashMap<char, usize> = {
        let values = "ＡＴＤＥＦ".chars().collect::<Vec<char>>();
        values.into_iter().enumerate().map(|(index, value)| (value, index)).collect::<HashMap<char, usize>>()
    };

    static ref ALT_HD_UTF8_TO_INDEX: HashMap<char, usize> = {
        let values = [
            "装備技システム短剣必殺使攻撃力防",
            "御魔導器拳こ一覧棒両手食物爆弾盾",
            "投射薬ん右左武兜鎧マントその他い",
        ]
        .concat();
        values.chars().enumerate().map(|(index, value)| (value, index)).collect::<HashMap<char, usize>>()
    };
}

fn fix_se(chr: char) -> char {
    match chr {
        'ù' => 'ｦ', // 0xA6
        'û' => 'ｧ', // 0xA7
        'ü' => 'ｨ', // 0xA8
        'Œ' => 'ｩ', // 0xA9
        'œ' => 'ｪ', // 0xAA
        '¡' => 'ｲ', // 0xB2
        '¿' => 'ｳ', // 0xB3
        'À' => 'ｴ', // 0xB4
        'Ä' => 'ｷ', // 0xB7
        'Ç' => 'ｸ', // 0xB8
        'È' => 'ｹ', // 0xB9
        'Ö' => 'ﾅ', // 0xC5
        'ß' => 'ﾊ', // 0xCA
        'à' => 'ﾋ', // 0xCB
        'á' => 'ﾌ', // 0xCC
        'â' => 'ﾍ', // 0xCD
        'ä' => 'ﾎ', // 0xCE
        'ç' => 'ﾏ', // 0xCF
        'è' => 'ﾐ', // 0xD0
        'é' => 'ﾑ', // 0xD1
        'ê' => 'ﾒ', // 0xD2
        'ì' => 'ﾔ', // 0xD4
        'í' => 'ﾕ', // 0xD5
        'î' => 'ﾖ', // 0xD6
        'ñ' => 'ﾘ', // 0xD8
        'ò' => 'ﾙ', // 0xD9
        'ó' => 'ﾚ', // 0xDA
        'ô' => 'ﾛ', // 0xDB
        'ö' => 'ﾜ', // 0xDC
        'ú' => 'ﾝ', // 0xDD
        _ => chr,
    }
}


fn alt_hd_utf8_to_index(c: &char) -> Option<usize> {
    return ALT_HD_UTF8_TO_INDEX.get(c).copied();
}

fn alt_utf8_to_index(c: &char) -> Option<usize> {
    return ALT_UTF8_TO_INDEX.get(c).copied();
}

fn alt_utf8_to_byte_literals(input_str: &str) -> String {
    let mut bytes = Vec::new();
    for char in input_str.chars() {
        if let Some(index) =alt_utf8_to_index(&char) {
            bytes.push(index as u8);
        }
    }
    bytes.push(0xFF);
    let output = bytes.iter()
    .map(|&c| format!("\\x{:02X}", c))
    .collect::<String>();
    let out2 = format!("\"{}\"", output);
    out2
}

fn alt_hd_utf8_to_byte_literals(input_str: &str) -> String {
    let mut bytes = Vec::new();
    for char in input_str.chars() {
        if let Some(index) = alt_hd_utf8_to_index(&char) {
            bytes.push(index as u8);
        }
    }
    
    bytes.push(0xFF);
    let output = bytes.iter()
    .map(|&c| format!("\\x{:02X}", c))
    .collect::<String>();
    let out2 = format!("\"{}\"", output);
    out2
}

fn s3_utf8_to_byte_literals(input_str: &str) -> String {
    let mut out = String::new();
    for char in input_str.chars() {
        out.push(fix_se(char));
    }
    out
}

use clap::{Arg};

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     let command = if args.len() > 1 { &args[1] } else { "" };

//     match command {
//         "parse" => {
//             if args.len() < 4 {
//                 eprintln!("Usage: parse <filename> <offset>");
//                 return;
//             }
//             let filename = &args[2];
//             let offset = &args[3];
//             if let Err(e) = parse(filename, offset) {
//                 eprintln!("Error: {}", e);
//             }
//         }
//         "process" => {
//             let filename = if args.len() > 2 { Some(args[2].clone()) } else { None };
//             if let Err(e) = process(filename) {
//                 eprintln!("Error: {}", e);
//             }
//         }
//         _ => {
//             eprintln!("Usage: <parse|process>");
//         }
//     }
// }
use clap::{ArgAction, Command};

/// Parses a GCC/Clang-style compile command line to find:
/// - index of the source file argument
/// - presence for -o <output> arg
/// - presence for -c arg
fn parse_command(args: &[String]) -> (Option<usize>, bool, bool) {
    let mut input_index = None;
    let mut output_present = false;
    let mut is_compile = false;

    let mut i = 0;
    while i < args.len() {
        let a = &args[i];
        if a == "-c" {
            is_compile = true;
        } else if a == "-o" {
            output_present = true;
            i += 1; // skip the output path
        } else if a.starts_with("-o") && a.len() > 2 {
            output_present = true;
        } else if a.ends_with(".c") {
            input_index = Some(i);
        }
        i += 1;
    }

    (input_index, output_present, is_compile)
}

/// Builds the argument list for the preprocessing pass:
/// strips `-c`/`-o ...`, appends `-E -DSOTN_STR`.
fn preprocess_args(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip_next = false;
    for a in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a == "-c" {
            continue;
        }
        if a == "-o" {
            skip_next = true;
            continue;
        }
        if a.starts_with("-o") && a.len() > 2 {
            continue;
        }
        out.push(a.clone());
    }
    out.push("-E".to_string());
    out.push("-DSOTN_STR".to_string());
    out
}

fn cc_fail(msg: &str) -> ! {
    eprintln!("sotn_str cc: {}", msg);
    std::process::exit(1);
}

// injects a temporary, `_S`-transformed file to the compiler
fn cc(cc_args: &[String]) -> io::Result<i32> {
    use std::io::Write;
    use std::process::{Command as PCommand, Stdio};

    if cc_args.is_empty() {
        cc_fail("usage: sotn_str cc <compiler> <args...>");
    }

    let compiler = &cc_args[0];
    let args = &cc_args[1..];

    let (input_index, output_present, is_compile) = parse_command(args);

    if !is_compile || input_index.is_none() || !output_present {
        let status = PCommand::new(compiler).args(args).status()?;
        return Ok(status.code().unwrap_or(1));
    }
    let input_index = input_index.unwrap();

    let pp_args = preprocess_args(args);
    let pp_output = PCommand::new(compiler)
        .args(&pp_args)
        .stdout(Stdio::piped())
        .output()?;
    if !pp_output.status.success() {
        return Ok(pp_output.status.code().unwrap_or(1));
    }

    let text = String::from_utf8_lossy(&pp_output.stdout);
    let mut encoded = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        encoded.push_str(&do_sub(line, false));
    }

    let tmp = tempfile_path();
    {
        let mut f = File::create(&tmp)?;
        f.write_all(encoded.as_bytes())?;
    }

    let mut compile_args: Vec<String> = args.to_vec();
    compile_args[input_index] = tmp.to_string_lossy().into_owned();

    let status = PCommand::new(compiler).args(&compile_args).status()?;

    let _ = std::fs::remove_file(&tmp);

    Ok(status.code().unwrap_or(1))
}

fn tempfile_path() -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let mut dir = std::env::temp_dir();
    dir.push(format!("sotn_str_cc_{}_{}.c", std::process::id(), nanos));
    dir
}

#[derive(serde::Deserialize)]
struct CodecRequest {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    bytes: Option<String>,
}

#[derive(serde::Serialize)]
struct CodecResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bytes: Option<String>,
}

fn parse_hex(value: &str) -> Result<Vec<u8>, String> {
    if !value.is_ascii() {
        return Err("hex byte string must contain only ASCII digits".to_string());
    }
    if value.len() % 2 != 0 {
        return Err("hex byte string must have an even number of digits".to_string());
    }
    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| format!("invalid hex byte at offset {index}"))
        })
        .collect()
}

fn codec_batch(operation: &str, platform: Platform) -> Result<(), String> {
    let stdin = io::stdin();
    let mut responses = Vec::new();
    for (line_number, line) in stdin.lock().lines().enumerate() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let request: CodecRequest = serde_json::from_str(&line)
            .map_err(|error| format!("line {}: invalid JSON: {error}", line_number + 1))?;
        let response = match operation {
            "encode" => {
                let text = request
                    .text
                    .ok_or_else(|| format!("line {}: missing text", line_number + 1))?;
                let bytes = encode_menu_string(&text, platform)
                    .map_err(|error| format!("line {}: {error}", line_number + 1))?;
                CodecResponse {
                    id: request.id,
                    text: None,
                    bytes: Some(bytes.iter().map(|byte| format!("{byte:02X}")).collect()),
                }
            }
            "decode" => {
                let bytes = request
                    .bytes
                    .ok_or_else(|| format!("line {}: missing bytes", line_number + 1))?;
                let bytes = parse_hex(&bytes)
                    .map_err(|error| format!("line {}: {error}", line_number + 1))?;
                let text = decode_menu_string(&bytes, platform)
                    .map_err(|error| format!("line {}: {error}", line_number + 1))?;
                CodecResponse {
                    id: request.id,
                    text: Some(text),
                    bytes: None,
                }
            }
            _ => return Err(format!("unsupported codec operation {operation:?}")),
        };
        responses.push(serde_json::to_string(&response).map_err(|error| error.to_string())?);
    }
    for response in responses {
        println!("{}", response);
    }
    Ok(())
}

fn main() {
    let matches = Command::new("string processor")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .about("String processor to interpret sequence of characters from SOTN")
        .subcommand(
            Command::new("process")
                .about("process a file")
                .arg(
                    Arg::new("filename")
                        .short('f')
                        .long("filename")
                        .help("Filename")
                        .action(ArgAction::Set)
                        .num_args(1),
                )
                .arg(
                    Arg::new("psp")
                        .short('p')
                        .long("psp")
                        .help("PSP")
                        .action(ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("cc")
                .about("use for C_COMPILER_LAUNCHER, re-encodes _S() strings on the fly")
                .trailing_var_arg(true)
                .allow_hyphen_values(true)
                .arg(
                    Arg::new("args")
                        .action(ArgAction::Append)
                        .num_args(0..)
                )
        )
        .subcommand(
            Command::new("codec")
                .about("encode or decode newline-delimited JSON without FFI")
                .subcommand_required(true)
                .subcommand(
                    Command::new("encode")
                        .arg(Arg::new("platform").long("platform").required(true)),
                )
                .subcommand(
                    Command::new("decode")
                        .arg(Arg::new("platform").long("platform").required(true)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        // Some(("parse", sub_m)) => {
        //     // let filename = sub_m.value_of("filename").unwrap();
        //     // let offset = sub_m.value_of("offset").unwrap();
        // }
        Some(("process", sub_m)) => {
            if sub_m.contains_id("filename") && sub_m.contains_id("psp")
            {
                let filename = sub_m
                .get_one::<String>("filename")
                .expect("is present");
                let _ = process(Some(filename.to_string()), true);
            }
            else if sub_m.contains_id("filename") {
                let filename = sub_m
                .get_one::<String>("filename")
                .expect("is present");
                let _ = process(Some(filename.to_string()), false);
            }
            else {
                let _ = process(None, false);
            }
        }
        Some(("cc", sub_m)) => {
            let args: Vec<String> = sub_m
                .get_many::<String>("args")
                .map(|v| v.cloned().collect())
                .unwrap_or_default();
            match cc(&args) {
                Ok(code) => std::process::exit(code),
                Err(e) => cc_fail(&format!("{}", e)),
            }
        }
        Some(("codec", sub_m)) => {
            let (operation, operation_matches) = sub_m.subcommand().unwrap();
            let platform = operation_matches
                .get_one::<String>("platform")
                .and_then(|value| Platform::parse(value).ok())
                .unwrap_or_else(|| cc_fail("codec platform must be psx or psp"));
            if let Err(error) = codec_batch(operation, platform) {
                cc_fail(&error);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex("21A0FF").unwrap(), vec![0x21, 0xA0, 0xFF]);
        assert!(parse_hex("123").is_err());
        assert!(parse_hex("AéB").is_err());
    }

    #[test]
    fn test_utf8_to_byte_literals_escaped()
    {
        assert_eq!(utf8_to_byte_literals_escaped("すで"), "\"\"\\xBD\\xC3\\xFF\\x9E\\xFF\"\"");
        assert_eq!(utf8_to_byte_literals_escaped("あかつきの剣"), "\"\"\\xB1\\xB6\\xC2\\xB7\\xC9\\x3C\\xFF\"\"");
        assert_eq!(utf8_to_byte_literals_escaped("聖なるめがね"), "\"\"\\xEE\\xC5\\xD9\\xD2\\xB6\\xFF\\x9E\\xC8\\xFF\"\"");
        assert_eq!(utf8_to_byte_literals_escaped("バルザイのえん月刀"), "\"\"\\x8A\\xFF\\x9E\\x99\\x7B\\xFF\\x9E\\x72\\xC9\\xB4\\xDD\\xFF\\xFF\\xED\\xFF\"\"");
        assert_eq!(utf8_to_byte_literals_escaped("Str. potion"), "\"\"\\x33\\x54\\x52\\x0E\\x00\\x50\\x4F\\x54\\x49\\x4F\\x4E\\xFF\"\"");
    }

    #[test]
    fn test_do_sub() {
        // Test case 1: Test _S("すで")
        let line = r#"{_S("すで"), "装備なし（素手）", 0, 0, 0, 3, 255, 0, 0, 36, 42, 0, 5, 128, 0, 0, false, 8, 0, 0, 0, 0, 4, 2, 1, 1, 1, 1, 0},"#;
        let out = do_sub(line, false);
        let expected = r#"{"\xBD\xC3\xFF\x9E\xFF", "装備なし（素手）", 0, 0, 0, 3, 255, 0, 0, 36, 42, 0, 5, 128, 0, 0, false, 8, 0, 0, 0, 0, 4, 2, 1, 1, 1, 1, 0},"#;
        assert_eq!(out, expected);

        // Test case 2: Test _S("")
        let line = r#"_S("")"#;
        let out = do_sub(line, false);
        let expected = r#""\xFF""#;
        assert_eq!(out, expected);

        // // Test case 3: Test _S with symbols and quotes
        // let line = r#"_S("\"(\")")"#;
        // let out = do_sub(line, false);
        // let expected = r#""\x02\x08\x02\x09\xFF""#;
        // assert_eq!(out, expected);

        // Test case 4: Test _S2("ＡＴＴ")
        let line = r#"_S2("ＡＴＴ")"#;
        let out = do_sub(line, false);
        let expected = r#""\x00\x01\x01\xFF""#;
        assert_eq!(out, expected);

        // Test case 5: Test _S2("")
        let line = r#"_S2("")"#;
        let out = do_sub(line, false);
        let expected = r#""\xFF""#;
        assert_eq!(out, expected);

        // Test case 6: Test _S2_HD("攻撃力")
        let line = r#"_S2_HD("攻撃力")"#;
        let out = do_sub(line, false);
        let expected = r#""\x0C\x0D\x0E\xFF""#;
        assert_eq!(out, expected);

        // Test case 7: Test _S2_HD("")
        let line = r#"_S2_HD("")"#;
        let out = do_sub(line, false);
        let expected = r#""\xFF""#;
        assert_eq!(out, expected);
    }

    #[test]
    fn more_do_sub()
    {
        let line = r#"{_S("たび人ぼう"), "旅人の基本装備である帽子", 0, 3, 0, 0, 1, 0, 0, 0x0000, 0x0000, 0x0000, 168, 168, 0, 0},"#;
        let out = do_sub(line, false);
        let expected = r#"{"\xC0\xCB\xFF\x9E\xA3\xCE\xFF\x9E\xB3\xFF", "旅人の基本装備である帽子", 0, 3, 0, 0, 1, 0, 0, 0x0000, 0x0000, 0x0000, 168, 168, 0, 0},"#;
        assert_eq!(out, expected);

        let line = r#"/* 0x18C */ {_S("Guardian"), 500, 50, 33, 34, 35, 0x0000, 0x0040, 0xE000, 0x0800, 60, 1500, 321, 255, 2, 1, 6, 24, 0x08403410},"#;
        let out = do_sub(line, false);
        let expected = r#"/* 0x18C */ {"\x27\x55\x41\x52\x44\x49\x41\x4E\xFF", 500, 50, 33, 34, 35, 0x0000, 0x0040, 0xE000, 0x0800, 60, 1500, 321, 255, 2, 1, 6, 24, 0x08403410},"#;
        assert_eq!(out, expected);

let line = r#"const char* g_goldCollectTexts[] = {
    _S("$1"),   _S("$25"),  _S("$50"),   _S("$100"),  _S("$250"),
    _S("$400"), _S("$700"), _S("$1000"), _S("$2000"), _S("$5000"),
};"#;

let expected = r#"const char* g_goldCollectTexts[] = {
    "\x04\x11\xFF",   "\x04\x12\x15\xFF",  "\x04\x15\x10\xFF",   "\x04\x11\x10\x10\xFF",  "\x04\x12\x15\x10\xFF",
    "\x04\x14\x10\x10\xFF", "\x04\x17\x10\x10\xFF", "\x04\x11\x10\x10\x10\xFF", "\x04\x12\x10\x10\x10\xFF", "\x04\x15\x10\x10\x10\xFF",
};"#;

        let out = do_sub(line, false);
        assert_eq!(out, expected);

        let line = r#"_S("ナイフ(サブ)"),"#;
        let out = do_sub(line, false);
        let expected = r#""\x85\x72\x8C\x08\x7B\x8C\xFF\x9E\x09\xFF","#;
        assert_eq!(out, expected);

        let line = r#"_S2_HD("その他")"#;
        let out = do_sub(line, false);
        let expected = r#""\x2C\x2D\x2E\xFF""#;
        assert_eq!(out, expected);

        let line = r#"_S2_HD("使い魔")"#;
        let out = do_sub(line, false);
        let expected = r#""\x0B\x2F\x11\xFF""#;
        assert_eq!(out, expected);
    }

    #[test]
    fn psp()
    {
        let line = r#"_S("Grand cœur")"#;
        let out = do_sub(line, true);
        let expected = r#""\x27\x52\x41\x4E\x44\x00\x43\xB1\x55\x52\xFF""#;
        assert_eq!(out, expected);

        let line = r#"_SE("Cadáveres frescos."),"#;
        let out = do_sub(line, true);
        let expected = r#""Cadﾌveres frescos.","#;
        assert_eq!(out, expected);
    }
}
