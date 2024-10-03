use std::fmt::{Debug, Formatter};

use mamba::check::check_all;
use mamba::check::result::TypeErr;
use mamba::common::result::WithSource;
use mamba::parse::parse;

pub mod invalid;
pub mod valid;

struct CheckTestErr(Vec<TypeErr>);

type CheckTestRet = Result<(), CheckTestErr>;

fn check_test(source: &String) -> CheckTestRet {
    check_all(&[*parse(&source).unwrap()])
        .map(|_| ())
        .map_err(|errs| {
            CheckTestErr(
                errs.into_iter()
                    .map(|err| err.with_source(&Some(source.clone()), &None))
                    .collect::<Vec<TypeErr>>(),
            )
        })
}

impl Debug for CheckTestErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.iter().map(|err| write!(f, "{}\n", err)).collect()
    }
}
