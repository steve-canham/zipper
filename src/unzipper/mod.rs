mod helper;

use std::path::PathBuf;
use std::fs;
use crate::error_defs::{AppError, CustomError};
use crate::SourceDetails;

pub fn unzip_folder(zipped_source: &PathBuf, unzipped_destination: &PathBuf) -> Result<i32, AppError>{

    // check source folder exists, destination can be creted if necessary.

    if !folder_exists(zipped_source) 
    {
        let msg = "Source folder does not appear to exist - aborting unzip";
        let cf_err = CustomError::new(msg);
        return Result::Err(AppError::CsErr(cf_err));
    }
    if !folder_exists(unzipped_destination) 
    {
        fs::create_dir_all(unzipped_destination)?;
    }

    Ok(1)
}

pub fn unzip_mdr_folder(_source: SourceDetails, _parent_zipped_src_fdr: &PathBuf, _parent_unzipped_dest_fdr: &PathBuf) -> Result<i32, AppError> {

    // both source and destination parent folders already confirmed to exist
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

internal void UnZipFiles(Options opts)
    {
        _loggingHelper.LogHeader("Setup");
        _loggingHelper.LogCommandLineParameters(opts);

        if (opts.SourceIds is not null && opts.SourceIds.Any())
        {
            // If the zipping is MDR file based there will be a list of source ids.
            // For each set up the folder to receive the unzipped file(s), then call the
            // relevant routine - unzipping into either a single folder of source files or 
            // a folder of source folders, each with a group of xml files

            foreach (int source_id in opts.SourceIds)
            {
                Source s = _dataLayer.FetchSourceParameters(source_id);
                if (s.database_name is not null)
                {
                    string zipped_parent_path = Path.Combine(opts.ZippedParentFolderPath!, s.database_name);
                    string unzipped_parent_path = Path.Combine(opts.UnzippedParentFolderPath!, s.database_name);
                    if (Directory.Exists(unzipped_parent_path))
                    {
                        string[] filePaths = Directory.GetFiles(unzipped_parent_path);
                        foreach (string filePath in filePaths)
                        {
                            File.Delete(filePath);
                        }
                    }
                    else
                    {
                        Directory.CreateDirectory(unzipped_parent_path);
                    }

                    _loggingHelper.LogLine("Unzipping files from " + s.database_name);
                    int num = (s.local_files_grouped == true)
                        ? _uzh.UnzipMdrFilesIntoMultipleFolders(s.grouping_range_by_id, zipped_parent_path, unzipped_parent_path)
                        : _uzh.UnzipMdrFilesIntoSingleFolder(zipped_parent_path, unzipped_parent_path);

                    _loggingHelper.LogLine("Unzipped " + num.ToString() + " zip files from " + s.database_name);
                }
            }
        }

        
        if (opts.UseFolder == true)
        {
            // If the unzipping is folder based (can be any folder) call the routine
            // with the source and destination path (if any) derived from options.

            _uzh.UnzipFolder(opts.ZippedParentFolderPath!, opts.UnzippedParentFolderPath!);
        }

        _loggingHelper.CloseLog();
    }

 */