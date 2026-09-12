//! sim-cli 入口 —— 只负责取参数与退出码，逻辑全在 lib.rs（便于测试）。

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    std::process::exit(sim_cli::main_with(&argv));
}
