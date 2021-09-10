#[macro_use]
extern crate log;

use humantime::format_duration;
use std::error;
use std::fs;
use std::io;
use std::io::Write;
use std::path;
use std::time::Instant;
use structopt::StructOpt;

// curl -L 'https://raw.githubusercontent.com/biolink/biolink-model/master/biolink-model.ttl' -o $@.tmp
// sed -E 's/<https:\/\/w3id.org\/biolink\/vocab\/([^[:space:]][^[:space:]]*):/<http:\/\/purl.obolibrary.org\/obo\/\1_/g' $@.tmp >$@

#[derive(StructOpt, Debug)]
#[structopt(name = "fix_biolink_model", about = "fix biolink model")]
struct Options {
    #[structopt(short = "f", long = "file", long_help = "biolink model file", required = true, parse(from_os_str))]
    file: path::PathBuf,
}
fn main() -> Result<(), Box<dyn error::Error>> {
    let start = Instant::now();
    env_logger::init();

    let options = Options::from_args();
    debug!("{:?}", options);

    let biolink_model_path: path::PathBuf = options.file;

    let data = fs::read_to_string(biolink_model_path.as_path())?;

    let prefix = "<https://w3id.org/biolink/vocab/";
    let prefix_replacement = "<http://purl.obolibrary.org/obo/";

    let mut lines: Vec<&str> = data.split("\n").collect();
    let biolink_model_file_tmp = fs::File::create(&biolink_model_path)?;
    let mut tmp_writer = io::BufWriter::new(biolink_model_file_tmp);
    let re = regex::Regex::new(r"^[^<https://w3id.org/biolink/vocab/logical_interpretation_enum>].+<https://w3id.org/biolink/vocab/(.+:.+)>")?;
    // let re = regex::Regex::new(r"<https://w3id.org/biolink/vocab/[^[:space:]][^[:space:]]*):>")?;
    for line in lines.iter_mut() {
        let mut line = line.to_string();
        if line.contains(prefix) && re.is_match(&line) {
            let second_part = &re.captures(&line).unwrap()[1];
            let fixed_second_part = &second_part.replacen(":", "_", 1);
            let fixed_line = line.replace(second_part, fixed_second_part).replace(&prefix, &prefix_replacement);
            //debug!("line: {}, fixed: {}", line, fixed_line);
            line = fixed_line;
        }
        tmp_writer.write_all(format!("{}\n", line).as_bytes()).expect("Unable to write data");
    }

    info!("Duration: {}", format_duration(start.elapsed()).to_string());
    Ok(())
}
