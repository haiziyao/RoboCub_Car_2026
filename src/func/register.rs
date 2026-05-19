use crate::config::{FuncParam, FuncParamConfig, FuncReturnConfig};
use crate::func::FuncWorkerMap;
use crate::func::tarits::*;
use crate::func::usual::*;

pub fn register_func(cfg: FuncParamConfig) -> FuncWorkerMap {
    let FuncParamConfig { func_param_list } = cfg;
    let mut map = FuncWorkerMap::new();
    func_param_list.iter().for_each(|x| {
        let FuncParam {
            function_id,
            returns,
            args,
        } = &x;
        map.add(function_id, function_factory(function_id, returns, args));
    });
    map
}

fn function_factory(function_id: &str, returns: &FuncReturnConfig, args: &[String]) -> FunctionDef {
    match function_id {
        "debug_fun" => FunctionDef::new(function_id, args.to_owned(), returns.clone(), fn_debug),
        "color_detect" => FunctionDef::new(
            function_id,
            args.to_owned(),
            returns.clone(),
            fn_color_detect,
        ),
        "qr_detect" => {
            FunctionDef::new(function_id, args.to_owned(), returns.clone(), fn_qr_detect)
        }
        "cross_detect" => FunctionDef::new(
            function_id,
            args.to_owned(),
            returns.clone(),
            fn_cross_detect,
        ),
        _ => panic!("unknown function_id `{function_id}`"),
    }
}
