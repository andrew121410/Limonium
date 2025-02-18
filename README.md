# Limonium

## Limonium is a tiny Minecraft Server management tool.

### Supported Platforms:
- x86_64-unknown-linux-gnu
- aarch64-unknown-linux-gnu

### Features:
- Can download & update MC Server Software
- Can compile software (Spigot, PlotSquared etc)
- Can backup your Minecraft Server
- Can search in logs for specific text

#### Global Arguments
1. --help `Shows the help menu`
2. --version `Shows the version of Limonium`
3. --self-update `Updates limonium if there is a new version available`
4. --nb `Doesn't show the banner when running the program`

## Download Function
Download function uses platform specific download APIs(Paper, Purpur) or Jenkins(Pufferfish) to download the software.

It will download the software and check the hash of the file to make sure it downloaded correctly, before it moves it to the directory you specified.

**It will overwrite the file if it already exists.**

### Softwares

1. [Paper](https://github.com/PaperMC/Paper) -> `./limonium download paper 1.21.4`
2. [Purpur](https://github.com/PurpurMC/Purpur) -> `./limonium download purpur 1.21.4`
3. [Pufferfish](https://github.com/pufferfish-gg/Pufferfish) -> `./limonium download pufferfish 1.21.4`
4. [Geyser](https://github.com/GeyserMc/Geyser) -> `./limonium download geyser 2.1.0` (default is geyser-standalone)

### Proxies

1. [Velocity](https://github.com/PaperMC/Velocity) -> `./limonium download velocity 3.2.0-SNAPSHOT`
2. [BungeeCord](https://github.com/SpigotMC/BungeeCord) -> `./limonium download bungeecord latest`

### Plugins
1. [Floodgate](https://github.com/GeyserMC/Floodgate) -> `./limonium download floodgate latest`
2. [ViaVersion](https://github.com/ViaVersion/ViaVersion) -> `./limonium download viaversion latest` (Available channels: dev, compatibility)
3. [ViaBackwards](https://github.com/ViaVersion/ViaBackwards) -> `./limonium download viabackwards latest` (Available channels: dev, compatibility)
4. [Citizens2](https://github.com/CitizensDev/Citizens2) -> `./limonium download citizens2 latest`

### Important

Limonium is not affiliated with any of the projects listed.

### Examples

### Optional Download Arguments
1. --o `The path of where the jar should go Example: --o /mc-servers/hub/Paper.jar`
2. --c `The channel so for geyser the default channel is "standalone" but can be changed to (spigot, bungeecord, velocity, fabric, sponge)`
3. --latest-use-at-your-own-risk `(Warning: Don't use this is bad (you don't want your Minecraft Server randomly getting upgraded to a new Minecraft version, without you knowing)) Using this argument with the latest version, It will find the latest version of the software for you (really used for something like Geyser or Velocity)`
4. --no-snapshot-version `When searching for the latest version, it will not include snapshot versions`
5. --run-jvmdowngrader `Runs JvmDowngrader to downgrade the JAR file to a Java Version Example: --run-jvmdowngrader 52 (Java 8) --run-jvmdowngrader 60 (Java 16) ETC` https://github.com/unimined/JvmDowngrader
### Download Usage

*Usage: &lt;software&gt; &lt;version&gt;*

```
./limonium download paper 1.21.4
```

```
./limonium download paper 1.21.4 --o Paper.jar
```

```
./limonium download paper 1.21.4 --o ./mc-servers/hub/Paper.jar
```

## Compile Function

The compile function will compile the software you specify, and put it in the directory you specify.

All the software will be downloaded & compiled in the ./limonium-compile directory you can delete at any time.

### Softwares

1. [Spigot](https://hub.spigotmc.org/stash/projects/SPIGOT/repos/spigot/browse) -> `./limonium compile spigot server.jar --version 1.21.4` (Not recommended to use)
2. [PlotSquared](https://github.com/IntellectualSites/PlotSquared) -> `./limonium compile plotsquared PlotSquared.jar`
3. [mcMMO](https://github.com/mcMMO-Dev/mcMMO) -> `./limonium compile mcmmo mcMMO.jar`
4. [CoreProtect](https://github.com/PlayPro/CoreProtect) -> `./limonium compile coreprotect CoreProtect.jar --version 23.2`

### Examples

### Optional Compile Arguments
1. --branch `The branch to use (If you don't specify a branch, it will use the default branch)`
2. --version `The version to use (used only for Spigot for now)`

### Compile Usage

*Usage: &lt;software&gt; &lt;output&gt;*

```
./limonium compile spigot server.jar --version 1.21.4
```

``` 
./limonium compile plotsquared PlotSquared.jar
```

## Backup Function

The backup function will back up the folders you specify, and compress them and put them in the backup directory.
By default, it will use tar.gz, unless specified otherwise.

### Optional Backup Arguments
1. --format `The format to use (tar.gz, tar.zst, zip)`
2. --level `The compression level to use (tar.gz 0-9) (tar.zst 1-22) (zip 0-9)`
3. --exclude `Excludes files from the backup`
4. --sftp `Uploads the backup to a SFTP server. Example 1: --sftp user@host:22 /remote/path Example 2: --sftp "user@host:22 path/to/key /remote/path"` (**Password Authentication is not supported.**)
5. --delete-after-upload `Deletes the local backup after uploading it to the SFTP server.`
6. --ask-before-uploading `Asks before uploading the backup to the SFTP server.`
7. --local-delete-after-time `Deletes backups locally after a certain amount of time. Example: --local-delete-after-time 1m (1 month) --local-delete-after-time 1w (1 week) --local-delete-after-time 1d (1 day)`
8. --local-always-keep `Always keep a certain number of backups locally when using --local-delete-after-time.`
9. --remote-delete-after-time `Deletes backups remotely after a certain amount of time. Example: --remote-delete-after-time 1m (1 month) --remote-delete-after-time 1w (1 week) --remote-delete-after-time 1d (1 day)`

Debug Arguments
1. --verbose `Shows more information`
2. --I `Overides the -I argument for tar. Example: --I "zstd -T0 -19 -v""`

### Backup Usage

*Usage: &lt;name&gt; &lt;folder/s&gt; &lt;backup_directory&gt;*

```
./limonium backup survival . ../survival-backups/
```
```
./limonium backup survival world ../survival-backups/
```
```
./limonium backup survival world:world_nether:world_the_end:plugins ../survival-backups/ --exclude plugins/dynmap/
```

## Log Function

The log function will search the logs for the text you specify, and will open up nano with the results.

### Optional Log Arguments
1. --path `The path to the logs.` (default is ./logs/)

### Log Usage

*Usage: &lt;days-back&gt; &lt;to-search&gt; &lt;lines-before&gt; &lt;lines-after&gt;*

```
./limonium log 10 "andrew121410"
```
```
./limonium log 10 "andrew121410" --path /mc-servers/hub/logs/
```
```
./limonium log 10 "andrew121410" 5 6
```
```
./limonium log 10 "andrew121410" 5 6 --path /mc-servers/hub/logs/
```

The above examples will search the logs for "andrew121410" in the last 10 days.
The 5 and 6 are the lines before and after the text you are searching for. So it will show 5 lines before and 6 lines after. So you will be able to see more context.

## Building

`cargo build -r`
