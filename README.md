<h2>Introduction</h2>
A conversion of an old zipping / unzipping tool originally written in C#.
Mostly to see how scheduled zipping / unzipping can be supported in a Rust CLI program.

As the main functionality is to zip / unzip the json files held in the various MDR source data folders, 
one common set of commands involves selecting all or some of those data folders for zipping or unzipping. 

In addition, a specified folder can also be zipped or unzipped to or from a specified folder.

Note that the MDR files are stored as either a single folder of files, for most sources, or as a collection of folders, for a few sources.
Zipping and unzipping these files assumes and takes advantage of this relatively simple structure, with no or a single level of folder hierarchy respectively.

<h2>Configuration</h2>
An app_config.toml file must be present, in the same folder as cargo.toml. It should have theb structure below:<br/>
<br/>
[folders]<br/>
mdr_zipped=""<br/>
mdr_unzipped=""<br/>
<br/>
fdr_zipped=""<br/>
fdr_unzipped=""<br/>
<br/>
log_folder_path=""<br/>
<br/>
[database]<br/>
db_host=""<br/>
db_user=""<br/>
db_password=""<br/>
db_port=""<br/>
db_name=""<br/>
<br/>
with the relevant values inserted between the double quotes. Folder paths can be inserted with posix forward slashes or with
doubled back slashes as path separaters. 

Note that mdr_unzipped / mdr_zipped refer to the <i>parent</i> folder of the MDR json files / ziupped json files respectively. 
Within those parent folders there are separate folders for each source (normally a trial registry). They must be present if the -m or -s flag is used (see below).

The fdr_zipped folder path should reference an individual archive <i>file</i>, not a folder. Conversely the fdr_unzipped path shpould be to a source or destination <i>folder</i>. Both are required if the -f flag is used (see below). They can also be provided as part of the CLI arguments, under the --fz and --fu switches. Zipping / unzipping takes place recursively down the folder tree. 

If a directory is zipped to a single folder and that archive file already exists it will be over-written. If it does not exist it will be created.<br/>
If an archive is unzipped to a folder then existing files of the same name will be over-written (other files are left alone). If the folder does not exist it will be created.

<h2>Flags</h2>

<ul>
<li> -z: A flag signifying perform a zip on the designated folder(s).</li>
<li> -u: A flag signifying perform an unzip on the designated folder(s). </li> 
<li> -m: A flag signifying that the -z or -u should be applied to <i>all</i> MDR data, using the default mdr folders in the configuration files</li>
<li> -s (followed by a string of comma separated integer source ids): Signifies that the -z or -u should be applied to data from the designated MDR sources only.</li>
<li> -f: A flag signifying use the -fz, -fu paths for zipped and unzipped files, or the fdr paths in the confi file, not the mdr defaults</li>
<li> --fz: The full path of the zipped archive file. If provided overwrites any configuration file value.</li>
<li> --fu: The folder path of the unzipped folder. If provided overwrites the configuration file value.</li>
</ul>
Again, folder paths can be inserted with posix forward slashes or with doubled back slashes as path separaters. 

Note:<br/>
The program will stop reporting an error if any of the following situations occur.
<ul>
<li>Neither -z or -u are used as flags (one must always be present)</li>
<li>Both -z and -u are used.</li>
<li>None of -m, -s, or -f are specified (one must always be present).</li>
<li>-f is specified at the same time as -m or -s.</li>
<li>-m or -s is specified without values present for the MDR parent folders.</li>
<li>-f is specified without values present for the zipped / unzipped folders, either in the config file or in the command line arguments.</li>
</ul>