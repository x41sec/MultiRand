use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::{env, fs, str};

use clap::{arg, command, value_parser};
use hex;

pub const IMPLS: &[&str] = &["ansic", "apple", "bcpl", "bcslib", "borland_c_lrand", "borland_c_rand", "c64_a", "c64_b", "c64_c", "cpp", "cray", "derive", "drand48", "glibc_old", "glibc_type_0", "lrand48", "maple", "minstd_16807", "minstd_48271", "mmix", "mrand48", "musl", "nag", "newlib_u16", "newlib", "numrecipes", "random0", "randu", "rtl_uniform", "simscript", "super_duper", "turbo_pascal", "urn12", "vbasic6", "zx81"];

struct Lcg {
    seed: i64,
    offset: usize,
    modulo: i64,
    mul: i64,
    inc: i64,
    lsb: u8,
    bitmask: i64,
}

impl Lcg {
    pub fn new(offset: Option<usize>, modulo: i64, mul: i64, inc: i64, msb: u8, lsb: u8) -> Self {
        let bitmask = if msb == 63 && lsb == 0 {
            i64::MAX
        } else {
            i64::pow(2, (msb - lsb + 1).into()) - 1
        };

        Self {
            seed: 0,
            offset: offset.unwrap_or(0), // None means uncertain, we haven't looked this up yet.
            modulo,
            mul,
            inc,
            lsb,
            bitmask,
        }
    }

    pub fn srand(&mut self, seed: i64, iter: usize) {
        self.seed = seed.into();
        for _ in 0..iter {
            self.rand();
        }
    }

    pub fn rand(&mut self) -> i64 {
        if self.modulo == 0 {
            self.seed = self.seed.wrapping_mul(self.mul);
            self.seed = self.seed.wrapping_add(self.inc);
        } else {
            self.seed = self.seed.wrapping_mul(self.mul) % self.modulo;
            self.seed = self.seed.wrapping_add(self.inc) % self.modulo;
        }

        return (self.seed >> self.lsb) & self.bitmask;
    }
}

impl Iterator for Lcg {
    type Item = i64;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.rand())
    }
}

