/**********************************************************************************
The setup module, and the get_params function in this file in particular, 
orchestrates the collection and fusion of parameters as provided in 
1) a config toml file, and 
2) command line arguments. 
Where a parameter may be given in either the config file or command line, the 
command line version always over-writes anything from the file.
The module also checks the parameters for completeness (those required will vary, 
depending on the activity specified). If possible, defaults are used to stand in for 
mising parameters. If not possible the program stops with a message explaining the 
problem.
The module also provides a database connection pool on demand.
***********************************************************************************/

pub mod config_reader;
pub mod log_helper;
mod cli_reader;

use crate::err::AppError;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions, PgPool};
use std::time::Duration;
use sqlx::ConnectOptions;
use std::path::PathBuf;
use std::ffi::OsString;
use std::fs;
use config_reader::Config;
use cli_reader::Flags;

pub struct InitParams {
    pub mdr_zipped: PathBuf,
    pub mdr_unzipped: PathBuf,
    pub fdr_zipped: PathBuf,
    pub fdr_unzipped: PathBuf,
    pub log_folder_path: PathBuf,
    pub source_list: Vec<i32>,
    pub flags: Flags,
}

pub fn get_params(args: Vec<OsString>, config_string: String) -> Result<InitParams, AppError> {

    // Called from main as the initial task of the program.
    // Returns a struct that contains the program's parameters.
    // Start by obtaining CLI arguments and reading parameters from .env file.
      
    let cli_pars = cli_reader::fetch_valid_arguments(args)?;

    let config_file: Config = config_reader::populate_config_vars(&config_string)?; 
    let file_pars = config_file.files;  // guaranteed to exist
    let empty_pb = PathBuf::from("");
  
    let mdr_zipped = file_pars.mdr_zipped;
    let mdr_unzipped = file_pars.mdr_unzipped;

    let mut source_list = Vec::new();
    if cli_pars.source_list.len() > 0
    {
        let ids_as_strs: Vec<&str> = cli_pars.source_list.split(',').collect(); 
        for sid in ids_as_strs {
            match sid.trim().parse::<i32>() {
                Ok(id) => source_list.push(id),
                Err(_) => {}   // do nothing with this id, whatever the error
            }
        }
    }

    let mut fdr_zipped = cli_pars.fz_folder;
    if fdr_zipped == empty_pb
    {
        fdr_zipped = file_pars.fdr_zipped;
    }
    
    let mut fdr_unzipped = cli_pars.fu_folder;
    if fdr_unzipped == empty_pb
    {
        fdr_unzipped = file_pars.fdr_unzipped;
    }

    if cli_pars.flags.all_mdr || source_list.len() > 0 {

        // if -a or -s flag check mdr zipping  folders exist.
       
       if mdr_zipped == empty_pb
       {
        return Result::Err(AppError::MissingProgramParameter("mdr_zipped".to_string()));
       }

       if mdr_unzipped == empty_pb
       {
        return Result::Err(AppError::MissingProgramParameter("mdr_unipped".to_string()));
       }
    }

    if cli_pars.flags.use_folder { 

        if fdr_zipped == empty_pb
        {
            return Result::Err(AppError::MissingProgramParameter("fdr_zipped".to_string()));
        }
        if fdr_unzipped == empty_pb
        {
            return Result::Err(AppError::MissingProgramParameter("fdr_unzipped".to_string()));
        }
    }

    if cli_pars.flags.use_folder && (cli_pars.flags.all_mdr || source_list.len() > 0) {   
        let msg = "Both folder and mdr processing requested at the same time!".to_string();
        return Result::Err(AppError::InconsistentProgramParameter(msg));
    }
 
    if !cli_pars.flags.all_mdr && !cli_pars.flags.use_folder && source_list.len() == 0 {   
        let msg = "No source type has been identified for zipping or unzipping!".to_string();
        return Result::Err(AppError::InconsistentProgramParameter(msg));
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
        source_list: source_list,
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
        
pub async fn get_db_pool() -> Result<PgPool, AppError> {  

    // Establish DB name and thence the connection string
    // (done as two separate steps to allow for future development).
    // Use the string to set up a connection options object and change 
    // the time threshold for warnings. Set up a DB pool option and 
    // connect using the connection options object.

    let db_name = match config_reader::fetch_db_name() {
        Ok(n) => n,
        Err(e) => return Err(e),
    };

    let db_conn_string = config_reader::fetch_db_conn_string(&db_name)?;  
   
    let mut opts: PgConnectOptions = db_conn_string.parse()
                     .map_err(|e| AppError::DBPoolError("Problem with parsing conection string".to_string(), e))?;

    opts = opts.log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(3));

    PgPoolOptions::new()
        .max_connections(5) 
        .connect_with(opts).await
        .map_err(|e| AppError::DBPoolError(format!("Problem with connecting to database {} and obtaining Pool", db_name), e))
}


// Tests
#[cfg(test)]


mod tests {
    use super::*;

    // Ensure the parameters are being correctly combined.

    #[test]
    fn check_config_vars_overwrite_blank_cli_values() {

        let config = r#"
[files]
mdr_zipped="E:\\MDR\\Zipped source files"
mdr_unzipped="E:\\MDR\\MDR Source files"

fdr_zipped="E:\\MDR source data\\UMLS\\zipped data"
fdr_unzipped="E:\\MDR source data\\UMLS\\data"

log_folder_path="E:\\MDR\\Zipping\\logs"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="mon"
"#;

        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-z", "-m"];   // one of z, u and one of m, f, s esential
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = get_params(test_args, config_string).unwrap();

        assert_eq!(res.flags.all_mdr, true);
        assert_eq!(res.flags.use_folder, false);
        assert_eq!(res.flags.do_zip, true);
        assert_eq!(res.flags.do_unzip, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.mdr_zipped, PathBuf::from("E:\\MDR\\Zipped source files"));
        assert_eq!(res.mdr_unzipped, PathBuf::from("E:\\MDR\\MDR Source files"));
        assert_eq!(res.fdr_zipped, PathBuf::from("E:\\MDR source data\\UMLS\\zipped data"));
        assert_eq!(res.fdr_unzipped, PathBuf::from("E:\\MDR source data\\UMLS\\data"));
        assert_eq!(res.log_folder_path, PathBuf::from("E:\\MDR\\Zipping\\logs"));
        assert_eq!(res.source_list, Vec::<i32>::new());
    }


    #[test]
    fn check_cli_vars_overwrite_env_values() {

    let config = r#"
[files]
mdr_zipped="E:\\MDR\\Zipped source files"
mdr_unzipped="E:\\MDR\\MDR Source files"

fdr_zipped="E:\\MDR source data\\UMLS\\zipped data"
fdr_unzipped="E:\\MDR source data\\UMLS\\data"

log_folder_path="E:\\MDR\\Zipping\\logs"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="mon"
"#;

        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-z", "-f", 
                             "--fz", "E:\\MDR source data\\funny\\zipped data", "--fu", "E:\\MDR source data\\funny\\data"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        
        let res = get_params(test_args, config_string).unwrap();

        assert_eq!(res.flags.do_zip, true);
        assert_eq!(res.flags.all_mdr, false);
        assert_eq!(res.flags.use_folder, true);
        assert_eq!(res.flags.do_unzip, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.mdr_zipped, PathBuf::from("E:\\MDR\\Zipped source files"));
        assert_eq!(res.mdr_unzipped, PathBuf::from("E:\\MDR\\MDR Source files"));
        assert_eq!(res.fdr_zipped, PathBuf::from("E:\\MDR source data\\funny\\zipped data"));
        assert_eq!(res.fdr_unzipped, PathBuf::from("E:\\MDR source data\\funny\\data"));
        assert_eq!(res.log_folder_path, PathBuf::from("E:\\MDR\\Zipping\\logs"));
        assert_eq!(res.source_list, Vec::<i32>::new());
    }


    #[test]
    fn check_s_value_interpreted_correctly() {

    let config = r#"
[files]
mdr_zipped="E:\\MDR\\Zipped source files"
mdr_unzipped="E:\\MDR\\MDR Source files"

log_folder_path="E:\\MDR\\Zipping\\logs"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="mon"
"#;

        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-z", "-s", "101, 102, 103, 104, 105"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        
        let res = get_params(test_args, config_string).unwrap();

        assert_eq!(res.flags.do_zip, true);
        assert_eq!(res.flags.all_mdr, false);
        assert_eq!(res.flags.use_folder, false);
        assert_eq!(res.flags.do_unzip, false);
        assert_eq!(res.flags.test_run, false);
        assert_eq!(res.mdr_zipped, PathBuf::from("E:\\MDR\\Zipped source files"));
        assert_eq!(res.mdr_unzipped, PathBuf::from("E:\\MDR\\MDR Source files"));
        assert_eq!(res.fdr_zipped, PathBuf::from(""));
        assert_eq!(res.fdr_unzipped, PathBuf::from(""));
        assert_eq!(res.log_folder_path, PathBuf::from("E:\\MDR\\Zipping\\logs"));
        assert_eq!(res.source_list, vec![101, 102, 103, 104, 105]);
    }

    #[test]
    #[should_panic]
    fn check_z_and_u_panics() {
    
    let config = r#"
[files]
mdr_zipped="E:\\MDR\\Zipped source files"
mdr_unzipped="E:\\MDR\\MDR Source files"

log_folder_path="E:\\MDR\\Zipping\\logs"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="mon"
"#;

        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-z", "-u"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let _res = get_params(test_args, config_string).unwrap();
    }


    #[test]
    #[should_panic]
    fn check_f_and_m_panics() {
    
    let config = r#"
[files]
mdr_zipped="E:\\MDR\\Zipped source files"
mdr_unzipped="E:\\MDR\\MDR Source files"

log_folder_path="E:\\MDR\\Zipping\\logs"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="mon"
"#;

        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-z", "-f", "-m"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let _res = get_params(test_args, config_string).unwrap();
    }

    #[test]
    #[should_panic]
    fn check_f_and_s_panics() {
    
        let config = r#"
    [files]
    mdr_zipped="E:\\MDR\\Zipped source files"
    mdr_unzipped="E:\\MDR\\MDR Source files"
    
    fdr_zipped="E:\\MDR source data\\UMLS\\zipped data"
    fdr_unzipped="E:\\MDR source data\\UMLS\\data"
    
    log_folder_path="E:\\MDR\\Zipping\\logs"
    
    [database]
    db_host="localhost"
    db_user="user_name"
    db_password="password"
    db_port="5433"
    db_name="mon"
    "#;
    
            let config_string = config.to_string();
            config_reader::populate_config_vars(&config_string).unwrap();
    
            let args : Vec<&str> = vec!["dummy target", "-z", "-f", "-s", "101, 102, 103"];
            let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
    
            let _res = get_params(test_args, config_string).unwrap();
        }


    #[test]
    #[should_panic]
    fn check_no_z_or_u_panics() {
    
    let config = r#"
[files]
mdr_zipped="E:\\MDR\\Zipped source files"
mdr_unzipped="E:\\MDR\\MDR Source files"
log_folder_path="E:\\MDR\\Zipping\\logs"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="mon"
"#;

        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-m"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let _res = get_params(test_args, config_string).unwrap();
    }


    #[test]
    #[should_panic]
    fn check_no_m_s_or_f_panics() {
    
    let config = r#"
[files]
mdr_zipped="E:\\MDR\\Zipped source files"
mdr_unzipped="E:\\MDR\\MDR Source files"
log_folder_path="E:\\MDR\\Zipping\\logs"

[database]
db_host="localhost"
db_user="user_name"
db_password="password"
db_port="5433"
db_name="mon"
"#;

        let config_string = config.to_string();
        config_reader::populate_config_vars(&config_string).unwrap();

        let args : Vec<&str> = vec!["dummy target", "-z"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let _res = get_params(test_args, config_string).unwrap();
    }
}

