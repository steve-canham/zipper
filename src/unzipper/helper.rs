

/*
internal int UnzipMdrFilesIntoSingleFolder(string zippedPath, string unzipPath)
    {
        string[] source_zip_list = Directory.GetFiles(zippedPath);
        int n = 0;
        if (source_zip_list.Any())
        {
            foreach (string sz in source_zip_list)
            {
                if (sz.ToLower().EndsWith(".zip"))
                {
                    ZipFile.ExtractToDirectory(sz, unzipPath);
                    n++;
                    _loggingHelper.LogLine("Unzipped " + sz);
                }
            }
        }
        return n;
    }


    internal int UnzipMdrFilesIntoMultipleFolders(int? groupingRange, string zippedPath, string unzippedPath)
    {
        int drop_length = 0;
        string folder_suffix = "";

        string[] source_zip_list = Directory.GetFiles(zippedPath);
        int n = 0;

        if (groupingRange is not null)
        {
            // obtain parameters for later bundling operation

            int j = 10;
            folder_suffix = "x";
            while (j != groupingRange)  // grouping range always a power of 10, e.g. 10000
            {
                j *= 10;
                folder_suffix += "x";
            }
            drop_length = folder_suffix.Length + 5;   // additional 5 required for '.json'
        }

        if (source_zip_list.Any())
        {
            foreach (string sz in source_zip_list)
            {
                if (sz.ToLower().EndsWith(".zip"))
                {
                    using (ZipArchive archive = ZipFile.OpenRead(sz))
                    {
                        // extract each file to the parent folder initially.
                        
                        foreach (ZipArchiveEntry entry in archive.Entries)
                        {
                            // Gets the full path to ensure that relative segments are removed.
                            
                            string destinationPath = Path.Combine(unzippedPath, entry.Name);
                            entry.ExtractToFile(destinationPath);
                        }
                    }
                    n++;
                }

                if (groupingRange is not null)
                {
                    // files just unzipped will need bundling up into separate folders....

                    string[] unzipped_list = Directory.GetFiles(unzippedPath);
                    if (unzipped_list.Any())
                    {
                        string full_file_path, folder_path, file_name;
                        foreach (string f in unzipped_list)
                        {
                            full_file_path = f;
                            int last_backslash = full_file_path.LastIndexOf("\\", StringComparison.Ordinal) + 1;
                            file_name = full_file_path[last_backslash..];

                            int file_stem_length = full_file_path.Length - drop_length;
                            folder_path = full_file_path[..file_stem_length] + folder_suffix;
                            if (!Directory.Exists(folder_path))
                            {
                                Directory.CreateDirectory(folder_path);
                            }

                            // move file to folder
                            File.Move(full_file_path, Path.Combine(folder_path, file_name));
                        }
                    }
                }
                _loggingHelper.LogLine("Unzipped " + sz);
            }
        }
        return n;
    }


    internal void UnzipFolder(string sourcePath, string destPath)
    {
        // Source path should not contain sub-folders (if present they will be ignored).
        // Looks for any .zip files and unzip tem if found.
        
        string[] source_file_list = Directory.GetFiles(sourcePath);
        int n = 0;
        if (source_file_list.Any())
        {
            foreach (string f in source_file_list)
            {
                if (f.ToLower().EndsWith(".zip"))
                {
                    ZipFile.ExtractToDirectory(f, destPath);
                    n++;
                }
            }
            _loggingHelper.LogLine($"Unzipped {n} zip files from {sourcePath}");
        }
    }

*/