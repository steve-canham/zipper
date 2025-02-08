use std::path::PathBuf;
//use std::fs;
use crate::error_defs::AppError;
//use crate::SourceDetails;
use chrono::Local;


fn zip_mdr_files_in_single_folder() -> Result<(), AppError> {

   /*
   string[] file_list = Directory.GetFiles(sourceFolderPath);
        return file_list.Length > 0 
            ? ZipFiles(file_list, zipFolderPath, databaseName) 
            : 0;    // if no files in the folder
   
    */

   Ok(())

}


fn zip_files(file_list: Vec<String>, zip_folder_path: &PathBuf, file_name_stem: String) -> Result<usize, AppError> {

    // file_list is the list of full paths for each file in the folder
    // zip_folder_path is the full path of the folder in which the zip files are to be stored
    // file_name_stem name is used as the beginning of the zip file name

    let files_per_zip = 10000;

    let file_num = file_list.len();
    let mut zip_files_needed = file_num / files_per_zip;
    if file_num % files_per_zip != 0 {
        zip_files_needed += 1;  // usually the case!
    } 

    for i in 0..zip_files_needed {

        let start_file_num = i * files_per_zip;
        let start_file = (start_file_num + 1).to_string();
        let mut end_file_num = (i + 1) * files_per_zip;
        if end_file_num >= file_num {
            end_file_num = file_num;
        }
        let end_file = (end_file_num).to_string();

        // Establish Zip file name and full path, and then Zip the relevant
        // source files into it. The 'entry_name' is the file name, rather than
        // the full path. Both are required as inputs to the zipping call.

        let today = Local::now().format("%m-%d %H%M%S").to_string();
        let zip_file_name = format!("{} {} {} to {}.zip", file_name_stem, today, start_file, end_file);
        let _zip_file_path: PathBuf = [zip_folder_path, &PathBuf::from(zip_file_name)].iter().collect(); 

        /* 
        using (ZipArchive zip = ZipFile.Open(zip_file_path, ZipArchiveMode.Create))
        {
            for (int i = start_file_num; i < end_file_num; i++)
            {
                string source_file_path = fileList[i];
                int last_backslash = source_file_path.LastIndexOf("\\", StringComparison.Ordinal) + 1;
                string entry_name = source_file_path[last_backslash..];
                zip.CreateEntryFromFile(source_file_path, entry_name);
            }
        }

        _loggingHelper.LogLine("Zipped " + zip_file_path);
    }
    */

    }
    
    Ok(file_num)
 
 }
 

