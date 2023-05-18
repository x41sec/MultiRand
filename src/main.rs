use std::env;
use std::io::{self, Write};

use clap::{arg, command, value_parser, ArgGroup};
use hex;

struct Lcg {
    seed: i64,
    modulo: i64,
    mul: i64,
    inc: i64,
    lsb: u8,
    bitmask: i64,
}

impl Lcg {
    pub fn new(modulo: i64, mul: i64, inc: i64, msb: u8, lsb: u8) -> Self {
        let bitmask = if msb == 63 && lsb == 0 {
            i64::MAX
        } else {
            i64::pow(2, (msb - lsb + 1).into()) - 1
        };

        Self {
            seed: 0,
            modulo,
            mul,
            inc,
            lsb,
            bitmask,
        }
    }

    pub fn srand(&mut self, seed: i64) {
        self.seed = seed.into();
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
        "ansic" => Lcg::new(i64::pow(2, 31), 1103515245, 12345, 30, 16),
        "apple" => Lcg::new(i64::pow(2, 35), 1220703125, 0, 63, 0),
        "bcpl" => Lcg::new(i64::pow(2, 32), 2147001325, 715136305, 63, 0),
        "bcslib" => Lcg::new(i64::pow(2, 35), i64::pow(5, 15), 261067085, 63, 0), // Boeing Computer Services
        "borland_c_lrand" => Lcg::new(i64::pow(2, 32), 22695477, 1, 30, 0),
        "borland_c_rand" => Lcg::new(i64::pow(2, 32), 22695477, 1, 30, 16),
        // "borland_delphi(seed: &mut u64, l: u64) -> u64 { Lcg::init(&mut (*seed * l), i64::pow(2, 32), 134775813, 1, 63, 32) } // takes extra parameter
        "c64_a" => Lcg::new(i64::pow(2, 23), 65793, 4282663, 22, 8),
        "c64_b" => Lcg::new(i64::pow(2, 32), 16843009, 826366247, 31, 16),
        "c64_c" => Lcg::new(i64::pow(2, 32), 16843009, 3014898611, 31, 16),
        "cpp" => Lcg::new(i64::pow(2, 32), 214013, 2531011, 30, 16),
        "cray" => Lcg::new(i64::pow(2, 48), 44485709377909, 0, 63, 0),
        "derive" => Lcg::new(i64::pow(2, 32), 3141592653, 1, 63, 0),
        "drand48" => Lcg::new(i64::pow(2, 48), 25214903917, 11, 47, 0), // also erand48
        "glibc_old" => Lcg::new(i64::pow(2, 32), 69069, 1, 63, 0),
        "glibc_type_0" => Lcg::new(i64::pow(2, 32), 1103515245, 12345, 30, 0), // used by gcc
        "lrand48" => Lcg::new(i64::pow(2, 48), 25214903917, 11, 47, 16), // also nrand48 and java.util.Random
        "maple" => Lcg::new(i64::pow(10, 12) - 11, 427419669081, 0, 63, 0),
        "minstd_16807" => Lcg::new(i64::pow(2, 31) - 1, 16807, 0, 63, 0),
        "minstd_48271" => Lcg::new(i64::pow(2, 31) - 1, 48271, 0, 63, 0),
        "mmix" => Lcg::new(0, 6364136223846793005, 1442695040888963407, 63, 0),
        "mrand48" => Lcg::new(i64::pow(2, 48), 25214903917, 11, 47, 15), // also jrand48
        "musl" => Lcg::new(0, 6364136223846793005, 1, 63, 33),
        "nag" => Lcg::new(i64::pow(2, 59), i64::pow(13, 13), 0, 63, 0),
        "newlib_u16" => Lcg::new(0, 6364136223846793005, 1, 46, 32),
        "newlib" => Lcg::new(0, 6364136223846793005, 1, 62, 32),
        "numrecipes" => Lcg::new(i64::pow(2, 32), 1664525, 1013904223, 63, 0),
        "random0" => Lcg::new(134456, 8121, 28411, 63, 0), // NOTE: result needs to be divided by 134456
        "randu" => Lcg::new(i64::pow(2, 31), 65539, 0, 63, 0),
        "rtl_uniform" => Lcg::new(i64::pow(2, 31) - 1, 2147483629, 2147483587, 63, 0),
        "simscript" => Lcg::new(i64::pow(2, 31) - 1, 630360016, 0, 63, 0),
        "super_duper" => Lcg::new(i64::pow(2, 32), 69069, 0, 63, 0),
        "turbo_pascal" => Lcg::new(i64::pow(2, 32), 134775813, 1, 63, 0),
        "urn12" => Lcg::new(i64::pow(2, 31), 452807053, 0, 63, 0),
        "vbasic6" => Lcg::new(i64::pow(2, 24), 1140671485, 12820163, 63, 0),
        "zx81" => Lcg::new(i64::pow(2, 16) + 1, 75, 74, 63, 0),
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

fn get_random_bytes<B: DynInt>(
    rng: &mut Lcg,
    count: usize,
    intsize: usize,
    target: &Option<Vec<u8>>,
) -> Vec<u8> {
    if let Some(target) = target {
        rng.flat_map(B::to_bytes)
            .zip(target)
            .take_while(|(r, t)| r == *t)
            .map(|(r, _t)| r)
            .collect()
    } else {
        rng.flat_map(B::to_bytes)
            .take(count * (intsize / 8))
            .collect()
    }
}

fn run(imp: &str, from: u64, to: u64, count: usize, intsize: u8, target: Option<Vec<u8>>) {
    let mut rng = get_lcg(imp);

    let fun = match intsize {
        8 => get_random_bytes::<[u8; 1]>,
        16 => get_random_bytes::<[u8; 2]>,
        32 => get_random_bytes::<[u8; 4]>,
        64 => get_random_bytes::<[u8; 8]>,
        _ => panic!("Invalid int size {intsize}"),
    };

    for seed in from..=to {
        rng.srand(seed as i64);
        let out = fun(&mut rng, count, intsize as usize, &target);
        match target {
            Some(ref target) if &out == target => println!("Found! seed={seed}"),
            Some(_) => (),
            _ => io::stdout().write_all(&out).unwrap(),
        }
    }
}

fn main() {
    let matches = command!()
        .arg(arg!(-i --impl <IMPL> "LCG implementation to use").required(true))
        .arg(
            arg!(-s --start <VALUE> "First seed to use")
                .required(true)
                .value_parser(value_parser!(u64)),
        )
        .arg(
            arg!(-e --end <VALUE> "Last seed to use")
                .required(true)
                .value_parser(value_parser!(u64)),
        )
        .arg(
            arg!(-c --count <VALUE> "Number of iterations per seed")
                .value_parser(value_parser!(usize))
                .default_value("0"),
        )
        .arg(
            arg!(-t --size <VALUE> "Integer size")
                .value_parser(value_parser!(u8))
                .default_value("64"),
        )
        .arg(arg!([target] "hex value to search for"))
        .group(
            ArgGroup::new("vers")
                .args(["count", "target"])
                .required(true),
        )
        .get_matches();

    let imp = matches.get_one::<String>("impl").unwrap();
    let from = matches.get_one::<u64>("start").unwrap();
    let to = matches.get_one::<u64>("end").unwrap();
    let count = matches.get_one::<usize>("count").unwrap();
    let size = matches.get_one::<u8>("size").unwrap();
    let tgt = matches.get_one::<String>("target");

    let target = tgt.map(|tgt| hex::decode(tgt).expect("hex decoding failed!"));

    run(imp, *from, *to, *count, *size, target);
}
