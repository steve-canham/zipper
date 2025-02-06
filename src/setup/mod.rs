/**********************************************************************************
* The setup module. Referenced in main by 'mod setup'.
* The two public modules allow integration tests to call into them, to give those
* tests the same DB conection pool and logging capability as the main library.
* The log established by log_helper seems to be available throughout the program
* via a suitable 'use' statement.
***********************************************************************************/

pub mod config_reader;
pub mod log_helper;
mod cli_reader;

/**********************************************************************************

***********************************************************************************/

use crate::error_defs::{AppError, CustomError};
use std::path::PathBuf;
use std::ffi::OsString;
use std::fs;
use config_reader::Config;

#[derive(Debug)]
pub struct CliPars {
    pub source_list: String,
    pub fz_folder: PathBuf,
    pub fu_folder: PathBuf,
    pub flags: Flags, 
}

#[derive(Debug, Clone, Copy)]
pub struct Flags {
    pub do_zip: bool,
    pub do_unzip: bool,
    pub all_mdr: bool,
    pub use_folder: bool,
    pub test_run: bool,
}

pub struct InitParams {
    pub mdr_zipped: PathBuf,
    pub mdr_unzipped: PathBuf,
    pub fdr_zipped: PathBuf,
    pub fdr_unzipped: PathBuf,
    pub log_folder_path: PathBuf,
    pub flags: Flags,
}

pub async fn get_params(args: Vec<OsString>) -> Result<InitParams, AppError> {

    // Called from main as the initial task of the program.
    // Returns a struct that contains the program's parameters.
    // Start by obtaining CLI arguments and reading parameters from .env file.
      
    let cli_pars = cli_reader::fetch_valid_arguments(args)?;

    let config_file_path = "./config_zipper.toml".to_string();
    let config_string: String = fs::read_to_string(config_file_path)?;

    let config_file: Config = config_reader::populate_config_vars(&config_string)?; 
    let file_pars = config_file.files;  // guaranteed to exist

    let empty_pb = PathBuf::from("");

    // if -a or -s flag check mdr zipping  folders exist.

    let mdr_zipped = file_pars.mdr_zipped;
    let mdr_unzipped = file_pars.mdr_unzipped;

    if cli_pars.flags.all_mdr || cli_pars.source_list.len() > 0 {

       // check mdr folders exit
       if mdr_zipped == empty_pb
       {
           let msg = "MDR based operation requested but parent folder for the zipped data not provided";
           let cf_err = CustomError::new(msg);
           return Result::Err(AppError::CsErr(cf_err));
       }

       if mdr_unzipped == empty_pb
       {
           let msg = "MDR based operation requested but parent folder for the unzipped data not provided";
           let cf_err = CustomError::new(msg);
           return Result::Err(AppError::CsErr(cf_err));
       }
    }
        
    // fdr folder paths may be available in CL arguments

    let mut fdr_zipped = cli_pars.fz_folder;
    if fdr_zipped == empty_pb
    {
        fdr_zipped = file_pars.fdr_zipped;
    }
    if cli_pars.flags.use_folder && fdr_zipped == empty_pb
    {
        let msg = "Folder based operation requested but no path provided for the zipped folder";
        let cf_err = CustomError::new(msg);
        return Result::Err(AppError::CsErr(cf_err));
    }
    
    let mut fdr_unzipped = cli_pars.fu_folder;
    if fdr_unzipped == empty_pb
    {
        fdr_unzipped = file_pars.fdr_unzipped;
    }
    if fdr_unzipped == empty_pb
    {
        let msg = "Folder based operation requested but no path provided for the unzipped folder";
        let cf_err = CustomError::new(msg);
        return Result::Err(AppError::CsErr(cf_err));
    }


    // if logging folder does not exist create it

    let mut log_folder = file_pars.log_folder_path;
    if log_folder == empty_pb {
        log_folder = PathBuf::from("E:\\MDR\\Zipping\\logs");
    }
    if !folder_exists (&log_folder) { 
        fs::create_dir_all(&log_folder)?;
    }

    // For execution flags read from the environment variables
       
    Ok(InitParams {
        mdr_zipped: mdr_zipped,
        mdr_unzipped: mdr_unzipped,
        fdr_zipped: fdr_zipped,
        fdr_unzipped: fdr_unzipped,
        log_folder_path: log_folder,
        flags: cli_pars.flags,
    })

}


fn folder_exists(folder_name: &PathBuf) -> bool {
    let xres = folder_name.try_exists();
    let res = match xres {
        Ok(true) => true,
        Ok(false) => false, 
        Err(_e) => false,           
    };
    res
}
        


