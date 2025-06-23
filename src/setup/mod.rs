pub mod config_reader;
pub mod log_helper;
pub mod cli_reader;

use crate::err::AppError;
use sqlx::postgres::{PgPoolOptions, PgConnectOptions, PgPool};
use std::time::Duration;
use sqlx::ConnectOptions;
use std::path::PathBuf;

use std::fs;
use config_reader::Config;
use cli_reader::{CliPars, Flags};
use std::sync::OnceLock;

pub struct InitParams {
    pub mdr_zipped: PathBuf,
    pub mdr_unzipped: PathBuf,
    pub fdr_zipped: PathBuf,
    pub fdr_unzipped: PathBuf,
    pub log_folder_path: PathBuf,
    pub source_list: Vec<i32>,
    pub flags: Flags,
}

pub static LOG_RUNNING: OnceLock<bool> = OnceLock::new();

pub fn get_params(cli_pars: CliPars, config_string: &String) -> Result<InitParams, AppError> {

    let config_file: Config = config_reader::populate_config_vars(&config_string)?; 
    let folder_pars = config_file.folders;  // guaranteed to exist
    let empty_pb = PathBuf::from("");
  
    let mdr_zipped = folder_pars.mdr_zipped;
    let mdr_unzipped = folder_pars.mdr_unzipped;

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
        fdr_zipped = folder_pars.fdr_zipped;
    }
    
    let mut fdr_unzipped = cli_pars.fu_folder;
    if fdr_unzipped == empty_pb
    {
        fdr_unzipped = folder_pars.fdr_unzipped;
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

    let mut log_folder = folder_pars.log_folder_path;
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



pub fn establish_log(params: &InitParams) -> Result<(), AppError> {

    if !log_set_up() {  // can be called more than once in context of integration tests
        log_helper::setup_log(&params.log_folder_path)?;
        LOG_RUNNING.set(true).unwrap(); // should always work
        log_helper::log_startup_params(&params);
    }
    Ok(())
}

pub fn log_set_up() -> bool {
    match LOG_RUNNING.get() {
        Some(_) => true,
        None => false,
    }
}

// Tests
#[cfg(test)]


mod tests {
    use super::*;
    use std::ffi::OsString;

    // Ensure the parameters are being correctly combined.

    #[test]
    fn check_config_vars_overwrite_blank_cli_values() {

        let config = r#"
[folders]
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
        let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

        let res = get_params(cli_pars, &config_string).unwrap();

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
[folders]
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
        let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

        let res = get_params(cli_pars, &config_string).unwrap();

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
[folders]
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
        let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

        let res = get_params(cli_pars, &config_string).unwrap();

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
[folders]
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
        let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

        let _res = get_params(cli_pars, &config_string).unwrap();
    }


    #[test]
    #[should_panic]
    fn check_f_and_m_panics() {
    
    let config = r#"
[folders]
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
        let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

        let _res = get_params(cli_pars, &config_string).unwrap();
    }

    #[test]
    #[should_panic]
    fn check_f_and_s_panics() {
    
        let config = r#"
    [folders]
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
    
            let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

            let _res = get_params(cli_pars, &config_string).unwrap();
        }


    #[test]
    #[should_panic]
    fn check_no_z_or_u_panics() {
    
    let config = r#"
[folders]
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

        let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

        let _res = get_params(cli_pars, &config_string).unwrap();
    }


    #[test]
    #[should_panic]
    fn check_no_m_s_or_f_panics() {
    
    let config = r#"
[folders]
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
        let cli_pars = cli_reader::fetch_valid_arguments(test_args).unwrap();

        let _res = get_params(cli_pars, &config_string).unwrap();
    }
}

