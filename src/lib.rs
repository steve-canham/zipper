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
    
    // Important that there are no errors in the intial three steps.
    // If one does occur the program exits.
    // 1) Collect initial parameters such as file names and CLI flags. 
    // CLI arguments are collected explicitly to facilitate unit testing. 
    // of 'get_params'. Relevant environmental variables are also read.
    // 2) Establish a log file, in the specified data folder.
    // The initial parameters are recorded as the initial part of the log.
    // 3) The database connection pool is established for the database "ror".

    println!("Hello, world!");
    
    let params = setup::get_params(args).await?;
    let flags = params.flags;
    let test_run = flags.test_run;

    if !test_run {
       log_helper::setup_log(&params.log_folder, &params.source_file_name)?;
       log_helper::log_startup_params(&params);
    }
            
    let _pool = setup::get_db_pool().await?;

    Ok(())
}