/* 
// Tests
#[cfg(test)]

mod tests {
    use super::*;
      
    
    #[tokio::test]
    async fn check_env_vars_overwrite_blank_cli_values() {

        // Note that in most cases the folder path given must exist, and be 
        // accessible, or get_params will panic and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/data")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("output_file_name", Some("results 25.json")),
            ("data_version", Some("v1.60")),
            ("data_date", Some("2025-12-11")),

        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, false);
            assert_eq!(res.flags.export_text, false);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/data"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/outputs"));
            assert_eq!(res.source_file_name, "v1.58 20241211.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 25.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.58");
            assert_eq!(res.data_date, "2024-12-11");
        }
       ).await;

    }


    #[tokio::test]
    async fn check_cli_vars_overwrite_env_values() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/20241211 1.58 data")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_version", Some("v1.59")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 27.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-r", "-p", "-t", "-x",
                                     "-f", "E:/ROR/data", "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, true);
            assert_eq!(res.flags.export_csv, true);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/data"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 27.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");
        }
       ).await;

    }


    #[tokio::test]
    async fn check_cli_vars_with_i_flag() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/20241211 1.58 data")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 27.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-r", "-p", "-i", 
                                        "-f", "E:/ROR/data", "-d", "2026-12-25", "-s", "schema2 data.json"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, false);
            assert_eq!(res.flags.process_data, false);
            assert_eq!(res.flags.export_text, false);
            assert_eq!(res.flags.create_lookups,true);
            assert_eq!(res.flags.create_summary, true);
            assert_eq!(res.data_folder, PathBuf::new());
            assert_eq!(res.log_folder, PathBuf::new());
            assert_eq!(res.output_folder, PathBuf::new());
            assert_eq!(res.source_file_name, "".to_string());
            assert_eq!(res.output_file_name, "".to_string());
            assert_eq!(res.data_version, "".to_string());
            assert_eq!(res.data_date, "".to_string());
        }
       ).await;

    }


    #[tokio::test]
    async fn check_cli_vars_with_a_flag_and_new_win_folders() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:\\ROR\\20241211 1.58 data")),
            ("log_folder_path", Some("E:\\ROR\\some logs")),
            ("output_folder_path", Some("E:\\ROR\\dummy\\some outputs")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 28.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-a", "-f", "E:\\ROR\\data", 
                                       "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, true);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/data"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/some logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/dummy/some outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 28.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");
        }
      ).await;

    }
    
    #[tokio::test]
    async fn check_cli_vars_with_a_flag_and_new_posix_folders() {

        // Note that the folder path given must exist, 
        // and be accessible, or get_params will panic
        // and an error will be thrown. 

        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/data")),
            ("log_folder_path", Some("E:/ROR/some logs 2")),
            ("output_folder_path", Some("E:/ROR/dummy 2/some outputs")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 28.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-a", "-f", "E:/ROR/data", 
                                       "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
    
            assert_eq!(res.flags.import_ror, true);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, true);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/data"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/some logs 2"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/dummy 2/some outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 28.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");
        }
      ).await;

    }


    #[tokio::test]
    #[should_panic]
    async fn check_wrong_data_folder_panics_if_r() {
    
    temp_env::async_with_vars(
    [
        ("data_folder_path", Some("E:/ROR/20240607 1.47 data")),
        ("log_folder_path", Some("E:/ROR/some logs")),
        ("output_folder_path", Some("E:/ROR/dummy/some outputs")),
        ("src_file_name", Some("v1.58 20241211.json")),
        ("data_date", Some("2025-12-11")),
        ("output_file_name", Some("results 28.json")),
    ],
    async { 
        let args : Vec<&str> = vec!["target/debug/ror1.exe", "-a", "-f", "E:/silly folder name", 
                                    "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let _res = get_params(test_args).await.unwrap();
        }
      ).await;
    }

    #[tokio::test]
    async fn check_wrong_data_folder_does_not_panic_if_not_r() {
    
        temp_env::async_with_vars(
        [
            ("data_folder_path", Some("E:/ROR/daft data")),
            ("log_folder_path", Some("E:/ROR/some logs")),
            ("output_folder_path", Some("E:/ROR/dummy/some outputs")),
            ("src_file_name", Some("v1.58 20241211.json")),
            ("data_date", Some("2025-12-11")),
            ("output_file_name", Some("results 28.json")),
        ],
        async { 
            let args : Vec<&str> = vec!["target/debug/ror1.exe", "-p", "-f", "E:/ROR/silly folder name", 
                                        "-d", "2026-12-25", "-s", "schema2 data.json", "-v", "v1.60"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
            let res = get_params(test_args).await.unwrap();
            assert_eq!(res.flags.import_ror, false);
            assert_eq!(res.flags.process_data, true);
            assert_eq!(res.flags.export_text, false);
            assert_eq!(res.flags.create_lookups, false);
            assert_eq!(res.flags.create_summary, false);
            assert_eq!(res.data_folder, PathBuf::from("E:/ROR/silly folder name"));
            assert_eq!(res.log_folder, PathBuf::from("E:/ROR/some logs"));
            assert_eq!(res.output_folder, PathBuf::from("E:/ROR/dummy/some outputs"));
            assert_eq!(res.source_file_name, "schema2 data.json");
            let lt = Local::now().format("%m-%d %H%M%S").to_string();
            assert_eq!(res.output_file_name, format!("results 28.json at {}.txt", lt));
            assert_eq!(res.data_version, "v1.60");
            assert_eq!(res.data_date, "2026-12-25");

            }
        ).await;
    }

}
*/