// https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use
// http://citeseer.ist.psu.edu/viewdoc/download?doi=10.1.1.53.3686&rep=rep1&type=pdf
fn get_lcg(name: &str) -> Lcg {
    return match name {
        "ansic" => Lcg::new(None, i64::pow(2, 31), 1103515245, 12345, 30, 16),
        "apple" => Lcg::new(None, i64::pow(2, 35), 1220703125, 0, 63, 0),
        "bcpl" => Lcg::new(None, i64::pow(2, 32), 2147001325, 715136305, 63, 0),
        "bcslib" => Lcg::new(None, i64::pow(2, 35), i64::pow(5, 15), 261067085, 63, 0), // Boeing Computer Services
        "borland_c_lrand" => Lcg::new(Some(1), i64::pow(2, 32), 22695477, 1, 30, 0),
        "borland_c_rand" => Lcg::new(Some(1), i64::pow(2, 32), 22695477, 1, 30, 16),
        // "borland_delphi(seed: &mut u64, l: u64) -> u64 { Lcg::init(&mut (*seed * l), i64::pow(2, 32), 134775813, 1, 63, 32) } // takes extra parameter
        "c64_a" => Lcg::new(None, i64::pow(2, 23), 65793, 4282663, 22, 8),
        "c64_b" => Lcg::new(None, i64::pow(2, 32), 16843009, 826366247, 31, 16),
        "c64_c" => Lcg::new(None, i64::pow(2, 32), 16843009, 3014898611, 31, 16),
        "cpp" => Lcg::new(None, i64::pow(2, 32), 214013, 2531011, 30, 16),
        "cray" => Lcg::new(None, i64::pow(2, 48), 44485709377909, 0, 63, 0),
        "derive" => Lcg::new(None, i64::pow(2, 32), 3141592653, 1, 63, 0),
        "drand48" => Lcg::new(None, i64::pow(2, 48), 25214903917, 11, 47, 0), // also erand48
        "glibc_old" => Lcg::new(None, i64::pow(2, 32), 69069, 1, 63, 0),
        "glibc_type_0" => Lcg::new(Some(0), i64::pow(2, 32), 1103515245, 12345, 30, 0), // used by gcc
        "lrand48" => Lcg::new(None, i64::pow(2, 48), 25214903917, 11, 47, 16), // also nrand48 and java.util.Random
        "maple" => Lcg::new(None, i64::pow(10, 12) - 11, 427419669081, 0, 63, 0),
        "minstd_16807" => Lcg::new(None, i64::pow(2, 31) - 1, 16807, 0, 63, 0),
        "minstd_48271" => Lcg::new(None, i64::pow(2, 31) - 1, 48271, 0, 63, 0),
        "mmix" => Lcg::new(None, 0, 6364136223846793005, 1442695040888963407, 63, 0),
        "mrand48" => Lcg::new(None, i64::pow(2, 48), 25214903917, 11, 47, 15), // also jrand48
        "musl" => Lcg::new(None, 0, 6364136223846793005, 1, 63, 33),
        "nag" => Lcg::new(None, i64::pow(2, 59), i64::pow(13, 13), 0, 63, 0),
        "newlib_u16" => Lcg::new(None, 0, 6364136223846793005, 1, 46, 32),
        "newlib" => Lcg::new(None, 0, 6364136223846793005, 1, 62, 32),
        "numrecipes" => Lcg::new(None, i64::pow(2, 32), 1664525, 1013904223, 63, 0),
        "random0" => Lcg::new(None, 134456, 8121, 28411, 63, 0), // NOTE: result needs to be divided by 134456
        "randu" => Lcg::new(None, i64::pow(2, 31), 65539, 0, 63, 0),
        "rtl_uniform" => Lcg::new(None, i64::pow(2, 31) - 1, 2147483629, 2147483587, 63, 0),
        "simscript" => Lcg::new(None, i64::pow(2, 31) - 1, 630360016, 0, 63, 0),
        "super_duper" => Lcg::new(None, i64::pow(2, 32), 69069, 0, 63, 0),
        "turbo_pascal" => Lcg::new(None, i64::pow(2, 32), 134775813, 1, 63, 0),
        "urn12" => Lcg::new(None, i64::pow(2, 31), 452807053, 0, 63, 0),
        "vbasic6" => Lcg::new(None, i64::pow(2, 24), 1140671485, 12820163, 63, 0),
        "zx81" => Lcg::new(None, i64::pow(2, 16) + 1, 75, 74, 63, 0),
        _ => panic!("Unknown implementation {name}!"),
    };
}

trait DynInt: AsRef<[u8]> + IntoIterator<Item = u8> {
    fn to_bytes(val: i64) -> Self;
}

impl DynInt for [u8; 1] {
    fn to_bytes(val: i64) -> Self {
        (val as i8).to_be_bytes()
    }
}
impl DynInt for [u8; 2] {
    fn to_bytes(val: i64) -> Self {
        (val as i16).to_be_bytes()
    }
}
impl DynInt for [u8; 4] {
    fn to_bytes(val: i64) -> Self {
        (val as i32).to_be_bytes()
    }
}
impl DynInt for [u8; 8] {
    fn to_bytes(val: i64) -> Self {
        (val as i64).to_be_bytes()
    }
}

fn iterate<'n, B: DynInt>(
    rng: &mut Lcg,
    maxlen: usize,
    needles: &'n Vec<Vec<u8>>,
) -> (Option<&'n Vec<u8>>, usize) {
    let rand = rng.flat_map(B::to_bytes).take(maxlen);
    if needles.len() == 0 {
        let out: Vec<u8> = rand.collect();
        io::stdout().write_all(&out).unwrap();
    } else {
        let mut matchcounts = vec![0; needles.len()]; // holds the number of matched chars for each needle
        for (i, r) in rand.enumerate() {
            let mut give_up = 0;
            for (ns_i, mc) in matchcounts.iter_mut().enumerate() {
                let needle = &needles[ns_i]; // &Vec<u8>
                if needle.len() - *mc - 1 >= maxlen {
                    give_up += 1; // needle is longer than remainder of maxlen, so it can't be in there
                    continue;
                }
                if r == needle[*mc] {
                    *mc += 1; // found a matching char
                } else if r == needle[0] {
                    *mc = 1; // edge case where we're setting the match count to 0, but index 0 of the needle matches the current char
                } else {
                    *mc = 0; // reset match count to 0
                }
                if *mc >= needle.len() {
                    return (Some(needle), i + 1);
                }
            }
            if give_up == needles.len() {
                return (None, 0);
            }
        }
    }
    return (None, 0);
}

