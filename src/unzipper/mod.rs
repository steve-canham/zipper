use std::path::PathBuf;
use std::fs;
use log::info;
use crate::err::AppError;
use crate::SourceDetails;
use std::fs::File;
use zip::ZipArchive;
use zip_extensions::read::zip_extract;

pub fn unzip_folder(zipped_source: &PathBuf, unzipped_destination: &PathBuf) -> Result<usize, AppError>{

    // check source folder exists, destination can be created if necessary.

    if !file_exists(zipped_source) 
    {
        let problem = "There is a problem accessing a designated folder or file".to_string();
        let detail = "Source folder (of zipped files) does not appear to exist.".to_string();
        return Result::Err(AppError::FileSystemError(problem, detail));
    }


    let file = File::open(&zipped_source)?;
    let archive = ZipArchive::new(file)
            .map_err(|e| AppError::UnzipError(e, unzipped_destination.to_owned()))?;

    info!("Unzipping files from {:?} to {:?}", zipped_source, unzipped_destination);

    zip_extract(zipped_source, unzipped_destination)
            .map_err(|e| AppError::UnzipError(e, unzipped_destination.to_owned()))?;

    Ok(archive.len())
}

pub fn unzip_mdr_folder(source: SourceDetails, parent_zipped_src_fdr: &PathBuf, parent_unzipped_dest_fdr: &PathBuf) -> Result<usize, AppError> {

    // both source and destination PARENT folders already confirmed to exist

    let database_name = PathBuf::from(source.database_name);
    if database_name == PathBuf::from("".to_string()) {
        let p = "No database name in Source details".to_string();
        let d = "Unable to unzip correspondig archive".to_string();
        return Err(AppError::FileSystemError(p, d));
    }

    let srce_folder: PathBuf = [parent_zipped_src_fdr, &database_name].iter().collect();
    let dest_folder: PathBuf = [parent_unzipped_dest_fdr, &database_name].iter().collect();

    info!("Unzipping files from {:?} to {:?}", srce_folder, dest_folder);
    
    // get each zip file in the source folder... (each source has one or more zip files in the associated folder)
    // Zip files are arranged in a single list, with no hierarchy of folders within each source's folder.
    // No need to delete existing files in dest folder - they will be over-written if necessary.

    let entries = fs::read_dir(&srce_folder)
                .map_err(|e| AppError::IoReadErrorWithPath(e, srce_folder))?;

    let mut file_num = 0;

    for e in entries {
         let src_path = e?.path();
         if src_path.is_file() {
            match src_path.extension() {
                Some(s) => {
                    if s == PathBuf::from("zip") {
                        file_num += unzip_folder(&src_path, &dest_folder)?;
                        info!("{:?} unzipped. Total files generated so far: {}", src_path, file_num);
                    } 
                    else {
                        continue;
                    }
                },
                None => continue, 
            };
        }
    }
    
    info!("Files generated in total: {}", file_num);
    Ok(file_num)

}


fn file_exists(file_path: &PathBuf) -> bool {
    let xres = file_path.try_exists();
    match xres {
        Ok(true) => true,
        Ok(false) => false, 
        Err(_e) => false,           
    }
}
