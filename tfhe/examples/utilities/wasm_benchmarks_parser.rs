#[path = "../../benches/utilities.rs"]
mod utilities;

use crate::utilities::{write_to_json, OperatorType};
use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use tfhe::keycache::NamedParam;
use tfhe::shortint::keycache::{
    PARAM_MESSAGE_1_CARRY_1_KS_PBS_NAME, PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS_NAME,
    PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_PBS_KS_NAME, PARAM_MESSAGE_2_CARRY_2_KS_PBS_NAME,
    PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64_NAME,
};
use tfhe::shortint::parameters::classic::compact_pk::{
    PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS, PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_PBS_KS,
};
use tfhe::shortint::parameters::classic::tuniform::p_fail_2_minus_64::ks_pbs::PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64;
use tfhe::shortint::parameters::{PARAM_MESSAGE_1_CARRY_1_KS_PBS, PARAM_MESSAGE_2_CARRY_2_KS_PBS};
use tfhe::shortint::{ClassicPBSParameters, PBSParameters};

const BENCHMARK_NAME_PREFIX: &str = "wasm::";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    raw_results_file: String,
}

fn params_from_name(name: &str) -> ClassicPBSParameters {
    match name.to_uppercase().as_str() {
        PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS_NAME => PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_KS_PBS,
        PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_PBS_KS_NAME => PARAM_MESSAGE_2_CARRY_2_COMPACT_PK_PBS_KS,
        PARAM_MESSAGE_1_CARRY_1_KS_PBS_NAME => PARAM_MESSAGE_1_CARRY_1_KS_PBS,
        PARAM_MESSAGE_2_CARRY_2_KS_PBS_NAME => PARAM_MESSAGE_2_CARRY_2_KS_PBS,
        PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64_NAME => {
            PARAM_MESSAGE_2_CARRY_2_KS_PBS_TUNIFORM_2M64
        }
        _ => panic!("failed to get parameters for name '{name}'"),
    }
}

fn write_result(file: &mut File, name: &str, value: usize) {
    let line = format!("{name},{value}\n");
    let error_message = format!("cannot write {name} result into file");
    file.write_all(line.as_bytes()).expect(&error_message);
}

pub fn parse_wasm_benchmarks(results_file: &Path, raw_results_file: &Path) {
    File::create(results_file).expect("create results file failed");
    let mut file = OpenOptions::new()
        .append(true)
        .open(results_file)
        .expect("cannot open parsed results file");

    let operator = OperatorType::Atomic;

    let raw_results = fs::read_to_string(raw_results_file).expect("cannot open raw results file");
    let results_as_json: HashMap<String, f32> = serde_json::from_str(&raw_results).unwrap();

    for (full_name, val) in results_as_json.iter() {
        let prefixed_full_name = format!("{BENCHMARK_NAME_PREFIX}{full_name}");
        let name_parts = full_name.split("_mean_").collect::<Vec<_>>();
        let bench_name = name_parts[0];
        let params: PBSParameters = params_from_name(name_parts[1]).into();
        let value_in_ns = (val * 1_000_000_f32) as usize;

        write_result(&mut file, &prefixed_full_name, value_in_ns);
        write_to_json::<u64, _>(
            &prefixed_full_name,
            params,
            params.name(),
            bench_name,
            &operator,
            0,
            vec![],
        );
    }
}

fn main() {
    let args = Args::parse();

    let work_dir = std::env::current_dir().unwrap();
    let mut new_work_dir = work_dir;
    new_work_dir.push("tfhe");
    std::env::set_current_dir(new_work_dir).unwrap();

    let results_file = Path::new("wasm_pk_gen.csv");
    let raw_results = Path::new(&args.raw_results_file);

    parse_wasm_benchmarks(results_file, raw_results);
}
