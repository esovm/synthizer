#![feature(plugin, optin_builtin_traits)]
#![plugin(regex_macros, docopt_macros)]

extern crate docopt;
extern crate rustc_serialize;
extern crate vec_map;
#[macro_use]
extern crate interpreter;

docopt!(Args, "
Usage:
  synthizer stream <input>
  synthizer write <input> <output> [--length=<sec>]
  synthizer --help

Options:
  -h, --help             Show this message.
  -l, --length=<sec>     Length of audio to render, in seconds [default: 32].
", flag_length: f32);

use interpreter::common::{Context, read_file};
use interpreter::compiler::Compiler;
use interpreter::audio::{write_wav, play_stream};

#[allow(dead_code)]
fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    let filename = args.arg_input;
    let source = read_file(&filename).unwrap();
    let ctxt = Context::new(filename, source);
    let mut compiler = Compiler::new(&ctxt);
    compiler.define_entrypoint("main", make_fn_ty!(&ctxt, fn(time: Number) -> Number));
    match compiler.compile() {
        Ok(issues) => {
            println!("{}", issues);
            if args.cmd_write {
                write_wav(&compiler, args.arg_output, args.flag_length);
            } else if args.cmd_stream {
                play_stream(&compiler);
            }
        },
        Err(issues) => println!("Compile Error!\n{}", issues),
    }
}
