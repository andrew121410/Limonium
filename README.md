# Limonium

## Limonium is a tiny Minecraft Server management tool.

Features:
- Can download Minecraft Server .jars (I use it for updating the server jars)
- Can backup your Minecraft Server
- Can search in logs for specific text

Limonium uses platform specific download APIs(Paper, Purpur) or Jenkins to download the .jars

### Global Arguments
1. --help `Shows the help menu`
2. --version `Shows the version of Limonium`
3. --self-update `Updates limonium if there is a new version available`

## Download Function

### Softwares

1. [Paper](https://github.com/PaperMC/Paper) -> `./limonium downlaod paper 1.20.1`
2. [Purpur](https://github.com/PurpurMC/Purpur) -> `./limonium downlaod purpur 1.20.1`
3. [Pufferfish](https://github.com/pufferfish-gg/Pufferfish) -> `./limonium downlaod pufferfish 1.20.1`
4. [Geyser](https://github.com/GeyserMc/Geyser) -> `./limonium downlaod geyser 2.1.0` (default is geyser-standalone)
6. [Spigot](https://hub.spigotmc.org/stash/projects/SPIGOT/repos/spigot/browse) -> `./limonium download spigot 1.20.1` (Not recommended)\
__If you choose Spigot then it will install BuildTools.jar to ./lmtmp/ then run it__

### Proxies

1. [Waterfall](https://github.com/PaperMC/Waterfall) -> `./limonium download waterfall 1.19`
2. [Velocity](https://github.com/PaperMC/Velocity) -> `./limonium download velocity 3.2.0-SNAPSHOT`

### Plugins
1. [ViaVersion](https://github.com/ViaVersion/ViaVersion) -> `./limonium download viaversion doesntmatter`
2. [ViaBackwards](https://github.com/ViaVersion/ViaBackwards) -> `./limonium download viabackwards doesntmatter`

### Important

Limonium is not affiliated with any of the projects listed.

_Note: When using `--serverjars.com` argument some choices may not work as they may not be added to serverjars.com_

### Examples

### Optional Download Arguments
1. --o `The path of where the jar should go Example: --o /mc-servers/hub/Paper.jar`
2. --serverjars.com `When this argument is used it will download the jar from` [ServerJars.com](https://serverjars.com/) `instead`
3. --c `The channel so for geyser the default channel is "standalone" but can be changed to (spigot, bungeecord, velocity, fabric, sponge)`
4. --latest-use-at-your-own-risk `(Warning: Don't use this is bad (you don't want your Minecraft Server randomly getting upgraded to a new Minecraft version, without you knowing)) Using this argument with the latest version, It will find the latest version of the software for you (really used for something like Geyser or Velocity)`
5. --latest-dont-include-snapshot-versions `When searching for the latest version, it will not include snapshot versions`
### Download Usage

*Usage: &lt;software&gt; &lt;version&gt;*

```
./limonium download paper 1.20.1
```

```
./limonium download paper 1.20.1 --o Paper.jar
```

```
./limonium download paper 1.20.1 --o ./mc-servers/hub/Paper.jar
```

## Backup Function

The backup function will back up the folders you specify, and compress them and put them in the backup directory.
By default, it will use tar.gz, unless specified otherwise.

### Optional Backup Arguments
1. --zip `Uses zip instead of tar.gz for backups.`
2. --exclude `Excludes files from the backup`
3. --sftp `Uploads the backup to a SFTP server. Example 1: --sftp user@host:22 /remote/path Example 2: --sftp "user@host:22 path/to/key /remote/path"` (**Password Authentication is not supported.**)
4. --delete-after-upload `Deletes the backup after uploading it to the SFTP server.`

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

The log function will search the logs for the text you specify.

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
