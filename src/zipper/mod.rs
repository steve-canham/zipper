mod helper;

use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use crate::err::AppError;
use crate::SourceDetails;
use zip_extensions::write::zip_create_from_directory_with_options;
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;
use zip::ZipWriter;
use log::info;
use std::fs;
use std::fs::File;
use chrono::Local;
use std::io::copy;

pub fn zip_folder(unzipped_source_folder: &PathBuf, zipped_destination_file: &PathBuf) -> Result<(), AppError> {
   
    // Used with -f. Zips all of the folder. 
    // Check source folder exists, destination zip file can be created if necessary .

    if !folder_exists(unzipped_source_folder) 
    {
        let problem = "There is a problem accessing a designated folder or file".to_string();
        let detail = "Source folder (of unzipped files) does not appear to exist.".to_string();
        return Result::Err(AppError::FileSystemError(problem, detail));
    }

    info!("Zipping files from {:?} to {:?}", unzipped_source_folder, zipped_destination_file);

    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip_create_from_directory_with_options(zipped_destination_file, unzipped_source_folder, |_| options)
                .map_err(|e| AppError::ZipError(e, zipped_destination_file.to_owned()))?;
    
    Ok(())
}


pub fn zip_mdr_folder(source: SourceDetails, parent_unzipped_src_fdr: &PathBuf, parent_zipped_dest_fdr: &PathBuf) -> Result<usize, AppError> {

    // Used with -s or -m. Both source and destination PARENT folders already confirmed to exist

    let database_name = source.database_name;
    if database_name.trim() == "".to_string() {
        let p = "No database name in Source details".to_string();
        let d = "Unable to unzip correspondig archive".to_string();
        return Err(AppError::FileSystemError(p, d));
    }

    let srce_folder: PathBuf = [parent_unzipped_src_fdr, &PathBuf::from(&database_name)].iter().collect();
    let dest_folder: PathBuf = [parent_zipped_dest_fdr, &PathBuf::from(&database_name)].iter().collect();

    info!("Zipping files from {:?} to {:?}", srce_folder, dest_folder);

    if source.local_files_grouped {
        zip_mdr_files_in_multiple_folders(&database_name, &srce_folder, &dest_folder)
    }
    else {
        zip_mdr_files_in_single_folder(&database_name, &srce_folder, &dest_folder)
    }
}