fn run(
    imp: &str,
    from: u64,
    to: u64,
    count: usize,
    offset: Option<usize>,
    intsize: u8,
    targets: &Vec<Vec<u8>>,
) {
    let mut rng = get_lcg(imp);
    let off = match offset {
        Some(x) => x,
        _ => rng.offset,
    };
    let bytes = (intsize as usize) / 8;

    let fun = match intsize {
        8 => iterate::<[u8; 1]>,
        16 => iterate::<[u8; 2]>,
        32 => iterate::<[u8; 4]>,
        64 => iterate::<[u8; 8]>,
        _ => panic!("Invalid int size {intsize}"),
    };

    for seed in from..=to {
        rng.srand(seed as i64, off);
        match fun(&mut rng, count * bytes, &targets) {
            (Some(ref res), i) => {
                println!(
                    "Found! {imp} seed={seed} bytes={}..{} (iteration={}..{}) -> 0x{}",
                    off + i - res.len(),
                    off + i,
                    (off + i - res.len()) / bytes,
                    (off + i) / bytes,
                    hex::encode(res)
                );
                return;
            }
            _ => (),
        }
    }

    if targets.len() > 0 {
        process::exit(1);
    }
}

fn main() {
    let matches = command!()
        .arg(arg!(-i --impl <IMPLS> "LCG implementations to use (comma separated), or \"all\". See --help for full list.").required(true).value_delimiter(',').long_help("LCG implementations to use (comma separated), or \"all\".\n".to_owned() + &IMPLS.join(", ")))
        .arg(
            arg!(-s --start <VALUE> "First seed to use")
                .required(true)
                .value_parser(value_parser!(u64))
        )
        .arg(
            arg!(-e --end <VALUE> "Last seed to use")
                .required(true)
                .value_parser(value_parser!(u64))
        )
        .arg(
            arg!(-o --offset <VALUE> "Initial, silent iterations per seed. Default is implementation specific.")
                .value_parser(value_parser!(usize))
        )
        .arg(
            arg!(-c --count <VALUE> "Number of iterations to run per seed")
                .required(true)
                .value_parser(value_parser!(usize))
        )
        .arg(
            arg!(-t --size <VALUE> "Integer size")
                .value_parser(value_parser!(u8))
                .default_value("64")
        )
        .arg(
            arg!(-m --match <FILE> "File with hex encoded matches to search for (whitespace separated)")
                .required(false)
                .value_parser(value_parser!(PathBuf))
        )
        .get_matches();

    let mut impls: Vec<_> = matches
        .get_many::<String>("impl")
        .unwrap()
        .map(|s| s.as_str())
        .collect();
    let from = matches.get_one::<u64>("start").unwrap();
    let to = matches.get_one::<u64>("end").unwrap();
    let count = matches.get_one::<usize>("count").unwrap();
    let off = matches.get_one::<usize>("offset");
    let size = matches.get_one::<u8>("size").unwrap();

    let input = match matches.get_one::<PathBuf>("match") {
        Some(file_path) => fs::read_to_string(file_path).unwrap(),
        _ => String::new(),
    };
    let targets = input
        .split_whitespace()
        .map(|s| hex::decode(s).expect("hex decoding failed!"))
        .collect();

    if impls[0] == "all" {
        impls = IMPLS.to_vec();
    }

    for imp in impls {
        run(imp, *from, *to, *count, off.copied(), *size, &targets);
    }
}
