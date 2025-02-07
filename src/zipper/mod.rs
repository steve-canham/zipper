mod helper;

use std::path::PathBuf;
use std::fs;
use crate::error_defs::{AppError, CustomError};
use crate::SourceDetails;


pub fn zip_folder(unzipped_source: &PathBuf, zipped_destination: &PathBuf) -> Result<i32, AppError> {
   
    // check source folder exists, destination can be creted if necessary .

    if !folder_exists(unzipped_source) 
    {
        let msg = "Source folder does not appear to exist - aborting zip";
        let cf_err = CustomError::new(msg);
        return Result::Err(AppError::CsErr(cf_err));
    }
    if !folder_exists(zipped_destination) 
    {
        fs::create_dir_all(zipped_destination)?;
    }
   

   Ok(1)
}

pub fn zip_mdr_folder(_source: SourceDetails, _parent_unzipped_src_fdr: &PathBuf, _parent_zipped_dest_fdr: &PathBuf) -> Result<i32, AppError> {

    // Get details for this source from the database


    Ok(1)
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

    internal void ZipFiles(Options opts)
    {
        _loggingHelper.LogHeader("Setup");
        _loggingHelper.LogCommandLineParameters(opts);

        if (opts.SourceIds is not null && opts.SourceIds.Any())
        {
            // If the zipping is MDR file based there will be a list of source ids.
            // For each set up the folder to receive the zip file(s), then call the
            // relevant routine - using either a single folder of source files or 
            // a folder of source folders, each with a group of xml files

            foreach (int source_id in opts.SourceIds)
            {
                Source s = _dataLayer.FetchSourceParameters(source_id);
                if (s.database_name is not null && s.id != 100159)
                {
                    // Note that the EMA source - 100159 - is zipped from the same folder
                    // as the EUCTR source - it should not therefore be done twice!

                    string unzipped_parent_path = Path.Combine(opts.UnzippedParentFolderPath!, s.database_name);
                    string zipped_parent_path = Path.Combine(opts.ZippedParentFolderPath!, s.database_name);
                    if (Directory.Exists(zipped_parent_path))
                    {
                        string[] filePaths = Directory.GetFiles(zipped_parent_path);
                        foreach (string filePath in filePaths)
                        {
                            File.Delete(filePath);
                        }
                    }
                    else
                    {
                        Directory.CreateDirectory(zipped_parent_path);
                    }

                    if (!Directory.Exists(unzipped_parent_path))
                    {
                        // Can happen if a new source added to source_parameters table
                        // before any files exist to be zipped.

                        Directory.CreateDirectory(unzipped_parent_path);
                    }

                    _loggingHelper.LogLine("Zipping files from " + s.local_folder);
                    int num = (s.local_files_grouped == true)
                        ? _zh.ZipMdrFilesInMultipleFolders(s.database_name, unzipped_parent_path,
                            zipped_parent_path)
                        : _zh.ZipMdrFilesInSingleFolder(s.database_name, unzipped_parent_path, zipped_parent_path);

                    _loggingHelper.LogLine("Zipped " + num + " files from " + s.database_name);
                }
            }
        }
        
        if (opts.UseFolder)
        {
            // If the zipping is folder based (can be any folder) call the routine
            // with the source and destination paths as derived from options.

            _zh.ZipFolder(opts.UnzippedParentFolderPath!, opts.ZippedParentFolderPath!);
        }

        _loggingHelper.CloseLog();
    }
*/