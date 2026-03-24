use fol_runtime::std as rt;
use fol_runtime::std as rt_model;

mod packages;

fn main() {
    let _runtime = rt::crate_name();
    let _runtime_tier = rt_model::tier_name();
    let _entry_package = "model_std_echo";
    let _entry_name = "main";
    let _ = (&_runtime, &_runtime_tier, &_entry_package, &_entry_name);
    let _ = packages::pkg__entry__model_std_echo::src::r__pkg__entry__model_std_echo__r0__main();
}
