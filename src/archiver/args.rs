use argh::FromArgs;
use std::path::PathBuf;

#[derive(FromArgs, Debug)]
#[argh(description = "Compress your files with zstd and make it a self-extracting executable")]
pub struct Args {
    #[argh(positional)]
    #[argh(description = "what to compress")]
    pub target: PathBuf,

    #[argh(positional)]
    #[argh(description = "path to unarchiver executable")]
    pub unarchiver: PathBuf,

    #[argh(option, short = 'l', default = "3")]
    #[argh(description = "zstd compress level, range from 0~23, default is 3")]
    pub level: i32,

    #[argh(option, short = 'o')]
    #[argh(description = "path to output sfx file, default is target name")]
    pub output: Option<PathBuf>,

    #[argh(switch, short = 'r')]
    #[argh(description = "the root directory with be included or not")]
    pub root: bool,

    #[argh(option, short = 't')]
    #[argh(description = "where temp files should be store, default depends on os")]
    pub temp: Option<PathBuf>
}
