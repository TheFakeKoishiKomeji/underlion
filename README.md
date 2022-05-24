# Underlion

A utility to download curseforge modpacks.

## Commands:

### find-bad
Finds mods in the given pack which cannot be downloaded with a normal API key.


Options:  
* `-f`, `--key-file KEY_FILE`  
	Provides an alternate file to pull to CF API key from (default: `.cfkey`)  
* `-k`, `--key`  
	Provides a CF API key (overrides `--key-file`.)  


### install
Installs a curseforge pack from the given pack zip into the given directory.

Usage:  
`install PACK_ZIP [INSTALL_TO]`  
`INSTALL_TO` defaults to a directory with the same name as the zip, minus the .zip extension.

Options:  
* `-f`, `--key-file KEY_FILE`  
	Provides an alternate file to pull to CF API key from (default: `.cfkey`)  
* `-k`, `--key`  
	Provides a CF API key (overrides `--key-file`.)  
* `-p`, `--parallel COUNT`  
	Uses COUNT threats for parallel downloads


### grab-key
Grabs the CF API key from the official curseforge client.

Options:  
* `-u`, `--cf-url URL`  
	Use an alternate URL to download the CF overwolf extension. (Overrides `--cf-version`)  
* `-v`,  `--cf-version VERSION`  
	Use an alternate version of the CF overwolf extension.  

## Support and Updates:
Check out my discord! https://discord.gg/w3EMU2Q2N3