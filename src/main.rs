use clap::Parser;
use colored::Colorize;
use cpp_demangle::Symbol;
use object::Object;
use std::{fs, path::PathBuf};
use threadpool::ThreadPool;
use walkdir::WalkDir;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// so source directory
    #[clap(short, long)]
    path: PathBuf,
    /// filter symbol name
    #[clap(short, long)]
    name: String,
}

fn main() {
    let args = Args::parse();
    let pool = ThreadPool::new(num_cpus::get());
    for entry in WalkDir::new(args.path).into_iter().filter_map(|e| e.ok()) {
        let filename = entry.into_path();
        let symbol_name = args.name.clone();
        pool.execute({
            move || {
                print_symbol(filename, &symbol_name);
            }
        })
    }
    pool.join();
}

fn print_symbol(filepath: PathBuf, symbol_name: &str) {
    if !filepath.to_str().unwrap().ends_with("so") {
        return;
    }
    let filename = filepath.clone();
    let filename = filename.file_name().unwrap().to_str().unwrap();
    let mut outbuff;
    let bin_data = fs::read(filepath).expect("read file err");
    let file =
        object::File::parse(&*bin_data).expect(format!("{} parse error", &filename).as_str());
    let exports = file.exports().unwrap();
    for export in exports {
        let name = export.name();
        if name[0] == b'_' && name[1] == b'Z' {
            let demangled = Symbol::new(name)
                .expect(format!("{} demangled err!!", std::str::from_utf8(name).unwrap()).as_str());
            let name_str = std::str::from_utf8(name).unwrap();
            outbuff = demangled.to_string();
            outbuff += &format!("\n\t\t{}", name_str).to_string();
        } else {
            let name_str = std::str::from_utf8(name).unwrap();
            outbuff = name_str.to_string();
        }

        let out_low = outbuff.to_lowercase();
        let matched_name=symbol_name.to_lowercase();
        
        if out_low.contains(&matched_name) {
            // let out = outbuff.replace(symbol_name, &out_replace);
            let offset=out_low.find(&matched_name).unwrap();
            let symbol_len=symbol_name.len();
            let out_replace=format!("{}", &outbuff[offset..offset+symbol_len]).red().to_string();
          outbuff.replace_range(offset..offset+symbol_len, &out_replace);
            println!("{}\t{}", format!("{}", filename).blue(), outbuff);
        }
    }
}
