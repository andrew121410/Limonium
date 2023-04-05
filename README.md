# Limonium

## Limonium is a tiny Minecraft Server management tool.

It can update your server jar, and backup your server.

The core feature of Limonium, and why it was created, was to update Paper (that's it) \
The other feature(s?) like the backup feature was an afterthought.

Limonium uses platform specific download APIs(Paper, Purpur) or Jenkins to download the .jars

## Download Function

### Softwares

1. [Paper](https://github.com/PaperMC/Paper) -> `./limonium downlaod paper 1.19.4`
2. [Purpur](https://github.com/PurpurMC/Purpur) -> `./limonium downlaod purpur 1.19.4`
3. [Pufferfish](https://github.com/pufferfish-gg/Pufferfish) -> `./limonium downlaod pufferfish 1.19.4`
6. [Spigot](https://hub.spigotmc.org/stash/projects/SPIGOT/repos/spigot/browse) -> `./limonium download spigot 1.19.4` (Not recommended)\
__If you choose Spigot then it will install BuildTools.jar to ./lmtmp/ then run it__

### Proxies

1. [Waterfall](https://github.com/PaperMC/Waterfall) -> `./limonium waterfall 1.19`

### Important

Limonium is not affiliated with any of the projects listed. It is just a tool to make it easier to download them.

_Note: When using `-serverjars.com` argument some choices may not work as they may not be added to serverjars.com_

### Examples

*Usage: &lt;project_id&gt; &lt;version&gt;*

### Main Arguments
1. --o `The path of where the jar should go Example: --o /mc-servers/hub/Paper.jar`
2. --serverjars.com `When this argument is used it will download the jar from` [ServerJars.com](https://serverjars.com/) `instead`
3. --self-update `Updates limonium if there is a new version available`

### Main Usage
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

### Backup Arguments
1. --backup `Easy way to backup your server. Example: --backup survival . ../survival-backups/`
2. --zip `Uses zip instead of tar.gz for backups. Example: --zip --backup survival . ../survival-backups/`
3. --exclude `Excludes files from the backup. Example: --backup survival . ../survival-backups/ --exclude logs:plugins/dynmap"`

### Backup Usage
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
