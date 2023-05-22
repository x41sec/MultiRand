# multirand

Implements insecure, pseudo-random [linear congruential generators](https://en.wikipedia.org/wiki/Linear_congruential_generator) using parameters of various common `rand()` implementations.


## Usage

```shell
multirand --impl <IMPLS> --start <VALUE> --end <VALUE> <--count <VALUE>|target>
```

Options:
  - `-i`, `--impl <IMPLS>`   LCG implementations to use (comma separated), or "all". See `--help` for full list.
  - `-s`, `--start <VALUE>`  First seed to use
  - `-e`, `--end <VALUE>`    Last seed to use
  - `-c`, `--count <VALUE>`  Number of iterations per seed
  - `-t`, `--size <VALUE>`   Integer size [default: 64]
  - `[target]`               Value to search for (hex string)

## Seeds

Common initial (uninitalized) seeds are `0`, `1`, or `12345` (ANSI C).  
The NAG Library uses `123456789 * (2^32 + 1)`.  
The current UNIX epoch is another common seed value.

Note that some `srand()` implementations set the seed and subsequently call `rand()` internally, which advances the seed. To compensate for this, calculate the initial seed manually or increase the `--count` by 1 and discard the first output.

## Note

Only glibc's `TYPE_0` implementation has been checked for producting the same output with `rand()`. Other parameters have been taken from [a Wikipedia list](https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use) and [a paper](http://citeseer.ist.psu.edu/viewdoc/download?doi=10.1.1.53.3686&rep=rep1&type=pdf), but not yet checked for implementation equality.

## Example

```shell
# use seeds 1 and 2, call glibc rand() 2 times each
$ multirand -i glibc_type_0 --start 1 --end 2 --count 2 -t64 | xxd -p -c 8
0000000041c67ea6
00000000167eb0e7
00000000038ccd13
0000000059214b50

# use seeds 1 and 2, call glibc rand() 2 times each, while casting outputs to 16 bit int
$ multirand -i glibc_type_0 --start 1 --end 2 --count 2 -t16 | xxd -p -c 2
7ea6
b0e7
cd13
4b50

# use seeds 1 and 2, call glibc rand() 2 times each, while casting outputs to 8 bit int
$ multirand -i glibc_type_0 --start 1 --end 2 --count 2 -t8 | xxd -p -c 1
a6
e7
13
50

# use seeds 1...10, call glibc rand() 16 times each, while casting outputs to 8 bit int
$ multirand -i glibc_type_0 --start 1 --end 10 --count 16 -t8 | xxd -p -c 16
a6e7943d328300397edf2cf58afb1871
1350494e6f7c055a8b6881266714bdb2
80b9fe5fac750a7b98f1d657442d62f3
ed22b370e96e0f9ca57a2b8821460734
5a8b6881266714bdb20380b9fe5fac75
c7f41d92636019debf8cd5eadb7851b6
345dd2a3a0591effcc152a1bb891f6f7
a1c687b4dd522320d99e7f4c95aa9b38
0e2f3cc51a4b2841e627d47d72c34079
7b98f1d657442d62f3b029ae4fdce5ba

# find the seed for a certain value
$ multirand -i glibc_type_0 --start 1 --end 10 -t8 345dd2a3a0591effcc152a1bb891f6f7
Found! glibc_type_0 seed=7

```