fn zip_mdr_files_in_single_folder(database_name: &String, srce_folder: &PathBuf, dest_folder: &PathBuf) -> Result<usize, AppError> {

    let file_list = fs::read_dir(srce_folder)
        .map_err(|e| AppError::IoReadErrorWithPath(e, srce_folder.to_owned()))?;

    let paths: Vec<PathBuf> = file_list
        .filter_map(|entry| Some(entry.ok()?.path()))
        .collect();

    let file_num = paths.len();   
    if file_num == 0 {
        return Ok(0);
    }

    let today = Local::now().format("%y%m%d").to_string();
    let file_name_stem = format!("{} {} ", database_name, today);
    let files_per_zip = 10000;
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let mut i = 0;
    let mut j = 0;
    
    // initialise first zip file

    let start_file ="1";
    let mut end_file_num = files_per_zip;
    if end_file_num >= file_num {
        end_file_num = file_num;
    }
    let end_file = end_file_num.to_string();

    let mut zip_file_name = format!("{} {} to {}.zip", file_name_stem, start_file, end_file);
    let mut zip_file_path: PathBuf = [dest_folder, &PathBuf::from(zip_file_name)].iter().collect(); 
    let mut zip_file = File::create(&zip_file_path)?;
    let mut zip = ZipWriter::new(zip_file); 

    for p in paths {

        // New zip file required?

        if j == 0 && i != 0 {
            
            zip.finish()        // complete previous zip
                .map_err(|e| AppError::ZipError(e, zip_file_path.to_owned()))?;
            info!("{:?} archive created from {} files", zip_file_path, files_per_zip);
       
            let start_file = (i + 1).to_string();
            let mut end_file_num = i + files_per_zip;
            if end_file_num >= file_num {
                end_file_num = file_num;
            }
            let end_file = end_file_num.to_string();
        
            zip_file_name = format!("{} {} to {}.zip", file_name_stem, start_file, end_file);
            zip_file_path = [dest_folder, &PathBuf::from(zip_file_name)].iter().collect(); 
            zip_file = File::create(&zip_file_path)?;
            zip = ZipWriter::new(zip_file); 
        }

        let file = File::open(&p)?;
        let file_name = match p.file_name() {
                Some(oss) => { 
                    match oss.to_str() {
                    Some(filename) => filename,
                    None => return Err(AppError::FileSystemError("Error when extracting file name from path".to_string(), 
                            "Could not turn OsStr to string".to_string())),
                        }
                },
                None => return Err(AppError::FileSystemError("Error when extracting file name from path".to_string(), 
                         "Could not read file name as OsStr".to_string()))
            };

            
        // Adding the file to the ZIP archive.

        zip.start_file(file_name, options)
                .map_err(|e| AppError::ZipError(e, p.to_owned()))?;

        let mut buffer = Vec::new();
        copy(&mut file.take(u64::MAX), &mut buffer)?;
        
        zip.write_all(&buffer)?;

        i += 1;
        j += 1;
        if j == files_per_zip {
            j = 0;
        }
    }

    zip.finish()
        .map_err(|e| AppError::ZipError(e, zip_file_path.to_owned()))?;

    info!("{} files zipped in total", i);

    
    Ok(file_num)
 
 }


 fn zip_mdr_files_in_multiple_folders(database_name: &String, srce_folder: &PathBuf, dest_folder: &PathBuf) -> Result<usize, AppError> {

    let folder_list = fs::read_dir(srce_folder)
        .map_err(|e| AppError::IoReadErrorWithPath(e, srce_folder.to_owned()))?;

    let folders: Vec<PathBuf> = folder_list
        .filter_map(|entry| Some(entry.ok()?.path()))
        .collect();

    let folder_num = folders.len();   
    if folder_num == 0 {
        return Ok(0);
    }
   
    let today = Local::now().format("%y%m%d").to_string();
    let file_name_stem = format!("{} {} ", database_name, today);
    let min_files_per_zip = 10000;
    //let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    //let mut source_folder= "".to_string();
    //let mut source_file_path= "".to_string();
    //let mut folder_name= "".to_string();
    //let mut entry_name= "".to_string();
    //let mut last_used_folder_name = "".to_string();

    // Produce a zip for each group of folders, checking that the max size has
    // not been exceeded after each folder.

    let mut i = 0;  // accumulative total of files zipped, overall
    let mut j = 0;  // accumulative total of files zipped in the current zip file
    
    // Set up initial zip file
    let start_file = (i + 1).to_string();
    let end_file = "...".to_string();

    let mut zip_file_name = format!("{} {} to {}.zip", file_name_stem, start_file, end_file);
    let mut zip_file_path: PathBuf = [dest_folder, &PathBuf::from(zip_file_name)].iter().collect(); 
    let mut zip_file = File::create(&zip_file_path)?;
    let mut zip = ZipWriter::new(zip_file); 

    for f in folders {

        let _folder_name = match f.file_name() {
            Some(oss) => { 
                match oss.to_str() {
                Some(filename) => filename,
                None => return Err(AppError::FileSystemError("Error when extracting folder name from path".to_string(), 
                        "Could not turn OsStr to string".to_string())),
                    }
            },
            None => return Err(AppError::FileSystemError("Error when extracting folder name from path".to_string(), 
                     "Could not read file name as OsStr".to_string()))
        };


        if j == 0 && i != 0 {

            //***** Finish off old zip - restart with new ***************
            // rename last zip to reflect last folder added

            zip.finish()        // complete previous zip
                .map_err(|e| AppError::ZipError(e, zip_file_path.to_owned()))?;

            //info!("{:?} archive created from {} files", zip_file_path, files_per_zip);
        
            // create next zip file

            let start_file = (i + 1).to_string();
            let end_file = "...".to_string();
        
            zip_file_name = format!("{} {} to {}.zip", file_name_stem, start_file, end_file);
            zip_file_path = [dest_folder, &PathBuf::from(zip_file_name)].iter().collect(); 
            zip_file = File::create(&zip_file_path)?;
            zip = ZipWriter::new(zip_file); 
        }

        // add the folder to the zip archive
              
        //********************************************************* 
        // zip the contents of this folder to the current zip file
        //*********************************************************

        let number_in_last_folder = 10;  // from adding the files of the last folder to the current zip

        i += number_in_last_folder;
        j += number_in_last_folder;
        if j >= min_files_per_zip {
            j = 0;
        }
    }

    Ok(folder_num)
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