/*



    internal int ZipMdrFilesInMultipleFolders(string databaseName, string sourceFolderPath, string zipFolderPath)
    {
        int file_num = 0;
        string[] folder_list = Directory.GetDirectories(sourceFolderPath);
        int folder_num = folder_list.Length;      // total folders in source directory
        if (folder_num == 0)
        {
            return 0;  // empty folder list - should not happen but could with a brand new source with no files
        }
        
        // produce a zip for each group of folders, checking that the max size has
        // not been exceeded after each folder.
               
        long max_zip_size = 18 * 1024 * 1024;     // 18 MB set as max zip size in this context

        string source_folder, source_file_path, folder_name, entry_name;
        int folder_backslash, file_backslash;
        string last_used_folder_name = "";

        int k = -1;                               // k is the index of the source folders in the source directory
        
        while (k < folder_num)
        {
            k++;      // Increments at start and each time returns to the outer loop

            // If the very last folder caused the file size to be exceeded
            // k now equals hew total folder number and the process has completed.
            // There is therefore no need for an additional zip file

            if (k == folder_num) break;

            // This code run at the beginning and each time inner loop is exited
            // need to create zip file path using the first folder in this 'batch'.

            bool new_zip_required = false;         // will be set to true if current file size greater than max size
  
            source_folder = folder_list[k];
            folder_backslash = source_folder.LastIndexOf("\\", StringComparison.Ordinal) + 1;
            string first_folder = source_folder[folder_backslash..];

            // While the file is being constructed the file name is the provisional one below.

            string zip_file_path = Path.Combine(zipFolderPath, databaseName + " " +
                                    _today + " " + first_folder + " onwards.zip");

            // Add this and following folder's files to the archive, as long as it stays within the size limit
            // initially k value is the same as in the outer loop, but will increase until max size exceeded

            using (ZipArchive zip = ZipFile.Open(zip_file_path, ZipArchiveMode.Create))
            {
                while (k < folder_num && !new_zip_required)
                {
                    source_folder = folder_list[k];
                    folder_backslash = source_folder.LastIndexOf("\\", StringComparison.Ordinal) + 1;
                    folder_name = source_folder[folder_backslash..];
                    last_used_folder_name = folder_name;

                    string[] file_list = Directory.GetFiles(source_folder);
                    {
                        foreach (string f in file_list)
                        {
                            source_file_path = f;
                            file_backslash = source_file_path.LastIndexOf("\\", StringComparison.Ordinal) + 1;
                            entry_name = source_file_path[folder_backslash..];   // includes folder and file
                            zip.CreateEntryFromFile(source_file_path, entry_name);
                        }
                    }

                    file_num += file_list.Length;
                    _loggingHelper.LogLine("Zipped " + folder_name);
                    long zip_file_size = new FileInfo(zip_file_path).Length;        // Used for current length of zip file    

                    // Is a new zip file required? If not get the next folder and repeat the zipping process.
                    // If yes, the inner while condition becomes false and control returns to the outer loop.

                    new_zip_required = zip_file_size > max_zip_size;
                    if (!new_zip_required)
                    {
                        k++;
                    }
                }
            }

            // Rename the zip file that has just been completed.

            string final_zip_name = Path.Combine(zipFolderPath, databaseName + " " +
                                         _today + " " + first_folder + " to " + last_used_folder_name + ".zip");
            File.Move(zip_file_path, final_zip_name);

        }

        return file_num;
    }


    internal void ZipFolder(string sourcePath, string destPath)
    {
        // Source path should not contain sub-folders. (They will be ignored).

        string[]? source_file_list = Directory.GetFiles(sourcePath);

        if (source_file_list?.Length > 0)
        {
            // zip file name starts with the source path after the drive letter, with back slashes replaced.

            string file_name_stem = sourcePath[3..].Replace("\\", "-");
            ZipFiles(source_file_list, destPath, file_name_stem);
        }
    }


    private int ZipFiles(string[] fileList, string zipFolderPath, string fileNameStem = "")
    {
        // file_list is the list of full paths for each file in the folder
        // zip_folder_path is the full path of the folder in which the zip files are to be stored
        // file_name_stem name is used as the beginning of the zip file name

        int file_num = fileList.Length;
        int zip_files_needed = (file_num % _filesPerZip == 0)
                                   ? file_num / _filesPerZip
                                   : (file_num / _filesPerZip) + 1;

        for ( int j = 0; j < zip_files_needed; j++)
        {
            // Get the start and end position in the file list for this pass,
            // and string equivalents for the zip file title.

            int start_file_num = (j * _filesPerZip);
            string start_file = (start_file_num + 1).ToString();
            int end_file_num;
            if ((j + 1) * _filesPerZip >= file_num)
            {
                end_file_num = file_num;
            }
            else
            {
                end_file_num = (j * _filesPerZip) + _filesPerZip;
            }
            string end_file = (end_file_num).ToString();

            // Establish Zip file title and full path, and then Zip the relevant
            // source files into it. The 'entry_name' is the file name, rather than
            // the full path. Both are required as inputs to the zipping call.

            string zip_file_name = fileNameStem + " " + _today + " "
                                        + start_file + " to " + end_file;
            string zip_file_path = Path.Combine(zipFolderPath, zip_file_name + ".zip");

            using (ZipArchive zip = ZipFile.Open(zip_file_path, ZipArchiveMode.Create))
            {
                for (int i = start_file_num; i < end_file_num; i++)
                {
                    string source_file_path = fileList[i];
                    int last_backslash = source_file_path.LastIndexOf("\\", StringComparison.Ordinal) + 1;
                    string entry_name = source_file_path[last_backslash..];
                    zip.CreateEntryFromFile(source_file_path, entry_name);
                }
            }

            _loggingHelper.LogLine("Zipped " + zip_file_path);
        }

        return file_num;
    }

*/