use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuncReturnConfig {
    pub web: bool,
    pub gpio: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FuncParam {
    pub function_id: String,
    pub returns: FuncReturnConfig,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct FuncParamConfig {
    pub func_param_list: Vec<FuncParam>,
}
