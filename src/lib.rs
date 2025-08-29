mod opts;
mod process;

pub use opts::{Opts, SubCommand};
pub use process::{process_csv, process_genpass};

mod test {
    #[test]
    fn add_two() {
        assert_eq!(1 + 1, 2);
    }
}
