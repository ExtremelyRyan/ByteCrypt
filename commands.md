# Command-Line Help for `crypt-ui`

This document contains the help content for the `crypt-ui` command-line program.

**Command Overview:**

* [`crypt-ui`](#crypt-ui)
* [`crypt-ui cloud`](#crypt-ui-cloud)
* [`crypt-ui cloud google`](#crypt-ui-cloud-google)
* [`crypt-ui cloud google upload`](#crypt-ui-cloud-google-upload)
* [`crypt-ui cloud google download`](#crypt-ui-cloud-google-download)
* [`crypt-ui cloud google view`](#crypt-ui-cloud-google-view)
* [`crypt-ui cloud dropbox`](#crypt-ui-cloud-dropbox)
* [`crypt-ui cloud dropbox upload`](#crypt-ui-cloud-dropbox-upload)
* [`crypt-ui cloud dropbox download`](#crypt-ui-cloud-dropbox-download)
* [`crypt-ui cloud dropbox view`](#crypt-ui-cloud-dropbox-view)
* [`crypt-ui config`](#crypt-ui-config)
* [`crypt-ui config database-path`](#crypt-ui-config-database-path)
* [`crypt-ui config crypt-path`](#crypt-ui-config-crypt-path)
* [`crypt-ui config ignore-items`](#crypt-ui-config-ignore-items)
* [`crypt-ui config hwid`](#crypt-ui-config-hwid)
* [`crypt-ui config zstd-level`](#crypt-ui-config-zstd-level)
* [`crypt-ui config load-default`](#crypt-ui-config-load-default)
* [`crypt-ui encrypt`](#crypt-ui-encrypt)
* [`crypt-ui decrypt`](#crypt-ui-decrypt)
* [`crypt-ui keeper`](#crypt-ui-keeper)
* [`crypt-ui keeper import`](#crypt-ui-keeper-import)
* [`crypt-ui keeper export`](#crypt-ui-keeper-export)
* [`crypt-ui keeper purge`](#crypt-ui-keeper-purge)
* [`crypt-ui keeper purge token`](#crypt-ui-keeper-purge-token)
* [`crypt-ui keeper purge database`](#crypt-ui-keeper-purge-database)
* [`crypt-ui keeper list`](#crypt-ui-keeper-list)
* [`crypt-ui ls`](#crypt-ui-ls)

## `crypt-ui`

CLI arguments

**Usage:** `crypt-ui [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `cloud` Upload, download, or view file or folder to cloud provider
* `config` View or change configuration
* `encrypt` Encrypt file or folder of files
* `decrypt` Decrypt file or folder of files
* `keeper` Import | Export | Purge database
* `ls` show local / cloud crypt folder

###### **Options:**

* `-d`, `--debug` Enable debug mode
* `--md` generate markdown document for commands
* `-t`

  Default value: `false`



## `crypt-ui cloud`

Upload, download, or view file or folder to cloud provider

**Usage:** `crypt-ui cloud [COMMAND]`

###### **Subcommands:**

* `google` View, upload, or download actions for Google Drive
* `dropbox` View, upload, or download actions for DropBox



## `crypt-ui cloud google`

View, upload, or download actions for Google Drive

**Usage:** `crypt-ui cloud google [COMMAND]`

###### **Subcommands:**

* `upload` Upload a file or folder
* `download` Download a file or folder
* `view` View a file or folder



## `crypt-ui cloud google upload`

Upload a file or folder

**Usage:** `crypt-ui cloud google upload [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` Path to the file to be encrypted and uploaded to the cloud


###### **Options:**

* `-n`, `--no-encrypt` if flag is passed, do not encrypt



## `crypt-ui cloud google download`

Download a file or folder

**Usage:** `crypt-ui cloud google download [PATH]`

###### **Arguments:**

* `<PATH>` name of the file you want to get from the cloud




## `crypt-ui cloud google view`

View a file or folder

**Usage:** `crypt-ui cloud google view [PATH]`

###### **Arguments:**

* `<PATH>`

  Default value: `Crypt`



## `crypt-ui cloud dropbox`

View, upload, or download actions for DropBox

**Usage:** `crypt-ui cloud dropbox [COMMAND]`

###### **Subcommands:**

* `upload` Upload a file or folder
* `download` Download a file or folder
* `view` View a file or folder



## `crypt-ui cloud dropbox upload`

Upload a file or folder

**Usage:** `crypt-ui cloud dropbox upload [OPTIONS] [PATH]`

###### **Arguments:**

* `<PATH>` Path to the file to be encrypted and uploaded to the cloud


###### **Options:**

* `-n`, `--no-encrypt` if flag is passed, do not encrypt



## `crypt-ui cloud dropbox download`

Download a file or folder

**Usage:** `crypt-ui cloud dropbox download [PATH]`

###### **Arguments:**

* `<PATH>` name of the file you want to get from the cloud




## `crypt-ui cloud dropbox view`

View a file or folder

**Usage:** `crypt-ui cloud dropbox view [PATH]`

###### **Arguments:**

* `<PATH>`

  Default value: `Crypt`



## `crypt-ui config`

View or change configuration

**Usage:** `crypt-ui config [COMMAND]`

###### **Subcommands:**

* `database-path` View or update the database path
* `crypt-path` View or update the crypt folder path
* `ignore-items` View or change which directories and/or filetypes are to be ignored
* `hwid` View or change current pc name associated with the cloud
* `zstd-level` View or change the compression level (-7 to 22) -- higher is more compression
* `load-default` Revert config back to default



## `crypt-ui config database-path`

View or update the database path

**Usage:** `crypt-ui config database-path [PATH]`

###### **Arguments:**

* `<PATH>` Database path; if empty, prints current path




## `crypt-ui config crypt-path`

View or update the crypt folder path

**Usage:** `crypt-ui config crypt-path [PATH]`

###### **Arguments:**

* `<PATH>` Database path; if empty, prints current path


## `crypt-ui config ignore-items`

View or change which directories and/or filetypes are to be ignored

**Usage:** `crypt-ui config ignore-items [ADD_REMOVE] [ITEM]`

###### **Arguments:**

* `<ADD_REMOVE>` value to update config

* `<ITEM>` value to update config




## `crypt-ui config hwid`

View or change current pc name associated with the cloud

**Usage:** `crypt-ui config hwid`



## `crypt-ui config zstd-level`

View or change the compression level (-7 to 22) -- higher is more compression

**Usage:** `crypt-ui config zstd-level [LEVEL]`

###### **Arguments:**

* `<LEVEL>` value to update config




## `crypt-ui config load-default`

Revert config back to default

**Usage:** `crypt-ui config load-default`



## `crypt-ui encrypt`

Encrypt file or folder of files

**Usage:** `crypt-ui encrypt [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>` Path to File or Directory

###### **Options:**

* `-o`, `--output <OUTPUT>` Change the output path



## `crypt-ui decrypt`

Decrypt file or folder of files

**Usage:** `crypt-ui decrypt [OPTIONS] <PATH>`

###### **Arguments:**

* `<PATH>` Path to File or Directory

###### **Options:**

* `-p`, `--in-place` Perform an in-place decryption

  Default value: `false`
* `-o`, `--output <OUTPUT>` Change the output path



## `crypt-ui keeper`

Import | Export | Purge database

**Usage:** `crypt-ui keeper [COMMAND]`

###### **Subcommands:**

* `import` View or update the database path
* `export` View or change which directories and/or filetypes are to be ignored
* `purge` PURGES DATABASE FROM SYSTEM
* `list` TODO: maybe get rid of this in the future. for now, handy debugging tool for small db. List each file in the database



## `crypt-ui keeper import`

View or update the database path

**Usage:** `crypt-ui keeper import <PATH>`

###### **Arguments:**

* `<PATH>`




## `crypt-ui keeper export`

View or change which directories and/or filetypes are to be ignored

**Usage:** `crypt-ui keeper export [ALT_PATH]`

###### **Arguments:**

* `<ALT_PATH>` value to update config




## `crypt-ui keeper purge`

PURGES DATABASE FROM SYSTEM

**Usage:** `crypt-ui keeper purge [COMMAND]`

###### **Subcommands:**

* `token` Purges google and Dropbox tokens
* `database` Purges database file and IS UNREVERSABLE!



## `crypt-ui keeper purge token`

Purges google and Dropbox tokens

**Usage:** `crypt-ui keeper purge token`



## `crypt-ui keeper purge database`

Purges database file and IS UNREVERSABLE!

**Usage:** `crypt-ui keeper purge database`



## `crypt-ui keeper list`

TODO: maybe get rid of this in the future. for now, handy debugging tool for small db. List each file in the database

**Usage:** `crypt-ui keeper list`



## `crypt-ui ls`

show local / cloud crypt folder

**Usage:** `crypt-ui ls [OPTIONS]`

###### **Options:**

* `-l`, `--local` Show all files contained in the local crypt folder

  Default value: `false`
* `-c`, `--cloud` Show all files contained in the cloud folder

  Default value: `false`



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>

