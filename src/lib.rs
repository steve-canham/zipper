pub mod setup;
pub mod err;
mod data;
mod zipper;
mod unzipper;

use std::sync::OnceLock;
use err::AppError;
use setup::log_helper;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub static LOG_RUNNING: OnceLock<bool> = OnceLock::new();

#[derive(sqlx::FromRow)]
pub struct SourceDetails {
    pub id: i32, 
    pub database_name: String, 
    pub local_folder: String, 
    pub local_files_grouped: bool, 
    pub grouping_range_by_id: Option<i32>
}

pub async fn run(args: Vec<OsString>) -> Result<(), AppError> {

    let config_file = PathBuf::from("./app_config.toml");
    let config_string: String = fs::read_to_string(&config_file)
                    .map_err(|e| AppError::IoReadErrorWithPath(e, config_file))?;
    
    let params = setup::get_params(args, config_string)?;
    let flags = params.flags;
    let test_run = flags.test_run;

    if !test_run {
       log_helper::setup_log(&params.log_folder_path)?;
       LOG_RUNNING.set(true).unwrap();   // no other thread - therefore should always work
       log_helper::log_startup_params(&params);
    }
    
    let pool = setup::get_db_pool().await?;

    if flags.use_folder {
         
         // call the appropriate zip or unzip fuunction with the folders concerned
         
         if flags.do_zip {
             zipper::zip_folder(&params.fdr_unzipped, &params.fdr_zipped)?;
         } else {
             unzipper::unzip_folder(&params.fdr_zipped, &params.fdr_unzipped)?;
         }
    }
    else {

        let mut source_list = params.source_list;
        if flags.all_mdr {
            source_list = data::get_all_ids(&pool).await?;  // get alll ids
        }

        for source_id in source_list.clone() {
            let source_dets = data::get_source_details(source_id, &pool).await?;
            if flags.do_zip {
                zipper::zip_mdr_folder(source_dets, &params.fdr_unzipped, &params.fdr_zipped)?;
            }
            else {
                unzipper::unzip_mdr_folder(source_dets, &params.fdr_unzipped, &params.fdr_zipped)?;
            }
        }      
    }

    Ok(())
}