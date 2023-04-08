# Limonium

## Limonium is a tiny Minecraft Server management tool.

It can update your server jar, and backup your server.

The core feature of Limonium, and why it was created, was to update Paper (that's it) \
The other feature(s?) like the backup feature was an afterthought.

Limonium uses platform specific download APIs(Paper, Purpur) or Jenkins to download the .jars

### Global Arguments
1. --help `Shows the help menu`
2. --version `Shows the version of Limonium`
3. --self-update `Updates limonium if there is a new version available`

## Download Function

### Softwares

1. [Paper](https://github.com/PaperMC/Paper) -> `./limonium downlaod paper 1.19.4`
2. [Purpur](https://github.com/PurpurMC/Purpur) -> `./limonium downlaod purpur 1.19.4`
3. [Pufferfish](https://github.com/pufferfish-gg/Pufferfish) -> `./limonium downlaod pufferfish 1.19.4`
4. [Geyser](https://github.com/GeyserMc/Geyser) -> `./limonium downlaod geyser 2.1.0` (default is geyser-standalone)
6. [Spigot](https://hub.spigotmc.org/stash/projects/SPIGOT/repos/spigot/browse) -> `./limonium download spigot 1.19.4` (Not recommended)\
__If you choose Spigot then it will install BuildTools.jar to ./lmtmp/ then run it__

### Proxies

1. [Waterfall](https://github.com/PaperMC/Waterfall) -> `./limonium download waterfall 1.19`

### Important

Limonium is not affiliated with any of the projects listed. It is just a tool to make it easier to download them.

_Note: When using `--serverjars.com` argument some choices may not work as they may not be added to serverjars.com_

### Examples

### Optional Download Arguments
1. --o `The path of where the jar should go Example: --o /mc-servers/hub/Paper.jar`
2. --serverjars.com `When this argument is used it will download the jar from` [ServerJars.com](https://serverjars.com/) `instead`
3. --c `The channel so for geyser the default channel is "standalone" but can be changed to (spigot, bungeecord, velocity, fabric, sponge)`

### Download Usage

*Usage: &lt;software&gt; &lt;version&gt;*

```
./limonium download paper 1.19.4
```

```
./limonium download paper 1.19.4 --o Paper.jar
```

```
./limonium download paper 1.19.4 --o ./mc-servers/hub/Paper.jar
```

## Backup Function

The backup function will back up the folders you specify, and compress them and put them in the backup directory.
By default, it will use tar.gz, unless specified otherwise.

### Optional Backup Arguments
1. --zip `Uses zip instead of tar.gz for backups.`
2. --exclude `Excludes files from the backup`

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

## Why is this simple thing in Rust?
Well, I wanted to learn the Rust programming language, but I didn't know what to make, so I thought of something simple.

## Building

`cargo build -r`
