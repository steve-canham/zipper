/***************************************************************************
 * Module uses clap crate to read command line arguments. These include 
 * possible A, S, T and C flags, and possible strings for the data folder and 
 * source file name. If no flags 'S' (= import data) is returned by default.
 * Folder and file names return an empty string ("") rather than null if not 
 * present. 
 ***************************************************************************/

use clap::{command, Arg, ArgMatches};
use crate::error_defs::{CustomError, AppError};
use crate::setup::{CliPars, Flags};
use std::ffi::OsString;
use std::path::PathBuf;


pub fn fetch_valid_arguments(args: Vec<OsString>) -> Result<CliPars, AppError>
{ 
    let parse_result = parse_args(args)?;

    // These parameters guaranteed to unwrap OK as all have a default value of "".

    let source_list = parse_result.get_one::<String>("source_list").unwrap();
    let fz_folder = parse_result.get_one::<String>("fz_folder").unwrap();
    let fu_folder = parse_result.get_one::<String>("fu_folder").unwrap();

    // Flag values are false if not present, true if present.

    let z_flag = parse_result.get_flag("z_flag");
    let u_flag = parse_result.get_flag("u_flag");

    let m_flag = parse_result.get_flag("m_flag");

    let f_flag = parse_result.get_flag("f_flag");
    let t_flag = parse_result.get_flag("t_flag");

    if z_flag && u_flag {   // both set do nothing and report as error
        let msg = "Both zip and unzip requested at the same time! Unable to proceed.";
        let cf_err = CustomError::new(msg);
        return Result::Err(AppError::CsErr(cf_err));
    }

    let flags = Flags {
        do_zip: z_flag,
        do_unzip: u_flag,
        all_mdr: m_flag,
        use_folder: f_flag,
        test_run: t_flag,
        };

    Ok(CliPars {
        source_list: source_list.clone(),
        fz_folder: PathBuf::from(fz_folder.clone()),
        fu_folder: PathBuf::from(fu_folder.clone()),
        flags: flags,
    })
}


fn parse_args(args: Vec<OsString>) -> Result<ArgMatches, clap::Error> {

    command!()
        .about("Imports data from ROR json file (v2) and imports it into a database")
        .arg(
            Arg::new("source_list")
           .short('s')
           .long("sources")
           .required(false)
           .help("A string with a list of integer ids of the mdr sources")
           .default_value("")
         )
        .arg(
             Arg::new("fz_folder")
            .long("fz")
            .required(false)
            .help("A string with the folder path of the zipped folder")
            .default_value("")
        )
        .arg(
            Arg::new("fu_folder")
           .long("fu")
           .required(false)
           .help("A string with the folder path of the unzipped folder")
           .default_value("")
         )
         .arg(
            Arg::new("z_flag")
           .short('z')
           .long("zip")
           .required(false)
           .help("A flag signifying perform a zip on the designated folders")
           .action(clap::ArgAction::SetTrue)
         )
        .arg(
            Arg::new("u_flag")
           .short('u')
           .long("unzip")
           .required(false)
           .help("A flag signifying perform an unzip on the designated folders")
           .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("m_flag")
            .short('m')
            .long("mdr")
            .required(false)
            .help("A flag signifying that the default mdr folders should be used, for all mdr files")
            .action(clap::ArgAction::SetTrue)
       )
        .arg(
             Arg::new("f_flag")
            .short('f')
            .long("folder")
            .required(false)
            .help("A flag signifying use the -fz, -fu paths for zipped and unzipped files, not the mdr defaults")
            .action(clap::ArgAction::SetTrue)
        )
       .arg(
            Arg::new("t_flag")
            .short('t')
            .long("test")
            .required(false)
            .help("A flag signifying that this is part of an integration test run - suppresses logs")
            .action(clap::ArgAction::SetTrue)
       )
    .try_get_matches_from(args)

}


#[cfg(test)]
mod tests {
    use super::*;

    // Ensure the parameters are being correctly extracted from the CLI arguments

    #[test]
    fn check_cli_no_explicit_params() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();
        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_list, "");
        assert_eq!(res.fz_folder, PathBuf::new());
        assert_eq!(res.fu_folder, PathBuf::new());
        assert_eq!(res.flags.do_zip, false);
        assert_eq!(res.flags.do_unzip, false);
        assert_eq!(res.flags.all_mdr, false);
        assert_eq!(res.flags.use_folder, false);
        assert_eq!(res.flags.test_run, false);

    }
  
    #[test]
    fn check_cli_with_m_and_z_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-z", "-m"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_list, "");
        assert_eq!(res.fz_folder, PathBuf::new());
        assert_eq!(res.fu_folder, PathBuf::new());
        assert_eq!(res.flags.do_zip, true);
        assert_eq!(res.flags.do_unzip, false);
        assert_eq!(res.flags.all_mdr, true);
        assert_eq!(res.flags.use_folder, false);
        assert_eq!(res.flags.test_run, false);

    }

    #[test]
    fn check_cli_with_u_and_f_flag() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-u", "-f"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_list, "");
        assert_eq!(res.fz_folder, PathBuf::new());
        assert_eq!(res.fu_folder, PathBuf::new());
        assert_eq!(res.flags.do_zip, false);
        assert_eq!(res.flags.do_unzip, true);
        assert_eq!(res.flags.all_mdr, false);
        assert_eq!(res.flags.use_folder, true);
        assert_eq!(res.flags.test_run, false);

    }

    #[test]
    fn check_cli_with_z_flag_and_s_list() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-z", "-s", "100101, 100102, 100103, 100104"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_list, "100101, 100102, 100103, 100104");
        assert_eq!(res.fz_folder, PathBuf::new());
        assert_eq!(res.fu_folder, PathBuf::new());
        assert_eq!(res.flags.do_zip, true);
        assert_eq!(res.flags.do_unzip, false);
        assert_eq!(res.flags.all_mdr, false);
        assert_eq!(res.flags.use_folder, false);
        assert_eq!(res.flags.test_run, false);
    }


    #[test]
    fn check_cli_with_uf_flag_and_fzfu_folders() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-u", "-f", "--fz", "data\\zipped folder", "--fu", "data\\unzipped folder"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_list, "");
        assert_eq!(res.fz_folder, PathBuf::from("data\\zipped folder"));
        assert_eq!(res.fu_folder, PathBuf::from("data\\unzipped folder"));
        assert_eq!(res.flags.do_zip, false);
        assert_eq!(res.flags.do_unzip, true);
        assert_eq!(res.flags.all_mdr, false);
        assert_eq!(res.flags.use_folder, true);
        assert_eq!(res.flags.test_run, false);
    }

    #[test]
    #[should_panic]
    fn should_panic_with_z_and_u_flags() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-z", "-u"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let _res = fetch_valid_arguments(test_args).unwrap();
    }


    #[test]
    fn check_cli_with_z_m_and_t_flags() {
        let target = "dummy target";
        let args : Vec<&str> = vec![target, "-m", "-t", "-z"];
        let test_args = args.iter().map(|x| x.to_string().into()).collect::<Vec<OsString>>();

        let res = fetch_valid_arguments(test_args).unwrap();
        assert_eq!(res.source_list, "");
        assert_eq!(res.fz_folder, PathBuf::new());
        assert_eq!(res.fu_folder, PathBuf::new());
        assert_eq!(res.flags.do_zip, true);
        assert_eq!(res.flags.do_unzip, false);
        assert_eq!(res.flags.all_mdr, true);
        assert_eq!(res.flags.use_folder, false);
        assert_eq!(res.flags.test_run, true);
    }

}


