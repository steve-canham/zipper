pub mod setup;
//mod import;
//mod process;
//mod summarise;
//mod export;
pub mod error_defs;

use error_defs::AppError;
use setup::log_helper;
use std::ffi::OsString;

pub async fn run(args: Vec<OsString>) -> Result<(), AppError> {
    
    
    println!("Hello, world!");
    
    let params = setup::get_params(args).await?;
    let flags = params.flags;
    let test_run = flags.test_run;

    if !test_run {
       log_helper::setup_log(&params.log_folder_path)?;
       log_helper::log_startup_params(&params);
    }

    Ok(())
}